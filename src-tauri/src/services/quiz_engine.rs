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
    Chunk {
        id: String,
        question_type: String,
        question: String,
        options: Vec<String>,
        answer: String,
        explanation: String,
    },
    Done {
        total: u32,
    },
    Error {
        message: String,
    },
}

#[derive(Clone, serde::Serialize)]
#[serde(rename_all = "camelCase", tag = "event", content = "data")]
pub enum DiagnosisStreamEvent {
    Initial {
        role: String,
        content: String,
        blind_spots: Vec<BlindSpot>,
        follow_up: Option<String>,
    },
    FollowUp {
        question: String,
        blind_spots: Vec<BlindSpot>,
    },
    Report {
        summary: String,
        blind_spots: Vec<BlindSpot>,
        overall_level: String,
        next_steps: Vec<String>,
    },
    Error {
        message: String,
    },
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

    let question_types_str = params
        .types
        .iter()
        .map(|qt| format!("{:?}", qt).to_lowercase())
        .collect::<Vec<_>>()
        .join(", ");
    let mut vars = HashMap::new();
    vars.insert("note_content", truncated_content);
    vars.insert("question_types", question_types_str);
    vars.insert("count", params.count.to_string());
    vars.insert("difficulty", params.difficulty.clone());
    vars.insert("language", params.lang.clone());
    let template_set =
        crate::utils::prompt_templates::get_template_set(&settings.quiz.prompt_template)
            .unwrap_or_else(crate::utils::prompt_templates::get_default_template_set);
    let prompt = crate::utils::prompt_templates::fill_template(&template_set.quiz_template, &vars);

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
        .post(format!(
            "{}/chat/completions",
            llm.base_url.trim_end_matches('/')
        ))
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
                                if let Some(content) =
                                    json["choices"][0]["delta"]["content"].as_str()
                                {
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
            let _ = tx.send(QuizStreamEvent::Done {
                total: questions.len() as u32,
            });
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
    let parsed: serde_json::Value = serde_json::from_str(&json_str).map_err(|e| {
        format!(
            "JSON parse error: {}. Raw: {}",
            e,
            &raw[..raw.len().min(200)]
        )
    })?;

    let questions = parsed["questions"]
        .as_array()
        .ok_or("Missing 'questions' array in response")?;
    if questions.is_empty() {
        return Err("Missing non-empty 'questions' array in response".to_string());
    }

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

    if matches!(question_type, QuestionType::Single | QuestionType::Multiple) && options.is_empty()
    {
        return Err(format!(
            "Question {} options are required for choice questions",
            index + 1
        ));
    }
    validate_choice_answer(&question_type, &options, &answer, index)?;

    Ok(QuizQuestion {
        id,
        question_type,
        question,
        options,
        answer,
        explanation,
    })
}

fn validate_choice_answer(
    question_type: &QuestionType,
    options: &[String],
    answer: &str,
    index: usize,
) -> Result<(), String> {
    if !matches!(question_type, QuestionType::Single | QuestionType::Multiple) {
        return Ok(());
    }

    let letters = choice_letters_from_answer(answer);
    if !letters.is_empty() {
        if matches!(question_type, QuestionType::Single) && letters.len() != 1 {
            return Err(format!(
                "Question {} single choice answer must contain exactly one option",
                index + 1
            ));
        }

        for letter in letters {
            let option_index = (letter as u8).saturating_sub(b'A') as usize;
            if option_index >= options.len() {
                return Err(format!(
                    "Question {} answer '{}' is outside the available options",
                    index + 1,
                    answer
                ));
            }
        }
        return Ok(());
    }

    let normalized_answers = split_answer_text(answer);
    if matches!(question_type, QuestionType::Single) && normalized_answers.len() != 1 {
        return Err(format!(
            "Question {} single choice answer must contain exactly one option",
            index + 1
        ));
    }

    if normalized_answers.iter().all(|normalized_answer| {
        options.iter().any(|option| {
            normalize_answer_text(option) == *normalized_answer
                || normalize_answer_text(&strip_option_label(option)) == *normalized_answer
        })
    }) {
        return Ok(());
    }

    Err(format!(
        "Question {} answer '{}' does not match any available options",
        index + 1,
        answer
    ))
}

fn choice_letters_from_answer(answer: &str) -> Vec<char> {
    let chars: Vec<char> = answer.to_ascii_uppercase().chars().collect();
    let mut letters = Vec::new();

    for (i, c) in chars.iter().enumerate() {
        if !c.is_ascii_uppercase() {
            continue;
        }
        let prev_is_letter = i > 0 && chars[i - 1].is_ascii_uppercase();
        let next_is_letter = i + 1 < chars.len() && chars[i + 1].is_ascii_uppercase();
        if !prev_is_letter && !next_is_letter && !letters.contains(c) {
            letters.push(*c);
        }
    }

    letters
}

fn normalize_answer_text(value: &str) -> String {
    value
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .trim()
        .to_ascii_lowercase()
}

fn split_answer_text(answer: &str) -> Vec<String> {
    let parts = answer
        .split(|c| matches!(c, ',' | ';' | '，' | '；' | '、'))
        .map(normalize_answer_text)
        .filter(|value| !value.is_empty())
        .collect::<Vec<_>>();

    if parts.is_empty() {
        vec![normalize_answer_text(answer)]
    } else {
        parts
    }
}

fn strip_option_label(option: &str) -> String {
    let trimmed = option.trim();
    let mut chars = trimmed.char_indices();
    let Some((_, first)) = chars.next() else {
        return String::new();
    };
    if !first.is_ascii_alphabetic() {
        return trimmed.to_string();
    }

    let after_first = &trimmed[first.len_utf8()..];
    let after_spaces = after_first.trim_start();
    let Some(separator) = after_spaces.chars().next() else {
        return trimmed.to_string();
    };
    if matches!(separator, '.' | '．' | '、' | ')' | ':' | '：') {
        after_spaces[separator.len_utf8()..].trim().to_string()
    } else {
        trimmed.to_string()
    }
}

fn parse_question_type(raw: &str, index: usize) -> Result<QuestionType, String> {
    match raw.trim().to_ascii_lowercase().as_str() {
        "single" => Ok(QuestionType::Single),
        "multiple" => Ok(QuestionType::Multiple),
        "short" => Ok(QuestionType::Short),
        other => Err(format!(
            "Question {} has unsupported question type: {}",
            index + 1,
            other
        )),
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
        let json_start = start + "```json".len();
        if let Some(end) = raw[json_start..].find("```") {
            let json_end = json_start + end;
            return raw[json_start..json_end].trim().to_string();
        }
    }
    if let Some(start) = raw.find('{') {
        let mut depth = 0;
        let mut in_string = false;
        let mut escaped = false;
        let json_candidate = &raw[start..];
        for (i, c) in json_candidate.char_indices() {
            if in_string {
                if escaped {
                    escaped = false;
                } else if c == '\\' {
                    escaped = true;
                } else if c == '"' {
                    in_string = false;
                }
                continue;
            }

            match c {
                '"' => in_string = true,
                '{' => depth += 1,
                '}' => {
                    depth -= 1;
                    if depth == 0 {
                        return json_candidate[..i + 1].to_string();
                    }
                }
                _ => {}
            }
        }
    }
    raw.to_string()
}

pub async fn submit_diagnosis_initial(
    data_dir: &Path,
    question: &str,
    correct_answer: &str,
    user_answer: &str,
    user_reasoning: &str,
    note_path: &str,
    tx: UnboundedSender<DiagnosisStreamEvent>,
) -> Result<DiagnosisRound, String> {
    let note_content = crate::services::fs_service::read_file_content(note_path)?;
    let truncated_content: String = note_content.chars().take(8000).collect();

    let settings = crate::services::config::get_settings_path(data_dir)?;
    let template_set =
        crate::utils::prompt_templates::get_template_set(&settings.quiz.prompt_template)
            .unwrap_or_else(crate::utils::prompt_templates::get_default_template_set);
    let prompt = build_diagnosis_initial_prompt(
        &template_set.diagnosis_initial_template,
        &truncated_content,
        question,
        correct_answer,
        user_answer,
        user_reasoning,
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

fn build_diagnosis_initial_prompt(
    template: &str,
    note_content: &str,
    question: &str,
    correct_answer: &str,
    user_answer: &str,
    user_reasoning: &str,
) -> String {
    let mut vars = HashMap::new();
    vars.insert("note_content", note_content.to_string());
    vars.insert("question", question.to_string());
    vars.insert("correct_answer", correct_answer.to_string());
    vars.insert("user_answer", user_answer.to_string());
    vars.insert("user_reasoning", user_reasoning.to_string());
    crate::utils::prompt_templates::fill_template(template, &vars)
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
    let template_set =
        crate::utils::prompt_templates::get_template_set(&settings.quiz.prompt_template)
            .unwrap_or_else(crate::utils::prompt_templates::get_default_template_set);
    let prompt = crate::utils::prompt_templates::fill_template(
        &template_set.diagnosis_followup_template,
        &vars,
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
    let template_set =
        crate::utils::prompt_templates::get_template_set(&settings.quiz.prompt_template)
            .unwrap_or_else(crate::utils::prompt_templates::get_default_template_set);
    let prompt = crate::utils::prompt_templates::fill_template(
        &template_set.diagnosis_report_template,
        &vars,
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
            obj.insert(
                "response_format".to_string(),
                serde_json::json!({ "type": "json_object" }),
            );
        }
    }

    let response = client
        .post(format!(
            "{}/chat/completions",
            settings.base_url.trim_end_matches('/')
        ))
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

    let json: Value = response
        .json()
        .await
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
    let parsed: Value = serde_json::from_str(&json_str).map_err(|e| {
        format!(
            "Failed to parse diagnosis: {}. Raw: {}",
            e,
            &raw[..raw.len().min(200)]
        )
    })?;

    Ok(InitialDiagnosis {
        answer_analysis: required_diagnosis_string(&parsed, "answer_analysis")?,
        blind_spots: parse_initial_blind_spots(&parsed)?,
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
    let parsed: Value =
        serde_json::from_str(&json_str).map_err(|e| format!("Failed to parse follow-up: {}", e))?;
    let should_continue = parsed["should_continue"]
        .as_bool()
        .ok_or_else(|| "Diagnosis response missing should_continue".to_string())?;
    let follow_up_question = string_field(&parsed, "follow_up_question").unwrap_or_default();

    if should_continue && follow_up_question.is_empty() {
        return Err("Diagnosis response missing follow_up_question".to_string());
    }

    Ok(FollowUpResponse {
        progress_assessment: required_diagnosis_string(&parsed, "progress_assessment")?,
        new_blind_spots: parse_blind_spots_array(&parsed["new_blind_spots"], "new_blind_spots")?,
        should_continue,
        follow_up_question,
    })
}

fn parse_diagnosis_report(raw: &str) -> Result<DiagnosisReport, String> {
    let json_str = extract_json_block(raw);
    let parsed: Value =
        serde_json::from_str(&json_str).map_err(|e| format!("Failed to parse report: {}", e))?;
    let next_steps = parse_non_empty_string_array(&parsed["next_steps"], "next_steps")?;

    Ok(DiagnosisReport {
        summary: required_diagnosis_string(&parsed, "summary")?,
        blind_spots: parse_blind_spots_array(&parsed["blind_spots"], "blind_spots")?,
        overall_level: required_diagnosis_string(&parsed, "overall_level")?,
        next_steps,
    })
}

fn parse_blind_spot(value: &Value, label: &str) -> Result<BlindSpot, String> {
    let tag = string_field(value, "tag")
        .or_else(|| string_field(value, "type"))
        .or_else(|| string_field(value, "concept"))
        .ok_or_else(|| format!("Diagnosis response missing {} tag", label))?;
    let description = string_field(value, "description")
        .ok_or_else(|| format!("Diagnosis response missing {} description", label))?;

    Ok(BlindSpot {
        tag,
        severity: string_field(value, "severity").unwrap_or_else(|| "medium".to_string()),
        description,
        note_reference: string_field(value, "note_reference").unwrap_or_default(),
        suggestion: string_field(value, "suggestion").unwrap_or_default(),
    })
}

fn parse_initial_blind_spots(parsed: &Value) -> Result<Vec<BlindSpot>, String> {
    if let Some(values) = parsed["blind_spots"].as_array() {
        let spots = values
            .iter()
            .enumerate()
            .map(|(index, value)| parse_blind_spot(value, &format!("blind_spots[{}]", index + 1)))
            .collect::<Result<Vec<_>, _>>()?;

        if spots.is_empty() {
            Err("Diagnosis response missing blind_spots".to_string())
        } else {
            Ok(spots)
        }
    } else {
        parse_blind_spot(&parsed["blind_spot"], "blind_spot").map(|blind_spot| vec![blind_spot])
    }
}

fn parse_blind_spots_array(value: &Value, label: &str) -> Result<Vec<BlindSpot>, String> {
    let Some(values) = value.as_array() else {
        return Ok(vec![]);
    };

    values
        .iter()
        .enumerate()
        .map(|(index, value)| parse_blind_spot(value, &format!("{}[{}]", label, index + 1)))
        .collect()
}

fn required_diagnosis_string(value: &Value, field: &str) -> Result<String, String> {
    value[field]
        .as_str()
        .map(str::trim)
        .filter(|raw| !raw.is_empty())
        .map(String::from)
        .ok_or_else(|| format!("Diagnosis response missing {}", field))
}

fn string_field(value: &Value, field: &str) -> Option<String> {
    value[field]
        .as_str()
        .map(str::trim)
        .filter(|raw| !raw.is_empty())
        .map(String::from)
}

fn parse_non_empty_string_array(value: &Value, field: &str) -> Result<Vec<String>, String> {
    let items = value
        .as_array()
        .ok_or_else(|| format!("Diagnosis response missing {}", field))?
        .iter()
        .filter_map(|item| item.as_str())
        .map(str::trim)
        .filter(|item| !item.is_empty())
        .map(String::from)
        .collect::<Vec<_>>();

    if items.is_empty() {
        Err(format!("Diagnosis response missing {}", field))
    } else {
        Ok(items)
    }
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
    fn parse_quiz_response_accepts_fenced_json() {
        let raw = r#"```json
{
  "questions": [
    {
      "id": "q1",
      "question_type": "single",
      "question": "Which claim is true?",
      "options": ["A. Alpha", "B. Beta"],
      "answer": "A",
      "explanation": "Alpha is true."
    }
  ]
}
```"#;

        let questions = parse_quiz_response(raw).expect("fenced JSON should parse");

        assert_eq!(questions.len(), 1);
        assert_eq!(questions[0].id, "q1");
    }

    #[test]
    fn parse_quiz_response_accepts_braces_inside_question_text() {
        let raw = r#"The quiz is:
{
  "questions": [
    {
      "id": "q1",
      "question_type": "short",
      "question": "What does `fn main() {` start in Rust?",
      "options": [],
      "answer": "A function body.",
      "explanation": "The opening brace starts the function body."
    }
  ]
}
Good luck."#;

        let questions =
            parse_quiz_response(raw).expect("JSON extraction should ignore braces inside strings");

        assert_eq!(questions.len(), 1);
        assert!(questions[0].question.contains("fn main()"));
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
    fn parse_quiz_response_rejects_empty_question_list() {
        let raw = r#"{ "questions": [] }"#;

        let error = parse_quiz_response(raw).expect_err("empty question list should be rejected");

        assert!(error.contains("questions"));
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
    fn parse_quiz_response_rejects_single_choice_answer_outside_options() {
        let raw = r#"{
            "questions": [
                {
                    "id": "q1",
                    "question_type": "single",
                    "question": "Which claim is true?",
                    "options": ["A. Alpha", "B. Beta"],
                    "answer": "D",
                    "explanation": "Alpha is true."
                }
            ]
        }"#;

        let error =
            parse_quiz_response(raw).expect_err("answer outside options should be rejected");

        assert!(error.contains("answer"));
        assert!(error.contains("options"));
    }

    #[test]
    fn parse_quiz_response_rejects_single_choice_answer_text_list() {
        let raw = r#"{
            "questions": [
                {
                    "id": "q1",
                    "question_type": "single",
                    "question": "Which claim is true?",
                    "options": ["A. Alpha", "B. Beta"],
                    "answer": "Alpha, Beta",
                    "explanation": "Only one option should be correct."
                }
            ]
        }"#;

        let error = parse_quiz_response(raw)
            .expect_err("single choice answer text list should be rejected");

        assert!(error.contains("single choice"));
    }

    #[test]
    fn parse_quiz_response_accepts_multiple_choice_answer_text_list() {
        let raw = r#"{
            "questions": [
                {
                    "id": "q1",
                    "question_type": "multiple",
                    "question": "Which claims are true?",
                    "options": ["A. Alpha", "B. Beta", "C. Gamma"],
                    "answer": "Alpha, Gamma",
                    "explanation": "Alpha and Gamma are true."
                }
            ]
        }"#;

        let questions =
            parse_quiz_response(raw).expect("multiple choice answer text list should parse");

        assert_eq!(questions.len(), 1);
        assert_eq!(questions[0].answer, "Alpha, Gamma");
    }

    #[test]
    fn diagnosis_initial_prompt_includes_known_correct_answer() {
        let prompt = build_diagnosis_initial_prompt(
            "Question={{question}}\nCorrect={{correct_answer}}\nUser={{user_answer}}\nReason={{user_reasoning}}\nNote={{note_content}}",
            "Ownership note",
            "Which claim is true?",
            "B. Borrowing keeps ownership with the original variable.",
            "A",
            "I thought borrowing moves ownership.",
        );

        assert!(prompt.contains("Correct=B. Borrowing keeps ownership with the original variable."));
        assert!(!prompt.contains("to be determined"));
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

    #[test]
    fn parse_diagnosis_initial_rejects_empty_answer_analysis() {
        let raw = r#"{
            "answer_analysis": "",
            "blind_spot": {
                "type": "concept confusion",
                "description": "Move semantics were treated as cloning."
            },
            "follow_up_question": "When does Rust clone?"
        }"#;

        let error = match parse_diagnosis_initial(raw) {
            Ok(_) => panic!("empty answer analysis should be rejected"),
            Err(error) => error,
        };

        assert!(error.contains("answer_analysis"));
    }

    #[test]
    fn parse_diagnosis_initial_rejects_empty_blind_spot() {
        let raw = r#"{
            "answer_analysis": "You confused move and copy.",
            "blind_spot": {},
            "follow_up_question": "When does Rust clone?"
        }"#;

        let error = match parse_diagnosis_initial(raw) {
            Ok(_) => panic!("empty blind spot should be rejected"),
            Err(error) => error,
        };

        assert!(error.contains("blind_spot"));
    }

    #[test]
    fn parse_diagnosis_initial_accepts_blind_spots_array() {
        let raw = r#"{
            "answer_analysis": "You confused move and copy.",
            "blind_spots": [
                {
                    "tag": "move semantics",
                    "severity": "high",
                    "description": "Move and clone were treated as equivalent.",
                    "note_reference": "Ownership section",
                    "suggestion": "Review move examples."
                }
            ],
            "follow_up_question": "When does Rust clone?"
        }"#;

        let diagnosis = parse_diagnosis_initial(raw).expect("blind_spots array should be accepted");

        assert_eq!(diagnosis.blind_spots.len(), 1);
        assert_eq!(diagnosis.blind_spots[0].tag, "move semantics");
        assert_eq!(diagnosis.blind_spots[0].severity, "high");
    }

    #[test]
    fn parse_follow_up_rejects_continue_without_question() {
        let raw = r#"{
            "progress_assessment": "The user is improving.",
            "new_blind_spots": [],
            "should_continue": true,
            "follow_up_question": ""
        }"#;

        let error = match parse_follow_up(raw) {
            Ok(_) => panic!("continued follow-up should include a question"),
            Err(error) => error,
        };

        assert!(error.contains("follow_up_question"));
    }

    #[test]
    fn parse_diagnosis_report_rejects_empty_summary() {
        let raw = r#"{
            "summary": "",
            "blind_spots": [],
            "overall_level": "Needs review",
            "next_steps": ["Review ownership examples"]
        }"#;

        let error = match parse_diagnosis_report(raw) {
            Ok(_) => panic!("empty report summary should be rejected"),
            Err(error) => error,
        };

        assert!(error.contains("summary"));
    }

    #[test]
    fn parse_diagnosis_report_rejects_empty_blind_spot_items() {
        let raw = r#"{
            "summary": "Review ownership.",
            "blind_spots": [{}],
            "overall_level": "Needs review",
            "next_steps": ["Review ownership examples"]
        }"#;

        let error = match parse_diagnosis_report(raw) {
            Ok(_) => panic!("empty blind spot item should be rejected"),
            Err(error) => error,
        };

        assert!(error.contains("blind_spots"));
    }
}
