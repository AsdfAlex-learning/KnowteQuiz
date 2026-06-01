use crate::models::settings::*;
use crate::services::storage;
use tauri::AppHandle;
use crate::models::quiz::QuestionType;
use std::path::Path;

fn default_settings() -> Settings {
    Settings {
        version: "1.0.0".to_string(),
        theme: "obsidian-dark".to_string(),
        llm: LlmConfig {
            provider: "openai-compatible".to_string(),
            base_url: "http://localhost:11434/v1".to_string(),
            api_key: String::new(),
            model: "qwen2.5:7b".to_string(),
            max_tokens: 4096,
            temperature: 0.7,
        },
        ui: UiLayoutContainer {
            layout: UiLayout {
                left_visible: true,
                right_visible: true,
                left_width: 280,
                right_width: 360,
            },
        },
        quiz: QuizDefaults {
            default_types: vec![QuestionType::Single, QuestionType::Multiple, QuestionType::Short],
            default_language: "zh".to_string(),
            default_count: 5,
            default_mode: "basic".to_string(),
            prompt_template: "default".to_string(),
            advanced: AdvancedConfig {
                max_diagnosis_rounds: 3,
                require_reasoning: true,
                show_diagnosis_report: true,
            },
        },
    }
}

pub fn get_settings_path(data_dir: &Path) -> Result<Settings, String> {
    match storage::read_json_path::<Settings>(data_dir, "settings.json") {
        Ok(settings) => Ok(settings),
        Err(_) => {
            let defaults = default_settings();
            storage::write_json_path(data_dir, "settings.json", &defaults)?;
            Ok(defaults)
        }
    }
}

pub fn save_settings_path(data_dir: &Path, settings: &Settings) -> Result<(), String> {
    storage::write_json_path(data_dir, "settings.json", settings)
}

// Backward-compatible wrappers for Tauri commands
pub fn get_settings(app: &AppHandle) -> Result<Settings, String> {
    let dir = crate::services::storage::get_data_dir(app)?;
    get_settings_path(&dir)
}

pub fn save_settings(app: &AppHandle, settings: &Settings) -> Result<(), String> {
    let dir = crate::services::storage::get_data_dir(app)?;
    save_settings_path(&dir, settings)
}
