use crate::models::mistake::{MistakeEntry, MistakeFilter};
use crate::models::settings::Settings;
use crate::services::llm_service::ConnectionTestResult;
use crate::services::storage::{DataBackupResult, DataRestoreResult, DataStatus};
use crate::services::{config, llm_service, mistake_service, storage};
use tauri::AppHandle;

#[tauri::command]
pub async fn get_settings(app: AppHandle) -> Result<Settings, String> {
    config::get_settings(&app)
}

#[tauri::command]
pub async fn save_settings(app: AppHandle, settings: Settings) -> Result<bool, String> {
    config::save_settings(&app, &settings)?;
    Ok(true)
}

#[tauri::command]
pub async fn list_prompt_templates() -> Result<Vec<(String, String, String)>, String> {
    Ok(crate::utils::prompt_templates::list_template_sets())
}

#[tauri::command]
pub async fn test_connection(app: AppHandle) -> Result<ConnectionTestResult, String> {
    let data_dir = storage::get_data_dir(&app)?;
    let settings = config::get_settings_path(&data_dir)?;
    Ok(llm_service::test_connection(&settings.llm).await)
}

#[tauri::command]
pub async fn backup_data(app: AppHandle) -> Result<DataBackupResult, String> {
    storage::backup_data_files(&app)
}

#[tauri::command]
pub async fn get_data_status(app: AppHandle) -> Result<DataStatus, String> {
    storage::data_status(&app)
}

#[tauri::command]
pub async fn restore_latest_backup(app: AppHandle) -> Result<DataRestoreResult, String> {
    storage::restore_latest_backup(&app)
}

#[tauri::command]
pub async fn save_mistake(app: AppHandle, entry: MistakeEntry) -> Result<bool, String> {
    let data_dir = storage::get_data_dir(&app)?;
    mistake_service::save_mistake(&data_dir, entry)?;
    Ok(true)
}

#[tauri::command]
pub async fn load_mistakes(
    app: AppHandle,
    filter: Option<MistakeFilter>,
) -> Result<Vec<MistakeEntry>, String> {
    let data_dir = storage::get_data_dir(&app)?;
    let filter = filter.unwrap_or_default();
    mistake_service::load_mistakes(&data_dir, &filter)
}
