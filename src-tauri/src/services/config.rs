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
            default_types: vec![QuestionType::Single, QuestionType::Short],
            default_language: "zh".to_string(),
            default_count: 5,
            default_mode: "basic".to_string(),
            default_difficulty: "medium".to_string(),
            prompt_template: "default".to_string(),
            advanced: AdvancedConfig {
                max_diagnosis_rounds: 3,
                require_reasoning: true,
                show_diagnosis_report: true,
            },
        },
        workspace: WorkspaceState::default(),
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

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_data_dir(test_name: &str) -> std::path::PathBuf {
        let dir = std::env::temp_dir()
            .join("knowtequiz-config-tests")
            .join(test_name)
            .join(uuid::Uuid::new_v4().to_string());
        std::fs::create_dir_all(&dir).expect("test temp dir should be created");
        dir
    }

    #[test]
    fn get_settings_path_preserves_old_settings_and_defaults_missing_workspace() {
        let dir = temp_data_dir("get_settings_path_preserves_old_settings_and_defaults_missing_workspace");
        let old_settings = serde_json::json!({
            "version": "1.0.0",
            "theme": "obsidian-dark",
            "llm": {
                "provider": "openai-compatible",
                "base_url": "http://localhost:11434/v1",
                "api_key": "",
                "model": "custom-model",
                "max_tokens": 4096,
                "temperature": 0.7
            },
            "ui": {
                "layout": {
                    "left_visible": true,
                    "right_visible": true,
                    "left_width": 280,
                    "right_width": 360
                }
            },
            "quiz": {
                "default_types": ["single", "short"],
                "default_language": "zh",
                "default_count": 5,
                "default_mode": "basic",
                "default_difficulty": "medium",
                "prompt_template": "default",
                "advanced": {
                    "max_diagnosis_rounds": 3,
                    "require_reasoning": true,
                    "show_diagnosis_report": true
                }
            }
        });
        std::fs::write(
            dir.join("settings.json"),
            serde_json::to_string_pretty(&old_settings).expect("settings json should serialize"),
        )
        .expect("old settings should be written");

        let settings = get_settings_path(&dir).expect("old settings should migrate");

        assert_eq!(settings.llm.model, "custom-model");
        assert_eq!(settings.workspace.root_path, None);
        assert!(settings.workspace.expanded_dirs.is_empty());
        assert_eq!(settings.workspace.selected_path, None);
        assert!(settings.workspace.scroll_positions.is_empty());
    }
}
