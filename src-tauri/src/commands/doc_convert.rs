use crate::errors::AppError;
use crate::models::doc_parse::{ConvertOutcome, DocumentStatus};
use crate::services::doc_parser;
use crate::services::storage;
use tauri::AppHandle;

/// Convert a PDF/DOCX/PPTX/HTML document into a Markdown note next to its
/// source. The produced note and its `.images/` directory are plain files in
/// the user's vault, so the existing scan/read pipeline picks them up.
#[tauri::command]
pub async fn convert_document(
    app: AppHandle,
    source_path: String,
) -> Result<ConvertOutcome, AppError> {
    // Validate the source is inside the configured note root before touching it.
    let data_dir = storage::get_data_dir(&app)?;
    let resolved = crate::services::fs_service::resolve_note_path(&data_dir, &source_path).await?;
    doc_parser::convert_document(&resolved.to_string_lossy()).await
}

/// Whether `source_path` has a converted note, and whether that note is older
/// than its source. Drives the tree badge.
#[tauri::command]
pub async fn get_document_status(
    app: AppHandle,
    source_path: String,
) -> Result<DocumentStatus, AppError> {
    let data_dir = storage::get_data_dir(&app)?;
    let resolved = crate::services::fs_service::resolve_note_path(&data_dir, &source_path).await?;
    Ok(doc_parser::document_status(&resolved))
}
