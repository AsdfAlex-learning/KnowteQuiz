use crate::models::note::{NoteTreeNode, NoteContent};
use crate::services::fs_service;
use crate::services::note_service;
use tauri::AppHandle;

#[tauri::command]
pub async fn select_folder(app: AppHandle) -> Result<Option<String>, String> {
    use tauri_plugin_dialog::DialogExt;
    let folder = app.dialog().file().blocking_pick_folder();
    Ok(folder.map(|p| p.to_string()))
}

#[tauri::command]
pub async fn scan_notes(root_path: String) -> Result<Vec<NoteTreeNode>, String> {
    fs_service::scan_directory(&root_path)
}

#[tauri::command]
pub async fn read_note(path: String) -> Result<NoteContent, String> {
    let content = fs_service::read_file_content(&path)?;
    Ok(note_service::extract_metadata(&content, &path))
}
