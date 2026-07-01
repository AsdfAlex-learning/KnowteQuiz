use crate::models::note::{NoteContent, NoteTreeNode};
use crate::services::fs_service;
use crate::services::note_service;
use crate::services::storage;
use tauri::AppHandle;

#[tauri::command]
pub async fn select_folder(app: AppHandle) -> Result<Option<String>, String> {
    use tauri_plugin_dialog::DialogExt;
    let folder = app.dialog().file().blocking_pick_folder();
    Ok(folder.map(|p| p.to_string()))
}

#[tauri::command]
pub async fn scan_notes(app: AppHandle, root_path: String) -> Result<Vec<NoteTreeNode>, String> {
    let data_dir = storage::get_data_dir(&app)?;
    fs_service::scan_directory_with_index(&root_path, &data_dir)
}

#[tauri::command]
pub async fn read_note(path: String) -> Result<NoteContent, String> {
    let content = fs_service::read_file_content(&path)?;
    Ok(note_service::extract_metadata(&content, &path))
}
