use crate::errors::AppError;
use crate::models::mistake::{MistakeEntry, MistakeFilter};
use crate::models::settings::Settings;
use crate::services::llm_service::ConnectionTestResult;
use crate::services::llm_service::LlmCapabilities;
use crate::services::storage::{DataBackupResult, DataRestoreResult, DataStatus};
use crate::services::{config, llm_service, media_service, mistake_service, storage};
use tauri::AppHandle;

#[tauri::command]
pub async fn get_settings(app: AppHandle) -> Result<Settings, AppError> {
    config::get_settings(&app)
}

#[tauri::command]
pub async fn save_settings(app: AppHandle, settings: Settings) -> Result<bool, AppError> {
    config::save_settings(&app, &settings)?;
    Ok(true)
}

#[tauri::command]
pub async fn list_prompt_templates() -> Result<Vec<(String, String, String)>, AppError> {
    Ok(crate::utils::prompt_templates::list_template_sets())
}

#[tauri::command]
pub async fn test_connection(app: AppHandle) -> Result<ConnectionTestResult, AppError> {
    let data_dir = storage::get_data_dir(&app)?;
    let settings = config::get_settings_path(&data_dir)?;
    llm_service::test_connection(&settings.llm).await
}

#[tauri::command]
pub async fn probe_llm(app: AppHandle) -> Result<LlmCapabilities, AppError> {
    let data_dir = storage::get_data_dir(&app)?;
    let settings = config::get_settings_path(&data_dir)?;
    llm_service::probe_capabilities(&settings.llm).await
}

#[tauri::command]
pub async fn backup_data(app: AppHandle) -> Result<DataBackupResult, AppError> {
    storage::backup_data_files(&app)
}

#[tauri::command]
pub async fn get_data_status(app: AppHandle) -> Result<DataStatus, AppError> {
    storage::data_status(&app)
}

#[tauri::command]
pub async fn restore_latest_backup(app: AppHandle) -> Result<DataRestoreResult, AppError> {
    storage::restore_latest_backup(&app)
}

#[tauri::command]
pub async fn save_mistake(app: AppHandle, entry: MistakeEntry) -> Result<bool, AppError> {
    let data_dir = storage::get_data_dir(&app)?;
    mistake_service::save_mistake(&data_dir, entry)?;
    Ok(true)
}

#[tauri::command]
pub async fn load_mistakes(
    app: AppHandle,
    filter: Option<MistakeFilter>,
) -> Result<Vec<MistakeEntry>, AppError> {
    let data_dir = storage::get_data_dir(&app)?;
    let filter = filter.unwrap_or_default();
    mistake_service::load_mistakes(&data_dir, &filter)
}

#[tauri::command]
pub async fn mark_mistake_reviewed(app: AppHandle, mistake_id: String, quality: u32) -> Result<bool, AppError> {
    let data_dir = storage::get_data_dir(&app)?;
    mistake_service::mark_mistake_reviewed(&data_dir, &mistake_id, quality)?;
    Ok(true)
}

#[tauri::command]
pub async fn open_data_dir(app: AppHandle) -> Result<String, AppError> {
    let data_dir = storage::get_data_dir(&app)?;
    let path = data_dir.to_string_lossy().to_string();

    #[cfg(target_os = "windows")]
    {
        std::process::Command::new("explorer.exe")
            .arg(&path)
            .spawn()
            .map_err(|e| AppError::Internal(format!("Failed to open data directory: {}", e)))?;
    }

    #[cfg(target_os = "macos")]
    {
        std::process::Command::new("open")
            .arg(&path)
            .spawn()
            .map_err(|e| AppError::Internal(format!("Failed to open data directory: {}", e)))?;
    }

    #[cfg(target_os = "linux")]
    {
        std::process::Command::new("xdg-open")
            .arg(&path)
            .spawn()
            .map_err(|e| AppError::Internal(format!("Failed to open data directory: {}", e)))?;
    }

    Ok(path)
}

#[tauri::command]
pub async fn save_media_file(
    app: AppHandle,
    request: tauri::ipc::Request<'_>,
) -> Result<media_service::MediaSaved, AppError> {
    let data_dir = storage::get_data_dir(&app)?;

    // The frontend sends the raw bytes as the invoke payload; kind and the
    // original file name (for extension validation) travel in headers.
    let kind = header_value(request.headers(), "x-kq-kind");
    let file_name = header_value(request.headers(), "x-kq-file-name");
    let bytes = match request.body() {
        tauri::ipc::InvokeBody::Raw(bytes) => bytes.to_vec(),
        tauri::ipc::InvokeBody::Json(_) => {
            return Err(AppError::InvalidInput(
                "Media upload must be sent as raw bytes".to_string(),
            ))
        }
    };

    tokio::task::spawn_blocking(move || media_service::save_media_file(&data_dir, &kind, &file_name, &bytes))
        .await
        .map_err(|e| AppError::Internal(format!("Media save task failed: {e}")))?
}

fn header_value(headers: &tauri::http::HeaderMap, name: &str) -> String {
    headers
        .get(name)
        .and_then(|value| value.to_str().ok())
        .unwrap_or_default()
        .to_string()
}
