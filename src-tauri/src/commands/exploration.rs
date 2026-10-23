use crate::errors::AppError;
use crate::models::exploration::{
    CreateExplorationSessionRequest, ExplorationSession, ExplorationSessionSummary,
};
use crate::services::exploration::session_store;
use crate::services::storage;
use tauri::AppHandle;

#[tauri::command]
pub async fn exploration_create(
    app: AppHandle,
    request: CreateExplorationSessionRequest,
) -> Result<ExplorationSession, AppError> {
    let data_dir = storage::get_data_dir(&app)?;
    tokio::task::spawn_blocking(move || {
        session_store::create_session(&data_dir, request.source_paths, request.title)
    })
    .await
    .map_err(|e| AppError::Internal(format!("Exploration create task failed: {e}")))?
}

#[tauri::command]
pub async fn exploration_list(app: AppHandle) -> Result<Vec<ExplorationSessionSummary>, AppError> {
    let data_dir = storage::get_data_dir(&app)?;
    tokio::task::spawn_blocking(move || session_store::list_sessions(&data_dir))
        .await
        .map_err(|e| AppError::Internal(format!("Exploration list task failed: {e}")))?
}

#[tauri::command]
pub async fn exploration_get(
    app: AppHandle,
    session_id: String,
) -> Result<ExplorationSession, AppError> {
    let data_dir = storage::get_data_dir(&app)?;
    tokio::task::spawn_blocking(move || session_store::load_session(&data_dir, &session_id))
        .await
        .map_err(|e| AppError::Internal(format!("Exploration load task failed: {e}")))?
}

#[tauri::command]
pub async fn exploration_delete(app: AppHandle, session_id: String) -> Result<bool, AppError> {
    let data_dir = storage::get_data_dir(&app)?;
    tokio::task::spawn_blocking(move || session_store::delete_session(&data_dir, &session_id))
        .await
        .map_err(|e| AppError::Internal(format!("Exploration delete task failed: {e}")))??;
    Ok(true)
}
