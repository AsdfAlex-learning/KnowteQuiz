use crate::models::note::NoteTreeNode;
use std::fs;
use std::path::Path;

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
}
