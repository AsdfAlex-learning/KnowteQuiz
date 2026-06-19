use crate::models::diagnosis::*;
use crate::models::quiz::*;
use crate::models::settings::LlmConfig;
use futures_util::StreamExt;
use reqwest::Client;
use serde_json::Value;
use std::collections::HashMap;
use std::path::Path;
use tokio::sync::mpsc::UnboundedSender;

#[derive(Clone, serde::Serialize)]
#[serde(rename_all = "camelCase", tag = "event", content = "data")]
pub enum QuizStreamEvent {
    Chunk { id: String, question_type: String, question: String, options: Vec<String>, answer: String, explanation: String },
    Done { total: u32 },
    Error { message: String },
}

#[derive(Clone, serde::Serialize)]
#[serde(rename_all = "camelCase", tag = "event", content = "data")]
pub enum DiagnosisStreamEvent {
    Initial { role: String, content: String, blind_spots: Vec<BlindSpot>, follow_up: Option<String> },
    FollowUp { question: String, blind_spots: Vec<BlindSpot> },
    Report { summary: String, blind_spots: Vec<BlindSpot>, overall_level: String, next_steps: Vec<String> },
    Error { message: String },
}

pub async fn generate_quiz_stream(
    data_dir: &Path,
    params: &QuizStreamParams,
    tx: UnboundedSender<QuizStreamEvent>,
) -> Result<(), String> {
    let note_content = crate::services::fs_service::read_file_content(&params.path)?;
    let truncated_content: String = note_content.chars().take(8000).collect();
    let settings = crate::services::config::get_settings_path(data_dir)?;
    let llm = &settings.llm;

    let question_types_str = params.types.iter()
        .map(|qt| format!("{:?}", qt).to_lowercase())
        .collect::<Vec<_>>()
        .join(", ");
    let mut vars = HashMap::new();
    vars.insert("note_content", truncated_content);
    vars.insert("question_types", question_types_str);
    vars.insert("count", params.count.to_string());
    vars.insert("difficulty", params.difficulty.clone());
    vars.insert("language", params.lang.clone());
    let template_set = crate::utils::prompt_templates::get_template_set(&settings.quiz.prompt_template)
        .unwrap_or_else(crate::utils::prompt_templates::get_default_template_set);
    let prompt = crate::utils::prompt_templates::fill_template(
        &template_set.quiz_template, &vars
    );

    let client = Client::new();
    let request_body = serde_json::json!({
        "model": llm.model,
        "messages": [
            { "role": "system", "content": "你是一个知识测验生成助手。请严格按照指定的 JSON 格式返回。" },
            { "role": "user", "content": prompt }
        ],
        "stream": true,
        "temperature": llm.temperature,
        "max_tokens": llm.max_tokens,
    });

    let response = client
        .post(format!("{}/chat/completions", llm.base_url.trim_end_matches('/')))
        .header("Authorization", format!("Bearer {}", llm.api_key))
        .header("Content-Type", "application/json")
        .json(&request_body)
        .send()
        .await
        .map_err(|e| format!("LLM API request failed: {}", e))?;

    if !response.status().is_success() {
        let status = response.status();
        let body = response.text().await.unwrap_or_default();
        let _ = tx.send(QuizStreamEvent::Error {
            message: format!("LLM API error {}: {}", status, body),
        });
        return Err(format!("LLM API error {}: {}", status, body));
    }

    let mut stream = response.bytes_stream();
    let mut accumulated = String::new();

    while let Some(chunk_result) = stream.next().await {
        match chunk_result {
            Ok(bytes) => {
                if let Ok(text) = String::from_utf8(bytes.to_vec()) {
                    for line in text.lines() {
                        if line.starts_with("data: ") {
                            let data = &line[6..];
                            if data.trim() == "[DONE]" {
                                continue;
                            }
                            if let Ok(json) = serde_json::from_str::<Value>(data) {
                                if let Some(content) = json["choices"][0]["delta"]["content"].as_str() {
                                    accumulated.push_str(content);
                                }
                            }
                        }
                    }
                }
            }
            Err(e) => {
                let _ = tx.send(QuizStreamEvent::Error {
                    message: format!("Stream error: {}", e),
                });
            }
        }
    }

    match parse_quiz_response(&accumulated) {
        Ok(questions) => {
            for q in &questions {
                let _ = tx.send(QuizStreamEvent::Chunk {
                    id: q.id.clone(),
                    question_type: format!("{:?}", q.question_type).to_lowercase(),
                    question: q.question.clone(),
                    options: q.options.clone(),
                    answer: q.answer.clone(),
                    explanation: q.explanation.clone(),
                });
            }
            let _ = tx.send(QuizStreamEvent::Done { total: questions.len() as u32 });
        }
        Err(e) => {
            let _ = tx.send(QuizStreamEvent::Error {
                message: format!("Failed to parse quiz response: {}", e),
            });
        }
    }

    Ok(())
}

fn parse_quiz_response(raw: &str) -> Result<Vec<QuizQuestion>, String> {
    let json_str = extract_json_block(raw);
    let parsed: serde_json::Value = serde_json::from_str(&json_str)
        .map_err(|e| format!("JSON parse error: {}. Raw: {}", e, &raw[..raw.len().min(200)]))?;

    let questions = parsed["questions"]
        .as_array()
        .ok_or("Missing 'questions' array in response")?;

    let mut result = Vec::new();
    for (i, q) in questions.iter().enumerate() {
        let question = parse_quiz_question(q, i)?;
        result.push(question);
    }
    Ok(result)
}

fn parse_quiz_question(q: &Value, index: usize) -> Result<QuizQuestion, String> {
    let id = q["id"]
        .as_str()
        .filter(|value| !value.trim().is_empty())
        .map(String::from)
        .unwrap_or_else(|| format!("q_{}", index));
    let question_type = parse_question_type(
        q["question_type"]
            .as_str()
            .or_else(|| q["type"].as_str())
            .unwrap_or("single"),
        index,
    )?;
    let question = required_string(q, "question", index, "question text")?;
    let answer = required_string(q, "answer", index, "answer")?;
    let explanation = required_string(q, "explanation", index, "explanation")?;
    let options = q["options"]
        .as_array()
        .map(|arr| {
            arr.iter()
                .filter_map(|v| v.as_str())
                .map(str::trim)
                .filter(|v| !v.is_empty())
                .map(String::from)
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();

    if matches!(question_type, QuestionType::Single | QuestionType::Multiple) && options.is_empty() {
        return Err(format!("Question {} options are required for choice questions", index + 1));
    }

    Ok(QuizQuestion {
        id,
        question_type,
        question,
        options,
        answer,
        explanation,
    })
}

fn parse_question_type(raw: &str, index: usize) -> Result<QuestionType, String> {
    match raw.trim().to_ascii_lowercase().as_str() {
        "single" => Ok(QuestionType::Single),
        "multiple" => Ok(QuestionType::Multiple),
        "short" => Ok(QuestionType::Short),
        other => Err(format!("Question {} has unsupported question type: {}", index + 1, other)),
    }
}

fn required_string(
    value: &Value,
    field: &str,
    index: usize,
    label: &str,
) -> Result<String, String> {
    value[field]
        .as_str()
        .map(str::trim)
        .filter(|raw| !raw.is_empty())
        .map(String::from)
        .ok_or_else(|| format!("Question {} missing {}", index + 1, label))
}

fn extract_json_block(raw: &str) -> String {
    if let Some(start) = raw.find("```json") {
        if let Some(end) = raw[start..].find("```") {
            let json_start = start + 7;
            let json_end = start + end;
            return raw[json_start..json_end].trim().to_string();
        }
    }
    if let Some(start) = raw.find('{') {
        let mut depth = 0;
        for (i, c) in raw[start..].chars().enumerate() {
            if c == '{' { depth += 1; }
            else if c == '}' {
                depth -= 1;
                if depth == 0 {
                    return raw[start..start + i + 1].to_string();
                }
            }
        }
    }
    raw.to_string()
}

pub async fn submit_diagnosis_initial(
    data_dir: &Path,
    question: &str,
    user_answer: &str,
    user_reasoning: &str,
    note_path: &str,
    tx: UnboundedSender<DiagnosisStreamEvent>,
) -> Result<DiagnosisRound, String> {
    let note_content = crate::services::fs_service::read_file_content(note_path)?;
    let truncated_content: String = note_content.chars().take(8000).collect();

    let mut vars = HashMap::new();
    vars.insert("note_content", truncated_content);
    vars.insert("question", question.to_string());
    vars.insert("correct_answer", "(to be determined by LLM)".to_string());
    vars.insert("user_answer", user_answer.to_string());
    vars.insert("user_reasoning", user_reasoning.to_string());

    let settings = crate::services::config::get_settings_path(data_dir)?;
    let template_set = crate::utils::prompt_templates::get_template_set(&settings.quiz.prompt_template)
        .unwrap_or_else(crate::utils::prompt_templates::get_default_template_set);
    let prompt = crate::utils::prompt_templates::fill_template(
        &template_set.diagnosis_initial_template, &vars
    );
    let response_text = call_llm(&settings.llm, &prompt, 0.3).await?;
    let diagnosis = parse_diagnosis_initial(&response_text)?;
    let round = initial_diagnosis_round(&diagnosis);

    let _ = tx.send(DiagnosisStreamEvent::Initial {
        role: "ai".to_string(),
        content: diagnosis.answer_analysis.clone(),
        blind_spots: diagnosis.blind_spots.clone(),
        follow_up: diagnosis.follow_up_question.clone(),
    });

    Ok(round)
}

pub async fn diagnose_follow_up(
    data_dir: &Path,
    session: &mut DiagnosisSession,
    user_reply: &str,
    tx: UnboundedSender<DiagnosisStreamEvent>,
) -> Result<(), String> {
    let previous_diagnosis = serde_json::to_string(&session.conversation)
        .map_err(|e| format!("Failed to serialize diagnosis: {}", e))?;

    let mut vars = HashMap::new();
    vars.insert("topic", session.question.clone());
    vars.insert("previous_diagnosis_json", previous_diagnosis);
    vars.insert("user_follow_up_answer", user_reply.to_string());

    let settings = crate::services::config::get_settings_path(data_dir)?;
    let template_set = crate::utils::prompt_templates::get_template_set(&settings.quiz.prompt_template)
        .unwrap_or_else(crate::utils::prompt_templates::get_default_template_set);
    let prompt = crate::utils::prompt_templates::fill_template(
        &template_set.diagnosis_followup_template, &vars
    );
    let response_text = call_llm(&settings.llm, &prompt, 0.3).await?;
    let follow_up = parse_follow_up(&response_text)?;

    session.conversation.push(DiagnosisRound {
        role: "user".to_string(),
        content: user_reply.to_string(),
        blind_spots: vec![],
        follow_up: None,
    });

    session.conversation.push(DiagnosisRound {
        role: "ai".to_string(),
        content: follow_up.progress_assessment.clone(),
        blind_spots: follow_up.new_blind_spots.clone(),
        follow_up: Some(follow_up.follow_up_question.clone()),
    });
    session.current_round += 1;

    if follow_up.should_continue && session.current_round < session.max_rounds {
        let _ = tx.send(DiagnosisStreamEvent::FollowUp {
            question: follow_up.follow_up_question,
            blind_spots: follow_up.new_blind_spots,
        });
    } else {
        let report = generate_diagnosis_report(data_dir, session).await?;
        session.final_report = Some(report.clone());
        let _ = tx.send(DiagnosisStreamEvent::Report {
            summary: report.summary,
            blind_spots: report.blind_spots,
            overall_level: report.overall_level,
            next_steps: report.next_steps,
        });
    }

    Ok(())
}

pub async fn generate_diagnosis_report(
    data_dir: &Path,
    session: &DiagnosisSession,
) -> Result<DiagnosisReport, String> {
    let note_content = crate::services::fs_service::read_file_content(&session.note_path)?;
    let truncated_content: String = note_content.chars().take(8000).collect();

    let conversation_json = serde_json::to_string(&session.conversation)
        .map_err(|e| format!("Failed to serialize conversation: {}", e))?;

    let mut vars = HashMap::new();
    vars.insert("diagnosis_conversation_json", conversation_json);
    vars.insert("note_content", truncated_content);

    let settings = crate::services::config::get_settings_path(data_dir)?;
    let template_set = crate::utils::prompt_templates::get_template_set(&settings.quiz.prompt_template)
        .unwrap_or_else(crate::utils::prompt_templates::get_default_template_set);
    let prompt = crate::utils::prompt_templates::fill_template(
        &template_set.diagnosis_report_template, &vars
    );
    let response_text = call_llm(&settings.llm, &prompt, 0.3).await?;
    parse_diagnosis_report(&response_text)
}

async fn call_llm(settings: &LlmConfig, prompt: &str, temperature: f64) -> Result<String, String> {
    let client = Client::new();

    let mut request_body = serde_json::json!({
        "model": settings.model,
        "messages": [
            { "role": "system", "content": "你是一位严格的学科导师。请用中文输出，并严格按 JSON 格式返回。" },
            { "role": "user", "content": prompt }
        ],
        "stream": false,
        "temperature": temperature,
        "max_tokens": settings.max_tokens,
    });

    if settings.base_url.contains("openai") || settings.base_url.contains("localhost:11434") {
        if let Some(obj) = request_body.as_object_mut() {
            obj.insert("response_format".to_string(), serde_json::json!({ "type": "json_object" }));
        }
    }

    let response = client
        .post(format!("{}/chat/completions", settings.base_url.trim_end_matches('/')))
        .header("Authorization", format!("Bearer {}", settings.api_key))
        .header("Content-Type", "application/json")
        .json(&request_body)
        .send()
        .await
        .map_err(|e| format!("LLM API request failed: {}", e))?;

    if !response.status().is_success() {
        let status = response.status();
        let body = response.text().await.unwrap_or_default();
        return Err(format!("LLM API error {}: {}", status, body));
    }

    let json: Value = response.json().await
        .map_err(|e| format!("Failed to parse LLM response: {}", e))?;

    json["choices"][0]["message"]["content"]
        .as_str()
        .map(String::from)
        .ok_or_else(|| "No content in LLM response".to_string())
}

struct InitialDiagnosis {
    answer_analysis: String,
    blind_spots: Vec<BlindSpot>,
    follow_up_question: Option<String>,
}

fn parse_diagnosis_initial(raw: &str) -> Result<InitialDiagnosis, String> {
    let json_str = extract_json_block(raw);
    let parsed: Value = serde_json::from_str(&json_str)
        .map_err(|e| format!("Failed to parse diagnosis: {}. Raw: {}", e, &raw[..raw.len().min(200)]))?;

    Ok(InitialDiagnosis {
        answer_analysis: parsed["answer_analysis"].as_str().unwrap_or("").to_string(),
        blind_spots: parse_blind_spots(&parsed["blind_spot"]),
        follow_up_question: parsed["follow_up_question"].as_str().map(String::from),
    })
}

fn initial_diagnosis_round(diagnosis: &InitialDiagnosis) -> DiagnosisRound {
    DiagnosisRound {
        role: "ai".to_string(),
        content: diagnosis.answer_analysis.clone(),
        blind_spots: diagnosis.blind_spots.clone(),
        follow_up: diagnosis.follow_up_question.clone(),
    }
}

struct FollowUpResponse {
    progress_assessment: String,
    new_blind_spots: Vec<BlindSpot>,
    should_continue: bool,
    follow_up_question: String,
}

fn parse_follow_up(raw: &str) -> Result<FollowUpResponse, String> {
    let json_str = extract_json_block(raw);
    let parsed: Value = serde_json::from_str(&json_str)
        .map_err(|e| format!("Failed to parse follow-up: {}", e))?;

    Ok(FollowUpResponse {
        progress_assessment: parsed["progress_assessment"].as_str().unwrap_or("").to_string(),
        new_blind_spots: parse_blind_spots_array(&parsed["new_blind_spots"]),
        should_continue: parsed["should_continue"].as_bool().unwrap_or(false),
        follow_up_question: parsed["follow_up_question"].as_str().unwrap_or("").to_string(),
    })
}

fn parse_diagnosis_report(raw: &str) -> Result<DiagnosisReport, String> {
    let json_str = extract_json_block(raw);
    let parsed: Value = serde_json::from_str(&json_str)
        .map_err(|e| format!("Failed to parse report: {}", e))?;

    Ok(DiagnosisReport {
        summary: parsed["summary"].as_str().unwrap_or("").to_string(),
        blind_spots: parse_blind_spots_array(&parsed["blind_spots"]),
        overall_level: parsed["overall_level"].as_str().unwrap_or("").to_string(),
        next_steps: parsed["next_steps"].as_array()
            .map(|arr| arr.iter().filter_map(|v| v.as_str().map(String::from)).collect())
            .unwrap_or_default(),
    })
}

fn parse_blind_spots(value: &Value) -> Vec<BlindSpot> {
    vec![BlindSpot {
        tag: value["type"].as_str().unwrap_or("").to_string(),
        severity: "medium".to_string(),
        description: value["description"].as_str().unwrap_or("").to_string(),
        note_reference: String::new(),
        suggestion: String::new(),
    }]
}

fn parse_blind_spots_array(value: &Value) -> Vec<BlindSpot> {
    value.as_array()
        .map(|arr| arr.iter().map(|v| BlindSpot {
            tag: v["tag"].as_str().or(v["type"].as_str()).unwrap_or("").to_string(),
            severity: v["severity"].as_str().unwrap_or("medium").to_string(),
            description: v["description"].as_str().unwrap_or("").to_string(),
            note_reference: v["note_reference"].as_str().unwrap_or("").to_string(),
            suggestion: v["suggestion"].as_str().unwrap_or("").to_string(),
        }).collect())
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_quiz_response_accepts_type_alias_from_spec() {
        let raw = r#"{
            "questions": [
                {
                    "id": "q1",
                    "type": "multiple",
                    "question": "Which claims are true?",
                    "options": ["A. Alpha", "B. Beta", "C. Gamma"],
                    "answer": "A,C",
                    "explanation": "Alpha and Gamma are true."
                }
            ]
        }"#;

        let questions = parse_quiz_response(raw).expect("quiz response should parse");

        assert_eq!(questions.len(), 1);
        assert!(matches!(questions[0].question_type, QuestionType::Multiple));
    }

    #[test]
    fn parse_quiz_response_rejects_empty_question_text() {
        let raw = r#"{
            "questions": [
                {
                    "id": "q1",
                    "question_type": "single",
                    "question": "",
                    "options": ["A. Alpha", "B. Beta"],
                    "answer": "A",
                    "explanation": "Alpha is true."
                }
            ]
        }"#;

        let error = parse_quiz_response(raw).expect_err("empty question should be rejected");

        assert!(error.contains("question text"));
    }

    #[test]
    fn parse_quiz_response_rejects_choice_question_without_options() {
        let raw = r#"{
            "questions": [
                {
                    "id": "q1",
                    "question_type": "single",
                    "question": "Which claim is true?",
                    "options": [],
                    "answer": "A",
                    "explanation": "Alpha is true."
                }
            ]
        }"#;

        let error = parse_quiz_response(raw).expect_err("choice question options are required");

        assert!(error.contains("options"));
    }

    #[test]
    fn initial_diagnosis_round_keeps_content_blind_spots_and_follow_up() {
        let diagnosis = InitialDiagnosis {
            answer_analysis: "The first reasoning step confuses two concepts.".to_string(),
            blind_spots: vec![BlindSpot {
                tag: "concept confusion".to_string(),
                severity: "medium".to_string(),
                description: "A and B were treated as equivalent.".to_string(),
                note_reference: String::new(),
                suggestion: String::new(),
            }],
            follow_up_question: Some("How would you distinguish A from B?".to_string()),
        };

        let round = initial_diagnosis_round(&diagnosis);

        assert_eq!(round.role, "ai");
        assert_eq!(round.content, diagnosis.answer_analysis);
        assert_eq!(round.blind_spots.len(), 1);
        assert_eq!(round.follow_up, diagnosis.follow_up_question);
    }
}
