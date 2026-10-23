use crate::errors::AppError;
use crate::models::exploration::{
    AskExplorationRequest, ConfirmExplorationDomainsRequest, CreateExplorationSessionRequest,
    ExplorationSession, ExplorationSessionSummary,
};
use crate::services::exploration::engine::{self, ExplorationStreamEvent};
use crate::services::exploration::session_store;
use crate::services::storage;
use tauri::ipc::Channel;
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

#[tauri::command]
pub async fn exploration_start_stream(
    app: AppHandle,
    session_id: String,
    on_event: Channel<ExplorationStreamEvent>,
) -> Result<(), AppError> {
    let data_dir = storage::get_data_dir(&app)?;
    let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel();
    tokio::spawn(async move {
        while let Some(event) = rx.recv().await {
            if on_event.send(event).is_err() {
                // Tauri channel closed: stop forwarding.
                break;
            }
        }
    });
    engine::initialize_session(&data_dir, &session_id, tx).await
}

#[tauri::command]
pub async fn exploration_confirm_domains(
    app: AppHandle,
    session_id: String,
    request: ConfirmExplorationDomainsRequest,
    on_event: Channel<ExplorationStreamEvent>,
) -> Result<(), AppError> {
    let data_dir = storage::get_data_dir(&app)?;
    let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel();
    tokio::spawn(async move {
        while let Some(event) = rx.recv().await {
            if on_event.send(event).is_err() {
                break;
            }
        }
    });
    engine::confirm_domains(&data_dir, &session_id, request.domains, tx).await
}

#[tauri::command]
pub async fn exploration_next_question(
    app: AppHandle,
    session_id: String,
) -> Result<ExplorationSession, AppError> {
    let data_dir = storage::get_data_dir(&app)?;
    engine::next_question(&data_dir, &session_id).await
}

#[tauri::command]
pub async fn exploration_ask(
    app: AppHandle,
    session_id: String,
    request: AskExplorationRequest,
) -> Result<ExplorationSession, AppError> {
    let data_dir = storage::get_data_dir(&app)?;
    engine::ask_turn(&data_dir, &session_id, &request.answer).await
}
