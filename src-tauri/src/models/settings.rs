use crate::models::quiz::QuestionType;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

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

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AdvancedConfig {
    #[serde(default = "default_max_diagnosis_rounds")]
    pub max_diagnosis_rounds: u32,
    #[serde(default = "default_require_reasoning")]
    pub require_reasoning: bool,
    #[serde(default = "default_show_diagnosis_report")]
    pub show_diagnosis_report: bool,
}

fn default_max_diagnosis_rounds() -> u32 {
    3
}

fn default_require_reasoning() -> bool {
    true
}

fn default_show_diagnosis_report() -> bool {
    true
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuizDefaults {
    pub default_types: Vec<QuestionType>,
    #[serde(default = "default_language")]
    pub default_language: String,
    pub default_count: u32,
    #[serde(default = "default_mode")]
    pub default_mode: String,
    #[serde(default = "default_difficulty")]
    pub default_difficulty: String,
    #[serde(default = "default_prompt_template")]
    pub prompt_template: String,
    #[serde(default)]
    pub advanced: AdvancedConfig,
}

fn default_language() -> String {
    "zh".to_string()
}

fn default_mode() -> String {
    "basic".to_string()
}

fn default_difficulty() -> String {
    "medium".to_string()
}

fn default_prompt_template() -> String {
    "default".to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct WorkspaceState {
    #[serde(default)]
    pub root_path: Option<String>,
    #[serde(default)]
    pub expanded_dirs: Vec<String>,
    #[serde(default)]
    pub selected_path: Option<String>,
    #[serde(default)]
    pub scroll_positions: HashMap<String, u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Settings {
    pub version: String,
    pub theme: String,
    #[serde(default = "default_ui_language")]
    pub ui_language: String,
    pub llm: LlmConfig,
    pub ui: UiLayoutContainer,
    pub quiz: QuizDefaults,
    #[serde(default)]
    pub workspace: WorkspaceState,
}

fn default_ui_language() -> String {
    "zh-CN".to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiLayoutContainer {
    pub layout: UiLayout,
}
