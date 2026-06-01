use serde::{Deserialize, Serialize};
use crate::models::quiz::QuestionType;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmConfig {
    pub provider: String,
    pub base_url: String,
    pub api_key: String,
    pub model: String,
    pub max_tokens: u32,
    pub temperature: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiLayout {
    pub left_visible: bool,
    pub right_visible: bool,
    pub left_width: u32,
    pub right_width: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdvancedConfig {
    pub max_diagnosis_rounds: u32,
    pub require_reasoning: bool,
    pub show_diagnosis_report: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuizDefaults {
    pub default_types: Vec<QuestionType>,
    pub default_language: String,
    pub default_count: u32,
    pub default_mode: String,
    #[serde(default = "default_prompt_template")]
    pub prompt_template: String,
    pub advanced: AdvancedConfig,
}

fn default_prompt_template() -> String {
    "default".to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Settings {
    pub version: String,
    pub theme: String,
    pub llm: LlmConfig,
    pub ui: UiLayoutContainer,
    pub quiz: QuizDefaults,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiLayoutContainer {
    pub layout: UiLayout,
}
