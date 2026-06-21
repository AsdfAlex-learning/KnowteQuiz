use crate::models::diagnosis::{DiagnosisReport, DiagnosisRound};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MistakeMode {
    Basic,
    Advanced,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiagnosisContext {
    pub rounds: u32,
    pub conversation: Vec<DiagnosisRound>,
    pub final_report: DiagnosisReport,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MistakeEntry {
    pub id: String,
    pub note_path: String,
    pub note_title: String,
    pub question: String,
    pub user_answer: String,
    pub correct_answer: String,
    pub explanation: String,
    pub mode: MistakeMode,
    #[serde(default)]
    pub user_reasoning: Option<String>,
    #[serde(default)]
    pub diagnosis: Option<DiagnosisContext>,
    pub created_at: String,
    #[serde(default)]
    pub review_count: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct MistakeFilter {
    pub mode: Option<MistakeMode>,
    pub note_path: Option<String>,
    #[serde(default)]
    pub offset: Option<u32>,
    pub limit: Option<u32>,
}
