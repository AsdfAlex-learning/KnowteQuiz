use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlindSpot {
    pub tag: String,
    pub severity: String,
    pub description: String,
    pub note_reference: String,
    pub suggestion: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiagnosisRound {
    pub role: String,
    pub content: String,
    #[serde(default)]
    pub blind_spots: Vec<BlindSpot>,
    #[serde(default)]
    pub follow_up: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiagnosisReport {
    pub summary: String,
    pub blind_spots: Vec<BlindSpot>,
    pub overall_level: String,
    pub next_steps: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiagnosisSession {
    pub session_id: String,
    pub question: String,
    pub user_answer: String,
    pub user_reasoning: String,
    pub note_path: String,
    pub note_content: String,
    pub conversation: Vec<DiagnosisRound>,
    pub current_round: u32,
    pub max_rounds: u32,
    pub final_report: Option<DiagnosisReport>,
}
