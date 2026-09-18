use crate::errors::AppError;
use crate::models::note::{NoteIndex, NoteIndexEntry, NoteTreeNode};
use crate::services::{note_service, storage};
use std::collections::HashMap;
use std::path::Path;
use tokio::fs;

const NOTE_INDEX_FILENAME: &str = "index.json";
const NOTE_INDEX_VERSION: &str = "1.0.0";

pub async fn scan_directory(root_path: &str) -> Result<Vec<NoteTreeNode>, AppError> {
    let root = Path::new(root_path);
    let metadata = fs::metadata(root)
        .await
        .map_err(|_| AppError::NotFound(format!("Directory does not exist: {}", root_path)))?;
    if !metadata.is_dir() {
        return Err(AppError::InvalidInput(format!(
            "Path is not a directory: {}",
            root_path
        )));
    }
    scan_recursive(root, root).await
}

pub async fn scan_directory_with_index(
    root_path: &str,
    data_dir: &Path,
) -> Result<Vec<NoteTreeNode>, AppError> {
    let tree = scan_directory(root_path).await?;

    // Try to read existing index for incremental scan
    let old_index = read_existing_index(data_dir, root_path);

    let index = build_note_index(root_path, &tree, old_index.as_ref()).await?;
    storage::write_json_path(data_dir, NOTE_INDEX_FILENAME, &index)?;
    Ok(tree)
}

fn read_existing_index(data_dir: &Path, root_path: &str) -> Option<NoteIndex> {
    let index: NoteIndex = storage::read_json_path(data_dir, NOTE_INDEX_FILENAME).ok()?;
    if index.version != NOTE_INDEX_VERSION || index.root_path != root_path {
        return None;
    }
    Some(index)
}

async fn build_note_index(
    root_path: &str,
    tree: &[NoteTreeNode],
    old_index: Option<&NoteIndex>,
) -> Result<NoteIndex, AppError> {
    // Build a lookup map from old index for quick access
    let old_entries: HashMap<&str, &NoteIndexEntry> = old_index
        .map(|idx| idx.notes.iter().map(|e| (e.path.as_str(), e)).collect())
        .unwrap_or_default();

    let mut notes = Vec::new();
    collect_index_entries(tree, &mut notes, &old_entries).await?;
    notes.sort_by(|a, b| a.path.cmp(&b.path));

    Ok(NoteIndex {
        version: NOTE_INDEX_VERSION.to_string(),
        root_path: root_path.to_string(),
        generated_at: chrono::Utc::now(),
        notes,
    })
}

async fn collect_index_entries(
    nodes: &[NoteTreeNode],
    notes: &mut Vec<NoteIndexEntry>,
    old_entries: &HashMap<&str, &NoteIndexEntry>,
) -> Result<(), AppError> {
    for node in nodes {
        if node.is_dir {
            Box::pin(collect_index_entries(&node.children, notes, old_entries)).await?;
            continue;
        }

        let path = Path::new(&node.path);
        let metadata = fs::metadata(path)
            .await
            .map_err(|e| AppError::Internal(format!("Failed to inspect note {}: {}", node.path, e)))?;
        let modified_at = metadata
            .modified()
            .ok()
            .map(chrono::DateTime::<chrono::Utc>::from);
        let size_bytes = metadata.len();

        // Check if file is unchanged (same size + modification time)
        if let Some(old) = old_entries.get(node.path.as_str()) {
            if old.size_bytes == size_bytes && old.modified_at == modified_at {
                // Reuse old entry — skip expensive title extraction
                notes.push(NoteIndexEntry {
                    path: node.path.clone(),
                    title: old.title.clone(),
                    size_bytes,
                    modified_at,
                });
                continue;
            }
        }

        // File is new or changed — extract title from content
        let title = fs::read_to_string(path)
            .await
            .map(|content| note_service::extract_metadata(&content, &node.path).title)
            .unwrap_or_else(|_| fallback_note_title(path));

        notes.push(NoteIndexEntry {
            path: node.path.clone(),
            title,
            size_bytes,
            modified_at,
        });
    }
    Ok(())
}

fn fallback_note_title(path: &Path) -> String {
    path.file_stem()
        .and_then(|name| name.to_str())
        .unwrap_or("Untitled")
        .to_string()
}

async fn scan_recursive(dir: &Path, _root: &Path) -> Result<Vec<NoteTreeNode>, AppError> {
    let mut entries: Vec<NoteTreeNode> = Vec::new();
    let mut dirs: Vec<NoteTreeNode> = Vec::new();
    let mut files: Vec<NoteTreeNode> = Vec::new();

    let mut read_dir = fs::read_dir(dir)
        .await
        .map_err(|e| AppError::Internal(format!("Failed to read directory {}: {}", dir.display(), e)))?;

    while let Some(entry) = read_dir
        .next_entry()
        .await
        .map_err(|e| AppError::Internal(format!("Failed to read entry: {}", e)))?
    {
        let path = entry.path();
        let name = path
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string();

        if should_ignore_entry(&name) {
            continue;
        }

        let full_path = path.to_string_lossy().to_string();

        if entry
            .file_type()
            .await
            .map_err(|e| AppError::Internal(format!("Failed to inspect entry: {}", e)))?
            .is_dir()
        {
            let children = Box::pin(scan_recursive(&path, _root)).await?;
            dirs.push(NoteTreeNode {
                name,
                path: full_path,
                is_dir: true,
                children,
            });
        } else if is_markdown_path(&path) {
            files.push(NoteTreeNode {
                name,
                path: full_path,
                is_dir: false,
                children: vec![],
            });
        }
    }

    dirs.sort_by(|a, b| a.name.cmp(&b.name));
    files.sort_by(|a, b| a.name.cmp(&b.name));
    entries.extend(dirs);
    entries.extend(files);
    Ok(entries)
}

fn should_ignore_entry(name: &str) -> bool {
    matches!(
        name,
        ".git"
            | ".hg"
            | ".svn"
            | ".obsidian"
            | "node_modules"
            | "target"
            | "dist"
            | "build"
            | ".sisyphus"
    ) || name.starts_with('.')
}

pub(crate) fn is_markdown_path(path: &Path) -> bool {
    matches!(
        path.extension()
            .and_then(|ext| ext.to_str())
            .map(|ext| ext.to_ascii_lowercase())
            .as_deref(),
        Some("md" | "markdown")
    )
}

pub async fn read_file_content(path: &str) -> Result<String, AppError> {
    let file_path = Path::new(path);
    if fs::metadata(file_path).await.is_err() {
        return Err(AppError::NotFound(format!("File does not exist: {}", path)));
    }
    fs::read_to_string(file_path)
        .await
        .map_err(|e| AppError::Internal(format!("Failed to read file {}: {}", path, e)))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs as std_fs;

    fn temp_notes_dir(test_name: &str) -> std::path::PathBuf {
        let dir = std::env::temp_dir()
            .join("knowtequiz-fs-tests")
            .join(test_name)
            .join(uuid::Uuid::new_v4().to_string());
        std_fs::create_dir_all(&dir).expect("test temp dir should be created");
        dir
    }

    #[tokio::test]
    async fn scan_directory_ignores_dependency_and_build_directories() {
        let root = temp_notes_dir("scan_directory_ignores_dependency_and_build_directories");
        std_fs::write(root.join("real.md"), "# Real").expect("real note should be written");
        std_fs::create_dir_all(root.join("node_modules/pkg")).expect("node_modules should be created");
        std_fs::write(root.join("node_modules/pkg/ignored.md"), "# Ignored")
            .expect("ignored note should be written");
        std_fs::create_dir_all(root.join("target/debug")).expect("target should be created");
        std_fs::write(root.join("target/debug/ignored.md"), "# Ignored")
            .expect("ignored note should be written");

        let tree = scan_directory(root.to_string_lossy().as_ref()).await.expect("scan should succeed");

        assert_eq!(tree.len(), 1);
        assert_eq!(tree[0].name, "real.md");
    }

    #[tokio::test]
    async fn scan_directory_accepts_markdown_extensions_case_insensitively() {
        let root =
            temp_notes_dir("scan_directory_accepts_markdown_extensions_case_insensitively");
        std_fs::write(root.join("README.MD"), "# Readme").expect("uppercase note should be written");
        std_fs::write(root.join("longform.Markdown"), "# Longform")
            .expect("markdown note should be written");
        std_fs::write(root.join("draft.mdx"), "# Draft").expect("mdx file should be written");

        let tree = scan_directory(root.to_string_lossy().as_ref()).await.expect("scan should succeed");
        let names = tree.iter().map(|node| node.name.as_str()).collect::<Vec<_>>();

        assert_eq!(names, vec!["README.MD", "longform.Markdown"]);
    }

    #[tokio::test]
    async fn scan_directory_with_index_writes_note_metadata_index() {
        let root = temp_notes_dir("scan_directory_with_index_writes_note_metadata_index");
        let data_dir = temp_notes_dir("scan_directory_with_index_writes_note_metadata_index_data");
        let note_text = "---\ntitle: Cached Ownership\n---\n\nOwnership notes.";
        std_fs::write(root.join("ownership.md"), note_text).expect("note should be written");

        let tree = scan_directory_with_index(root.to_string_lossy().as_ref(), &data_dir)
            .await
            .expect("scan should succeed");

        assert_eq!(tree.len(), 1);
        let index_path = data_dir.join("index.json");
        let index_text =
            std_fs::read_to_string(index_path).expect("index.json should be written after scan");
        let index: crate::models::note::NoteIndex =
            serde_json::from_str(&index_text).expect("index should be valid JSON");

        assert_eq!(index.root_path, root.to_string_lossy());
        assert_eq!(index.notes.len(), 1);
        assert_eq!(index.notes[0].path, root.join("ownership.md").to_string_lossy());
        assert_eq!(index.notes[0].title, "Cached Ownership");
        assert_eq!(index.notes[0].size_bytes, note_text.len() as u64);
        assert!(index.notes[0].modified_at.is_some());
    }

    #[tokio::test]
    async fn scan_directory_with_index_keeps_tree_when_note_content_cannot_be_indexed() {
        let root =
            temp_notes_dir("scan_directory_with_index_keeps_tree_when_note_content_cannot_be_indexed");
        let data_dir = temp_notes_dir(
            "scan_directory_with_index_keeps_tree_when_note_content_cannot_be_indexed_data",
        );
        std_fs::write(root.join("binary.md"), [0xff, 0xfe, 0xfd])
            .expect("invalid utf8 markdown should be written");

        let tree = scan_directory_with_index(root.to_string_lossy().as_ref(), &data_dir)
            .await
            .expect("scan should still succeed");

        assert_eq!(tree.len(), 1);
        assert_eq!(tree[0].name, "binary.md");
    }

    #[tokio::test]
    async fn scan_directory_returns_error_for_nonexistent_path() {
        let result = scan_directory("/nonexistent/path/that/does/not/exist").await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn scan_directory_with_index_preserves_unchanged_notes() {
        let root = temp_notes_dir("scan_directory_with_index_preserves_unchanged_notes");
        let data_dir = temp_notes_dir("scan_directory_with_index_preserves_unchanged_notes_data");
        let note_text = "---\ntitle: Ownership\n---\n\nNotes.";
        std_fs::write(root.join("ownership.md"), note_text).expect("note should be written");

        // First scan — builds index
        let _ = scan_directory_with_index(root.to_string_lossy().as_ref(), &data_dir)
            .await
            .expect("first scan should succeed");

        // Second scan — should reuse cached titles
        let tree = scan_directory_with_index(root.to_string_lossy().as_ref(), &data_dir)
            .await
            .expect("second scan should succeed");

        assert_eq!(tree.len(), 1);
        assert_eq!(tree[0].name, "ownership.md");
    }

    #[tokio::test]
    async fn scan_directory_with_index_rebuilds_when_note_content_changes() {
        let root = temp_notes_dir("scan_directory_with_index_rebuilds_when_note_content_changes");
        let data_dir =
            temp_notes_dir("scan_directory_with_index_rebuilds_when_note_content_changes_data");

        std_fs::write(root.join("note.md"), "---\ntitle: Old\n---\n\nOld content.")
            .expect("note should be written");

        let _ = scan_directory_with_index(root.to_string_lossy().as_ref(), &data_dir)
            .await
            .expect("first scan should succeed");

        // Modify the note
        std_fs::write(root.join("note.md"), "---\ntitle: New\n---\n\nNew content.")
            .expect("note should be updated");

        let tree = scan_directory_with_index(root.to_string_lossy().as_ref(), &data_dir)
            .await
            .expect("second scan should succeed");

        assert_eq!(tree.len(), 1);

        let index_path = data_dir.join("index.json");
        let index: crate::models::note::NoteIndex =
            serde_json::from_str(&std_fs::read_to_string(index_path).unwrap()).unwrap();
        assert_eq!(index.notes[0].title, "New");
    }

    #[tokio::test]
    async fn read_file_content_returns_error_for_missing_file() {
        let result = read_file_content("/nonexistent/file.md").await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn read_file_content_returns_content_for_existing_file() {
        let dir = temp_notes_dir("read_file_content_returns_content_for_existing_file");
        let file_path = dir.join("test.md");
        std_fs::write(&file_path, "# Hello").expect("test file should be written");

        let content = read_file_content(file_path.to_string_lossy().as_ref())
            .await
            .expect("read should succeed");
        assert_eq!(content, "# Hello");
    }
}
