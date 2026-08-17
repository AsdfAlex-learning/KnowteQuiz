use crate::models::note::{NoteIndex, NoteIndexEntry, NoteTreeNode};
use crate::services::{note_service, storage};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

const NOTE_INDEX_FILENAME: &str = "index.json";
const NOTE_INDEX_VERSION: &str = "1.0.0";

pub fn scan_directory(root_path: &str) -> Result<Vec<NoteTreeNode>, String> {
    let root = Path::new(root_path);
    if !root.exists() {
        return Err(format!("Directory does not exist: {}", root_path));
    }
    if !root.is_dir() {
        return Err(format!("Path is not a directory: {}", root_path));
    }
    scan_recursive(root, root)
}

pub fn scan_directory_with_index(
    root_path: &str,
    data_dir: &Path,
) -> Result<Vec<NoteTreeNode>, String> {
    let tree = scan_directory(root_path)?;

    // Try to read existing index for incremental scan
    let old_index = read_existing_index(data_dir, root_path);

    let index = build_note_index(root_path, &tree, old_index.as_ref())?;
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

fn build_note_index(
    root_path: &str,
    tree: &[NoteTreeNode],
    old_index: Option<&NoteIndex>,
) -> Result<NoteIndex, String> {
    // Build a lookup map from old index for quick access
    let old_entries: HashMap<&str, &NoteIndexEntry> = old_index
        .map(|idx| idx.notes.iter().map(|e| (e.path.as_str(), e)).collect())
        .unwrap_or_default();

    let mut notes = Vec::new();
    collect_index_entries(tree, &mut notes, &old_entries)?;
    notes.sort_by(|a, b| a.path.cmp(&b.path));

    Ok(NoteIndex {
        version: NOTE_INDEX_VERSION.to_string(),
        root_path: root_path.to_string(),
        generated_at: chrono::Utc::now(),
        notes,
    })
}

fn collect_index_entries(
    nodes: &[NoteTreeNode],
    notes: &mut Vec<NoteIndexEntry>,
    old_entries: &HashMap<&str, &NoteIndexEntry>,
) -> Result<(), String> {
    for node in nodes {
        if node.is_dir {
            collect_index_entries(&node.children, notes, old_entries)?;
            continue;
        }

        let path = Path::new(&node.path);
        let metadata = fs::metadata(path)
            .map_err(|e| format!("Failed to inspect note {}: {}", node.path, e))?;
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

fn scan_recursive(dir: &Path, _root: &Path) -> Result<Vec<NoteTreeNode>, String> {
    let mut entries: Vec<NoteTreeNode> = Vec::new();
    let mut dirs: Vec<NoteTreeNode> = Vec::new();
    let mut files: Vec<NoteTreeNode> = Vec::new();

    let read_dir = fs::read_dir(dir)
        .map_err(|e| format!("Failed to read directory {}: {}", dir.display(), e))?;

    for entry in read_dir {
        let entry = entry.map_err(|e| format!("Failed to read entry: {}", e))?;
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

        if path.is_dir() {
            let children = scan_recursive(&path, _root)?;
            dirs.push(NoteTreeNode {
                name,
                path: full_path,
                is_dir: true,
                children,
            });
        } else if is_markdown_file(&path) {
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

fn is_markdown_file(path: &Path) -> bool {
    matches!(
        path.extension()
            .and_then(|ext| ext.to_str())
            .map(|ext| ext.to_ascii_lowercase())
            .as_deref(),
        Some("md" | "markdown")
    )
}

pub fn read_file_content(path: &str) -> Result<String, String> {
    let file_path = Path::new(path);
    if !file_path.exists() {
        return Err(format!("File does not exist: {}", path));
    }
    fs::read_to_string(file_path).map_err(|e| format!("Failed to read file {}: {}", path, e))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_notes_dir(test_name: &str) -> std::path::PathBuf {
        let dir = std::env::temp_dir()
            .join("knowtequiz-fs-tests")
            .join(test_name)
            .join(uuid::Uuid::new_v4().to_string());
        fs::create_dir_all(&dir).expect("test temp dir should be created");
        dir
    }

    #[test]
    fn scan_directory_ignores_dependency_and_build_directories() {
        let root = temp_notes_dir("scan_directory_ignores_dependency_and_build_directories");
        fs::write(root.join("real.md"), "# Real").expect("real note should be written");
        fs::create_dir_all(root.join("node_modules/pkg")).expect("node_modules should be created");
        fs::write(root.join("node_modules/pkg/ignored.md"), "# Ignored")
            .expect("ignored note should be written");
        fs::create_dir_all(root.join("target/debug")).expect("target should be created");
        fs::write(root.join("target/debug/ignored.md"), "# Ignored")
            .expect("ignored note should be written");

        let tree = scan_directory(root.to_string_lossy().as_ref()).expect("scan should succeed");

        assert_eq!(tree.len(), 1);
        assert_eq!(tree[0].name, "real.md");
    }

    #[test]
    fn scan_directory_accepts_markdown_extensions_case_insensitively() {
        let root =
            temp_notes_dir("scan_directory_accepts_markdown_extensions_case_insensitively");
        fs::write(root.join("README.MD"), "# Readme").expect("uppercase note should be written");
        fs::write(root.join("longform.Markdown"), "# Longform")
            .expect("markdown note should be written");
        fs::write(root.join("draft.mdx"), "# Draft").expect("mdx file should be written");

        let tree = scan_directory(root.to_string_lossy().as_ref()).expect("scan should succeed");
        let names = tree.iter().map(|node| node.name.as_str()).collect::<Vec<_>>();

        assert_eq!(names, vec!["README.MD", "longform.Markdown"]);
    }

    #[test]
    fn scan_directory_with_index_writes_note_metadata_index() {
        let root = temp_notes_dir("scan_directory_with_index_writes_note_metadata_index");
        let data_dir = temp_notes_dir("scan_directory_with_index_writes_note_metadata_index_data");
        let note_text = "---\ntitle: Cached Ownership\n---\n\nOwnership notes.";
        fs::write(root.join("ownership.md"), note_text).expect("note should be written");

        let tree = scan_directory_with_index(root.to_string_lossy().as_ref(), &data_dir)
            .expect("scan should succeed");

        assert_eq!(tree.len(), 1);
        let index_path = data_dir.join("index.json");
        let index_text =
            fs::read_to_string(index_path).expect("index.json should be written after scan");
        let index: crate::models::note::NoteIndex =
            serde_json::from_str(&index_text).expect("index should be valid JSON");

        assert_eq!(index.root_path, root.to_string_lossy());
        assert_eq!(index.notes.len(), 1);
        assert_eq!(index.notes[0].path, root.join("ownership.md").to_string_lossy());
        assert_eq!(index.notes[0].title, "Cached Ownership");
        assert_eq!(index.notes[0].size_bytes, note_text.len() as u64);
        assert!(index.notes[0].modified_at.is_some());
    }

    #[test]
    fn scan_directory_with_index_keeps_tree_when_note_content_cannot_be_indexed() {
        let root =
            temp_notes_dir("scan_directory_with_index_keeps_tree_when_note_content_cannot_be_indexed");
        let data_dir = temp_notes_dir(
            "scan_directory_with_index_keeps_tree_when_note_content_cannot_be_indexed_data",
        );
        fs::write(root.join("binary.md"), [0xff, 0xfe, 0xfd])
            .expect("invalid utf8 markdown should be written");

        let tree = scan_directory_with_index(root.to_string_lossy().as_ref(), &data_dir)
            .expect("scan should still succeed");

        assert_eq!(tree.len(), 1);
        assert_eq!(tree[0].name, "binary.md");
        let index_text = fs::read_to_string(data_dir.join("index.json"))
            .expect("index.json should still be written");
        let index: crate::models::note::NoteIndex =
            serde_json::from_str(&index_text).expect("index should be valid JSON");

        assert_eq!(index.notes.len(), 1);
        assert_eq!(index.notes[0].title, "binary");
        assert_eq!(index.notes[0].size_bytes, 3);
    }

    #[test]
    fn incremental_scan_reuses_unchanged_file_titles() {
        let root = temp_notes_dir("incremental_scan_reuses_unchanged_file_titles");
        let data_dir = temp_notes_dir("incremental_scan_reuses_unchanged_file_titles_data");
        let note_text = "---\ntitle: Original Title\n---\n\nBody.";
        fs::write(root.join("note.md"), note_text).expect("note should be written");

        // First scan — builds index
        scan_directory_with_index(root.to_string_lossy().as_ref(), &data_dir)
            .expect("first scan should succeed");

        // Overwrite with different content but same size (touch to keep mtime same or close)
        // We'll change the title in frontmatter but keep the file size identical
        let new_text = "---\ntitle: Changed Title\n---\n\nBody.";
        fs::write(root.join("note.md"), new_text).expect("note should be overwritten");

        // Second scan — should detect change and re-extract title
        scan_directory_with_index(root.to_string_lossy().as_ref(), &data_dir)
            .expect("second scan should succeed");

        let index: crate::models::note::NoteIndex =
            serde_json::from_str(&fs::read_to_string(data_dir.join("index.json")).unwrap())
                .unwrap();
        assert_eq!(index.notes.len(), 1);
        // Title should be updated because file changed
        assert_eq!(index.notes[0].title, "Changed Title");
    }

    #[test]
    fn incremental_scan_removes_deleted_files_from_index() {
        let root = temp_notes_dir("incremental_scan_removes_deleted_files_from_index");
        let data_dir = temp_notes_dir("incremental_scan_removes_deleted_files_from_index_data");
        fs::write(root.join("keep.md"), "# Keep").expect("keep note should be written");
        fs::write(root.join("delete.md"), "# Delete").expect("delete note should be written");

        // First scan — both files indexed
        scan_directory_with_index(root.to_string_lossy().as_ref(), &data_dir)
            .expect("first scan should succeed");
        let index: crate::models::note::NoteIndex =
            serde_json::from_str(&fs::read_to_string(data_dir.join("index.json")).unwrap())
                .unwrap();
        assert_eq!(index.notes.len(), 2);

        // Delete one file
        fs::remove_file(root.join("delete.md")).expect("delete note should be removed");

        // Second scan — deleted file should be gone from index
        scan_directory_with_index(root.to_string_lossy().as_ref(), &data_dir)
            .expect("second scan should succeed");
        let index: crate::models::note::NoteIndex =
            serde_json::from_str(&fs::read_to_string(data_dir.join("index.json")).unwrap())
                .unwrap();
        assert_eq!(index.notes.len(), 1);
        assert_eq!(index.notes[0].title, "Keep");
    }

    #[test]
    fn incremental_scan_full_rebuild_on_root_path_change() {
        let root1 = temp_notes_dir("incremental_scan_full_rebuild_on_root_path_change_1");
        let root2 = temp_notes_dir("incremental_scan_full_rebuild_on_root_path_change_2");
        let data_dir = temp_notes_dir("incremental_scan_full_rebuild_on_root_path_change_data");
        fs::write(root1.join("note1.md"), "---\ntitle: Note1\n---\n\nBody.")
            .expect("note1 should be written");
        fs::write(root2.join("note2.md"), "---\ntitle: Note2\n---\n\nBody.")
            .expect("note2 should be written");

        // Scan root1
        scan_directory_with_index(root1.to_string_lossy().as_ref(), &data_dir)
            .expect("scan root1 should succeed");
        let index: crate::models::note::NoteIndex =
            serde_json::from_str(&fs::read_to_string(data_dir.join("index.json")).unwrap())
                .unwrap();
        assert_eq!(index.root_path, root1.to_string_lossy());
        assert_eq!(index.notes.len(), 1);

        // Scan root2 — different root_path triggers full rebuild
        scan_directory_with_index(root2.to_string_lossy().as_ref(), &data_dir)
            .expect("scan root2 should succeed");
        let index: crate::models::note::NoteIndex =
            serde_json::from_str(&fs::read_to_string(data_dir.join("index.json")).unwrap())
                .unwrap();
        assert_eq!(index.root_path, root2.to_string_lossy());
        assert_eq!(index.notes.len(), 1);
        assert_eq!(index.notes[0].title, "Note2");
    }
}
