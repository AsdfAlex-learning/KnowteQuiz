use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NoteIndex {
    pub version: String,
    pub root_path: String,
    pub generated_at: chrono::DateTime<chrono::Utc>,
    pub notes: Vec<NoteIndexEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NoteIndexEntry {
    pub path: String,
    pub title: String,
    pub size_bytes: u64,
    pub modified_at: Option<chrono::DateTime<chrono::Utc>>,
}

/// What kind of file a tree node represents. Markdown notes are readable and
/// quizzable; documents (PDF/DOCX/PPTX/HTML) are conversion candidates only.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, Default, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum NoteNodeKind {
    #[default]
    Markdown,
    Document,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NoteTreeNode {
    pub name: String,
    pub path: String,
    #[serde(default)]
    pub kind: NoteNodeKind,
    pub is_dir: bool,
    pub children: Vec<NoteTreeNode>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NoteContent {
    pub path: String,
    pub title: String,
    pub content: String,
    pub metadata: std::collections::HashMap<String, String>,
}

/// One selectable directory in the in-app folder picker.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DirectoryEntry {
    pub name: String,
    pub path: String,
}

/// A directory listing plus the platform roots (drives / home / volumes) the
/// picker can jump to.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DirectoryListing {
    pub path: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub parent: Option<String>,
    pub directories: Vec<DirectoryEntry>,
    pub roots: Vec<DirectoryEntry>,
}
