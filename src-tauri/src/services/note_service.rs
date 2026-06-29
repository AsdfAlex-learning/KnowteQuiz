use crate::models::note::NoteContent;

pub fn extract_metadata(content: &str, path: &str) -> NoteContent {
    let metadata = extract_frontmatter(content);
    let title = extract_title(content, path, &metadata);
    NoteContent {
        path: path.to_string(),
        title,
        content: extract_body_content(content),
        metadata,
    }
}

fn extract_title(
    content: &str,
    path: &str,
    metadata: &std::collections::HashMap<String, String>,
) -> String {
    for line in content.lines() {
        let trimmed = line.trim();
        if let Some(stripped) = trimmed.strip_prefix("# ") {
            return stripped.trim().to_string();
        }
    }
    if let Some(title) = metadata.get("title").filter(|title| !title.trim().is_empty()) {
        return title.trim().to_string();
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
                    metadata.insert(key.trim().to_string(), normalize_frontmatter_value(value));
                }
            }
        }
    }
    metadata
}

pub fn extract_body_content(content: &str) -> String {
    let Some(stripped) = content.strip_prefix("---") else {
        return content.to_string();
    };
    let Some(pos) = stripped.find("---") else {
        return content.to_string();
    };
    stripped[pos + 3..].trim_start().to_string()
}

fn normalize_frontmatter_value(value: &str) -> String {
    let value = value.trim();
    if value.len() >= 2 {
        let mut chars = value.chars();
        let first = chars.next();
        let last = chars.next_back();
        if matches!((first, last), (Some('"'), Some('"')) | (Some('\''), Some('\''))) {
            return chars.as_str().to_string();
        }
    }
    value.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extract_metadata_uses_frontmatter_title_when_note_has_no_heading() {
        let note = extract_metadata(
            "---\ntitle: Rust Ownership\nsource: book\n---\n\nOwnership notes.",
            "/notes/ownership.md",
        );

        assert_eq!(note.title, "Rust Ownership");
        assert_eq!(
            note.metadata.get("source").map(String::as_str),
            Some("book")
        );
    }

    #[test]
    fn extract_metadata_unquotes_frontmatter_scalar_values() {
        let note = extract_metadata(
            "---\ntitle: \"Rust Ownership\"\nsource: 'book'\n---\n\nOwnership notes.",
            "/notes/ownership.md",
        );

        assert_eq!(note.title, "Rust Ownership");
        assert_eq!(
            note.metadata.get("source").map(String::as_str),
            Some("book")
        );
    }

    #[test]
    fn extract_metadata_returns_body_without_frontmatter() {
        let note = extract_metadata(
            "---\ntitle: Rust Ownership\ntags: rust\n---\n\n# Ownership\n\nMoves transfer values.",
            "/notes/ownership.md",
        );

        assert_eq!(note.content, "# Ownership\n\nMoves transfer values.");
        assert!(!note.content.contains("tags: rust"));
    }

    #[test]
    fn extract_metadata_prefers_heading_over_frontmatter_title() {
        let note = extract_metadata(
            "---\ntitle: Frontmatter Title\n---\n\n# Heading Title\n\nBody.",
            "/notes/ownership.md",
        );

        assert_eq!(note.title, "Heading Title");
    }
}
