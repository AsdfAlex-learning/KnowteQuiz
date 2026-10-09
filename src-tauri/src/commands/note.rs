use crate::errors::AppError;
use crate::models::note::{NoteContent, NoteTreeNode};
use crate::services::fs_service;
use crate::services::note_service;
use crate::services::storage;
use tauri::AppHandle;

#[tauri::command]
pub async fn select_folder(app: AppHandle) -> Result<Option<String>, AppError> {
    use tauri_plugin_dialog::DialogExt;
    // The native dialog parks its thread until the user picks; keep it off
    // the async runtime workers.
    let folder = tokio::task::spawn_blocking(move || app.dialog().file().blocking_pick_folder())
        .await
        .map_err(|e| AppError::Internal(format!("Folder picker task failed: {e}")))?;
    Ok(folder.map(|p| p.to_string()))
}

#[tauri::command]
pub async fn scan_notes(app: AppHandle, root_path: String) -> Result<Vec<NoteTreeNode>, AppError> {
    let data_dir = storage::get_data_dir(&app)?;
    fs_service::scan_directory_with_index(&root_path, &data_dir).await
}

#[tauri::command]
pub async fn read_note(app: AppHandle, path: String) -> Result<NoteContent, AppError> {
    let data_dir = storage::get_data_dir(&app)?;
    let resolved = fs_service::resolve_note_path(&data_dir, &path).await?;
    let content = fs_service::read_file_content(&resolved.to_string_lossy()).await?;
    Ok(note_service::extract_metadata(&content, &path))
}
