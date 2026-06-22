use crate::models::note::NoteContent;

pub fn extract_metadata(content: &str, path: &str) -> NoteContent {
    let title = extract_title(content, path);
    let metadata = extract_frontmatter(content);
    NoteContent {
        path: path.to_string(),
        title,
        content: content.to_string(),
        metadata,
    }
}

fn extract_title(content: &str, path: &str) -> String {
    for line in content.lines() {
        let trimmed = line.trim();
        if let Some(stripped) = trimmed.strip_prefix("# ") {
            return stripped.trim().to_string();
        }
    }
    std::path::Path::new(path)
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("Untitled")
        .to_string()
}

fn extract_frontmatter(content: &str) -> std::collections::HashMap<String, String> {
    let mut metadata = std::collections::HashMap::new();
    if let Some(stripped) = content.strip_prefix("---") {
        let end = stripped.find("---");
        if let Some(pos) = end {
            let frontmatter = &content[3..pos + 3];
            for line in frontmatter.lines() {
                if let Some((key, value)) = line.split_once(':') {
                    metadata.insert(key.trim().to_string(), value.trim().to_string());
                }
            }
        }
    }
    metadata
}
