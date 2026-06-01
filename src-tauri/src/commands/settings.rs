use crate::models::settings::Settings;
use crate::models::mistake::MistakeEntry;
use crate::services::{config, storage};
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
pub async fn test_connection(app: AppHandle) -> Result<bool, String> {
    let data_dir = storage::get_data_dir(&app)?;
    let settings = config::get_settings_path(&data_dir)?;
    let llm = &settings.llm;

    let client = reqwest::Client::new();
    let request_body = serde_json::json!({
        "model": llm.model,
        "messages": [
            { "role": "user", "content": "hi" }
        ],
        "max_tokens": 5,
    });

    let response = client
        .post(format!("{}/chat/completions", llm.base_url.trim_end_matches('/')))
        .header("Authorization", format!("Bearer {}", llm.api_key))
        .header("Content-Type", "application/json")
        .json(&request_body)
        .send()
        .await
        .map_err(|e| format!("Connection failed: {}", e))?;

    Ok(response.status().is_success())
}

#[tauri::command]
pub async fn save_mistake(app: AppHandle, entry: MistakeEntry) -> Result<bool, String> {
    let mut mistakes: Vec<MistakeEntry> = storage::read_json(&app, "mistakes.json")
        .unwrap_or_default();
    mistakes.insert(0, entry);
    storage::write_json(&app, "mistakes.json", &mistakes)?;
    Ok(true)
}

#[tauri::command]
pub async fn load_mistakes(app: AppHandle) -> Result<Vec<MistakeEntry>, String> {
    let mistakes: Vec<MistakeEntry> = storage::read_json(&app, "mistakes.json")
        .unwrap_or_default();
    Ok(mistakes)
}
