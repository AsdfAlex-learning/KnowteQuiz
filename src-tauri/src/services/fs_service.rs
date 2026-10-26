use crate::errors::AppError;
use crate::models::note::{
    DirectoryEntry, DirectoryListing, NoteIndex, NoteIndexEntry, NoteNodeKind, NoteTreeNode,
};
use crate::services::{config, note_service, storage};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use tokio::fs;

const NOTE_INDEX_FILENAME: &str = "index.json";
const NOTE_INDEX_VERSION: &str = "1.0.0";

/// Resolve `candidate` against the configured note root: canonicalizes both
/// and rejects anything outside the root. Shared by the web server (network
/// exposure) and the quiz engine so prompts can only contain workspace notes.
pub async fn resolve_note_path(data_dir: &Path, candidate: &str) -> Result<PathBuf, AppError> {
    let settings = config::get_settings_path(data_dir)?;
    let root = settings
        .workspace
        .root_path
        .ok_or_else(|| AppError::InvalidInput("No note root configured".to_string()))?;
    let root_canon = fs::canonicalize(&root)
        .await
        .map_err(|e| AppError::InvalidInput(format!("Invalid note root: {e}")))?;
    let target_canon = fs::canonicalize(candidate)
        .await
        .map_err(|_| AppError::NotFound(format!("File does not exist: {candidate}")))?;
    if !target_canon.starts_with(&root_canon) {
        return Err(AppError::InvalidInput(
            "Path is outside the configured note root".to_string(),
        ));
    }
    Ok(target_canon)
}

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
        let metadata = fs::metadata(path).await.map_err(|e| {
            AppError::Internal(format!("Failed to inspect note {}: {}", node.path, e))
        })?;
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

    let mut read_dir = fs::read_dir(dir).await.map_err(|e| {
        AppError::Internal(format!("Failed to read directory {}: {}", dir.display(), e))
    })?;

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
                kind: NoteNodeKind::Markdown,
                is_dir: true,
                children,
            });
        } else if is_markdown_path(&path) {
            files.push(NoteTreeNode {
                name,
                path: full_path,
                kind: NoteNodeKind::Markdown,
                is_dir: false,
                children: vec![],
            });
        } else if is_document_path(&path) {
            files.push(NoteTreeNode {
                name,
                path: full_path,
                kind: NoteNodeKind::Document,
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
    // Derived asset directories produced by document conversion
    // (`design.docx.images/`) belong to a note and must not appear in the tree.
    if name.ends_with(".images") && name.len() > ".images".len() {
        return true;
    }
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

/// Extensions the document-conversion feature accepts. Mirrors the `doc_parser`
/// whitelist; keep the two in sync.
pub(crate) const DOCUMENT_EXTENSIONS: [&str; 5] = ["pdf", "docx", "pptx", "html", "htm"];

/// True when the file is a conversion candidate (PDF / DOCX / PPTX / HTML).
pub(crate) fn is_document_path(path: &Path) -> bool {
    path.extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| ext.to_ascii_lowercase())
        .is_some_and(|ext| DOCUMENT_EXTENSIONS.contains(&ext.as_str()))
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

const MAX_DIRECTORY_ENTRIES: usize = 500;

/// Lists subdirectories for the in-app folder picker. Hidden folders and a
/// couple of Windows system folders are skipped; files are never returned.
pub fn list_directories(path: Option<&str>) -> Result<DirectoryListing, AppError> {
    let roots = directory_roots();
    let current = match path.map(str::trim).filter(|value| !value.is_empty()) {
        Some(value) => PathBuf::from(value),
        None => default_start_dir(&roots),
    };

    let metadata = std::fs::metadata(&current).map_err(|_| {
        AppError::NotFound(format!("Directory does not exist: {}", current.display()))
    })?;
    if !metadata.is_dir() {
        return Err(AppError::InvalidInput(format!(
            "Path is not a directory: {}",
            current.display()
        )));
    }

    let mut directories: Vec<DirectoryEntry> = Vec::new();
    let entries = std::fs::read_dir(&current).map_err(|error| {
        AppError::Internal(format!(
            "Failed to read directory {}: {}",
            current.display(),
            error
        ))
    })?;
    for entry in entries.flatten() {
        if directories.len() >= MAX_DIRECTORY_ENTRIES {
            break;
        }
        let name = entry.file_name().to_string_lossy().to_string();
        if name.starts_with('.') || is_ignored_directory_name(&name) {
            continue;
        }
        let entry_path = entry.path();
        if !entry_path.is_dir() {
            continue;
        }
        directories.push(DirectoryEntry {
            name,
            path: entry_path.to_string_lossy().to_string(),
        });
    }
    directories.sort_by_key(|entry| entry.name.to_lowercase());

    Ok(DirectoryListing {
        path: current.to_string_lossy().to_string(),
        parent: current
            .parent()
            .map(|parent| parent.to_string_lossy().to_string()),
        directories,
        roots,
    })
}

fn default_start_dir(roots: &[DirectoryEntry]) -> PathBuf {
    dirs::home_dir()
        .filter(|home| home.is_dir())
        .or_else(|| roots.first().map(|root| PathBuf::from(&root.path)))
        .unwrap_or_else(|| PathBuf::from("."))
}

fn directory_roots() -> Vec<DirectoryEntry> {
    let mut roots = Vec::new();

    #[cfg(target_os = "windows")]
    {
        for letter in b'A'..=b'Z' {
            let share = format!("{}:", letter as char);
            let path = format!("{}:\\", letter as char);
            if Path::new(&path).exists() {
                roots.push(DirectoryEntry { name: share, path });
            }
        }
    }

    #[cfg(not(target_os = "windows"))]
    {
        if let Some(home) = dirs::home_dir().filter(|home| home.is_dir()) {
            let name = home
                .file_name()
                .map(|value| value.to_string_lossy().to_string())
                .unwrap_or_else(|| "Home".to_string());
            roots.push(DirectoryEntry {
                name,
                path: home.to_string_lossy().to_string(),
            });
        }
        roots.push(DirectoryEntry {
            name: "/".to_string(),
            path: "/".to_string(),
        });

        #[cfg(target_os = "macos")]
        {
            if let Ok(entries) = std::fs::read_dir("/Volumes") {
                for entry in entries.flatten() {
                    if entry.path().is_dir() {
                        roots.push(DirectoryEntry {
                            name: entry.file_name().to_string_lossy().to_string(),
                            path: entry.path().to_string_lossy().to_string(),
                        });
                    }
                }
            }
        }
    }

    roots
}

fn is_ignored_directory_name(name: &str) -> bool {
    matches!(name, "$Recycle.Bin" | "System Volume Information")
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
        std_fs::create_dir_all(root.join("node_modules/pkg"))
            .expect("node_modules should be created");
        std_fs::write(root.join("node_modules/pkg/ignored.md"), "# Ignored")
            .expect("ignored note should be written");
        std_fs::create_dir_all(root.join("target/debug")).expect("target should be created");
        std_fs::write(root.join("target/debug/ignored.md"), "# Ignored")
            .expect("ignored note should be written");

        let tree = scan_directory(root.to_string_lossy().as_ref())
            .await
            .expect("scan should succeed");

        assert_eq!(tree.len(), 1);
        assert_eq!(tree[0].name, "real.md");
    }

    #[tokio::test]
    async fn scan_directory_accepts_markdown_extensions_case_insensitively() {
        let root = temp_notes_dir("scan_directory_accepts_markdown_extensions_case_insensitively");
        std_fs::write(root.join("README.MD"), "# Readme")
            .expect("uppercase note should be written");
        std_fs::write(root.join("longform.Markdown"), "# Longform")
            .expect("markdown note should be written");
        std_fs::write(root.join("draft.mdx"), "# Draft").expect("mdx file should be written");

        let tree = scan_directory(root.to_string_lossy().as_ref())
            .await
            .expect("scan should succeed");
        let names = tree
            .iter()
            .map(|node| node.name.as_str())
            .collect::<Vec<_>>();

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
        assert_eq!(
            index.notes[0].path,
            root.join("ownership.md").to_string_lossy()
        );
        assert_eq!(index.notes[0].title, "Cached Ownership");
        assert_eq!(index.notes[0].size_bytes, note_text.len() as u64);
        assert!(index.notes[0].modified_at.is_some());
    }

    #[tokio::test]
    async fn scan_directory_with_index_keeps_tree_when_note_content_cannot_be_indexed() {
        let root = temp_notes_dir(
            "scan_directory_with_index_keeps_tree_when_note_content_cannot_be_indexed",
        );
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

    #[tokio::test]
    async fn scan_directory_lists_documents_as_document_kind() {
        let root = temp_notes_dir("scan_directory_lists_documents_as_document_kind");
        std_fs::write(root.join("paper.pdf"), "%PDF").expect("pdf should be written");
        std_fs::write(root.join("deck.pptx"), "PK").expect("pptx should be written");
        std_fs::write(root.join("spec.docx"), "PK").expect("docx should be written");
        std_fs::write(root.join("page.html"), "<p>").expect("html should be written");
        std_fs::write(root.join("note.md"), "# Note").expect("md should be written");

        let tree = scan_directory(root.to_string_lossy().as_ref())
            .await
            .expect("scan should succeed");

        let by_name = |name: &str| {
            tree.iter()
                .find(|node| node.name == name)
                .unwrap_or_else(|| panic!("{name} should be in the tree"))
        };

        assert_eq!(by_name("note.md").kind, NoteNodeKind::Markdown);
        for doc in ["paper.pdf", "deck.pptx", "spec.docx", "page.html"] {
            assert_eq!(
                by_name(doc).kind,
                NoteNodeKind::Document,
                "{doc} should be a document"
            );
        }
    }

    #[tokio::test]
    async fn scan_directory_hides_derived_asset_directories() {
        let root = temp_notes_dir("scan_directory_hides_derived_asset_directories");
        std_fs::write(root.join("design.docx.md"), "# Converted")
            .expect("converted note should be written");
        std_fs::create_dir_all(root.join("design.docx.images"))
            .expect("images dir should be created");
        std_fs::write(root.join("design.docx.images/image001.png"), "PNG")
            .expect("image should be written");
        std_fs::create_dir_all(root.join("assets")).expect("user dir should be created");
        std_fs::write(root.join("assets/real.png"), "PNG").expect("user image should be written");

        let tree = scan_directory(root.to_string_lossy().as_ref())
            .await
            .expect("scan should succeed");
        let names = tree
            .iter()
            .map(|node| node.name.as_str())
            .collect::<Vec<_>>();

        assert!(
            !names.contains(&"design.docx.images"),
            "derived asset dir must be hidden, got {names:?}"
        );
        assert!(
            !names.contains(&"design.docx.images/image001.png"),
            "derived asset content must not leak"
        );
        assert!(
            names.contains(&"design.docx.md"),
            "converted note must show"
        );
        assert!(names.contains(&"assets"), "user dirs must still show");
        assert!(
            !names.contains(&"assets/real.png"),
            "non-whitelisted files stay hidden"
        );
    }

    #[test]
    fn is_document_path_accepts_only_whitelisted_extensions() {
        for doc in ["a.pdf", "a.PDF", "a.docx", "a.pptx", "a.html", "a.htm"] {
            assert!(
                is_document_path(std::path::Path::new(doc)),
                "{doc} should be accepted"
            );
        }
        for other in ["a.md", "a.png", "a.txt", "a", "a.doc", "a.xls"] {
            assert!(
                !is_document_path(std::path::Path::new(other)),
                "{other} should be rejected"
            );
        }
    }

    #[test]
    fn list_directories_returns_sorted_subdirectories_and_roots() {
        let root = temp_notes_dir("list_directories_returns_sorted_subdirectories_and_roots");
        std_fs::create_dir_all(root.join("beta")).expect("beta should be created");
        std_fs::create_dir_all(root.join("Alpha")).expect("Alpha should be created");
        std_fs::create_dir_all(root.join(".hidden")).expect("hidden should be created");
        std_fs::write(root.join("note.md"), "# Note").expect("file should be written");

        let listing =
            list_directories(Some(root.to_string_lossy().as_ref())).expect("listing should work");

        let names: Vec<&str> = listing
            .directories
            .iter()
            .map(|entry| entry.name.as_str())
            .collect();
        assert_eq!(names, vec!["Alpha", "beta"]);
        assert!(!listing.roots.is_empty());
        assert_eq!(listing.path, root.to_string_lossy());
        assert!(listing.parent.is_some());
    }

    #[test]
    fn list_directories_rejects_missing_or_file_paths() {
        assert!(list_directories(Some("definitely/not/a/real/dir")).is_err());

        let root = temp_notes_dir("list_directories_rejects_missing_or_file_paths");
        let file = root.join("note.md");
        std_fs::write(&file, "x").expect("file should be written");

        assert!(list_directories(Some(file.to_string_lossy().as_ref())).is_err());
    }

    #[test]
    fn list_directories_without_a_path_starts_at_a_root() {
        let listing = list_directories(None).expect("default listing should work");

        assert!(!listing.roots.is_empty());
        assert!(Path::new(&listing.path).is_dir());
    }
}
