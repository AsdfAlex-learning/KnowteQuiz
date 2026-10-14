use serde::{Deserialize, Serialize};

/// Input formats the document converter accepts. Keep this list in sync with
/// `fs_service::DOCUMENT_EXTENSIONS` (what the tree shows) and the `.images`
/// derived-directory convention (`design.docx.images/`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DocumentFormat {
    Docx,
    Pptx,
    Pdf,
    Html,
}

impl DocumentFormat {
    pub fn from_extension(ext: &str) -> Option<Self> {
        match ext.to_ascii_lowercase().as_str() {
            "docx" => Some(DocumentFormat::Docx),
            "pptx" => Some(DocumentFormat::Pptx),
            "pdf" => Some(DocumentFormat::Pdf),
            "html" | "htm" => Some(DocumentFormat::Html),
            _ => None,
        }
    }
}

/// An image extracted from a document, ready to be written next to the
/// converted note. `ref_name` is the bare filename used in the Markdown.
#[derive(Debug, Clone)]
pub struct ParsedImage {
    pub ref_name: String,
    pub bytes: Vec<u8>,
}

/// What a format parser produced. Images are *not* written to disk by the
/// parser; the converter decides where they go.
#[derive(Debug, Clone, Default)]
pub struct ParsedDocument {
    /// Best-effort document title (first heading / slide title).
    pub title: Option<String>,
    pub markdown: String,
    pub images: Vec<ParsedImage>,
    /// Non-fatal problems (skipped element, encoding fallback, …). Surfaced to
    /// the user so partial output is honest rather than silently wrong.
    pub warnings: Vec<String>,
}

/// Outcome of converting a document into a note.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConvertOutcome {
    pub source_path: String,
    pub output_path: String,
    /// `Some` when at least one image was written to the derived dir.
    pub image_dir: Option<String>,
    pub title: Option<String>,
    pub image_count: u32,
    pub warnings: Vec<String>,
}

/// Current conversion state of a document, used for the "converted"/"stale"
/// badge in the file tree.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentStatus {
    pub source_path: String,
    pub converted: bool,
    /// Whether the source was modified after its converted note.
    pub stale: bool,
    pub output_path: Option<String>,
}
