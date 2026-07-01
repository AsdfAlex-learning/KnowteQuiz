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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NoteTreeNode {
    pub name: String,
    pub path: String,
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
