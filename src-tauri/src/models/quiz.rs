use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum QuestionType {
    Single,
    Multiple,
    Short,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuizQuestion {
    pub id: String,
    pub question_type: QuestionType,
    pub question: String,
    pub options: Vec<String>,
    pub answer: String,
    pub explanation: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuizStreamParams {
    pub path: String,
    pub types: Vec<QuestionType>,
    pub count: u32,
    pub difficulty: String,
    pub lang: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuizResult {
    pub total: u32,
    pub questions: Vec<QuizQuestion>,
}
