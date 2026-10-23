use crate::errors::AppError;
use crate::models::diagnosis::*;
use crate::models::llm_run::{
    endpoint_host_of, LlmRunKind, LlmRunRecord, LlmUsage, LlmUsageSource,
};
use crate::models::quiz::*;
use crate::models::settings::LlmConfig;
use crate::services::llm_run_store;
use crate::services::llm_service::{http_client, streaming_http_client};
use futures_util::StreamExt;
use serde_json::Value;
use std::collections::HashMap;
use std::path::Path;
use std::time::Duration;
use tokio::sync::mpsc::UnboundedSender;

/// Maximum gap between LLM stream chunks before a generation is considered
/// hung. Streaming requests carry no total deadline (local models can think
/// for minutes), so this idle bound is the only stream-level timeout.
const LLM_STREAM_IDLE_TIMEOUT: Duration = Duration::from_secs(90);

#[derive(Clone, serde::Serialize)]
#[serde(rename_all = "camelCase", tag = "event", content = "data")]
pub enum QuizStreamEvent {
    Phase {
        phase: String,
    },
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
#[serde(rename_all = "snake_case", tag = "event", content = "data")]
pub enum DiagnosisStreamEvent {
    /// Progress marker emitted between LLM calls so the UI can show which stage
    /// the diagnosis is in (the same values the quiz stream uses).
    Phase {
        phase: String,
    },
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
) -> Result<(), AppError> {
    let resolved_note =
        crate::services::fs_service::resolve_note_path(data_dir, &params.path).await?;
    let note_content =
        crate::services::fs_service::read_file_content(&resolved_note.to_string_lossy()).await?;
    let truncated_content = note_content_for_prompt(&note_content);
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

    if tx
        .send(QuizStreamEvent::Phase {
            phase: "requesting_model".to_string(),
        })
        .is_err()
    {
        // Receiver gone: the client disconnected before generation started.
        return Ok(());
    }

    let client = streaming_http_client();
    let llm_started = std::time::Instant::now();
    let llm_ts_start = chrono::Local::now().to_rfc3339();
    let mut stream_usage: Option<LlmUsage> = None;
    let mut stream_usage_source = LlmUsageSource::Unavailable;
    let request_body = serde_json::json!({
        "model": llm.model,
        "messages": [
            { "role": "system", "content": "你是一个知识测验生成助手。请严格按照指定的 JSON 格式返回。" },
            { "role": "user", "content": prompt }
        ],
        "stream": true,
        "stream_options": { "include_usage": true },
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
        .map_err(|e| AppError::Llm(format!("LLM API request failed: {}", e)))?;

    let http_status = Some(response.status().as_u16());

    if !response.status().is_success() {
        let status = response.status();
        let body = response.text().await.unwrap_or_default();
        let _ = tx.send(QuizStreamEvent::Error {
            message: format!("LLM API error {}: {}", status, body),
        });
        record_quiz_stream_run(
            data_dir,
            llm,
            &prompt,
            llm_started,
            &llm_ts_start,
            http_status,
            None,
            LlmUsageSource::Unavailable,
            Some(format!("LLM API error {status}")),
            Some(&body),
            llm.record_llm_io,
        );
        return Err(AppError::Llm(format!("LLM API error {}: {}", status, body)));
    }

    let mut stream = response.bytes_stream();
    let mut accumulated_bytes: Vec<u8> = Vec::new();

    loop {
        let next_chunk = match tokio::time::timeout(LLM_STREAM_IDLE_TIMEOUT, stream.next()).await {
            Ok(chunk_result) => chunk_result,
            Err(_) => {
                let message = format!(
                    "LLM stream idle for over {}s; generation aborted",
                    LLM_STREAM_IDLE_TIMEOUT.as_secs()
                );
                let _ = tx.send(QuizStreamEvent::Error {
                    message: message.clone(),
                });
                let partial = String::from_utf8_lossy(&accumulated_bytes).into_owned();
                record_quiz_stream_run(
                    data_dir,
                    llm,
                    &prompt,
                    llm_started,
                    &llm_ts_start,
                    http_status,
                    stream_usage,
                    stream_usage_source,
                    Some(message.clone()),
                    Some(&partial),
                    llm.record_llm_io,
                );
                return Err(AppError::Llm(message));
            }
        };
        let Some(chunk_result) = next_chunk else {
            break;
        };
        match chunk_result {
            Ok(bytes) => {
                if tx.is_closed() {
                    // Client disconnected mid-stream: stop pulling the LLM.
                    return Ok(());
                }
                accumulated_bytes.extend_from_slice(&bytes);
            }
            Err(e) => {
                let _ = tx.send(QuizStreamEvent::Error {
                    message: format!("Stream error: {}", e),
                });
            }
        }
    }

    // Decode accumulated bytes once at the end to handle UTF-8 multibyte chars
    // split across TCP chunk boundaries.
    let raw_text = String::from_utf8_lossy(&accumulated_bytes).into_owned();
    let mut accumulated = String::new();
    for line in raw_text.lines() {
        if let Some(data) = line.strip_prefix("data: ") {
            if data.trim() == "[DONE]" {
                continue;
            }
            if let Ok(json) = serde_json::from_str::<Value>(data) {
                if let Some(text) = json["choices"][0]["delta"]["content"].as_str() {
                    accumulated.push_str(text);
                }
                // With `stream_options.include_usage` the endpoint sends a final
                // chunk carrying `usage` (usually with `"choices": []`).
                let (usage, source) = parse_usage(&json);
                if usage.is_some() {
                    stream_usage = usage;
                    stream_usage_source = source;
                }
            }
        }
    }

    record_quiz_stream_run(
        data_dir,
        llm,
        &prompt,
        llm_started,
        &llm_ts_start,
        http_status,
        stream_usage,
        stream_usage_source,
        None,
        Some(&accumulated),
        llm.record_llm_io,
    );

    if tx
        .send(QuizStreamEvent::Phase {
            phase: "parsing_response".to_string(),
        })
        .is_err()
    {
        return Ok(());
    }

    match parse_quiz_response(&accumulated) {
        Ok(questions) => {
            for q in &questions {
                if tx
                    .send(QuizStreamEvent::Chunk {
                        id: q.id.clone(),
                        question_type: format!("{:?}", q.question_type).to_lowercase(),
                        question: q.question.clone(),
                        options: q.options.clone(),
                        answer: q.answer.clone(),
                        explanation: q.explanation.clone(),
                    })
                    .is_err()
                {
                    return Ok(());
                }
            }
            let _ = tx.send(QuizStreamEvent::Done {
                total: questions.len() as u32,
            });
        }
        Err(e) => {
            // Don't spend a fixup roundtrip if the receiver is already gone.
            if tx.is_closed() {
                return Ok(());
            }
            // Attempt one auto-repair by asking the LLM to fix the JSON
            let fixup_prompt = format!(
                "你刚才返回了以下内容，但它不是有效的 JSON 格式。请用标准 JSON 格式重新输出完整的题目数组。\n\n原始返回内容：\n```\n{}\n```\n\n请严格按照以下格式输出：\n```json\n{{\n  \"questions\": [\n    {{\n      \"question_type\": \"single\",\n      \"question\": \"题目文本\",\n      \"options\": [\"A. 选项\", \"B. 选项\"],\n      \"answer\": \"正确答案字母或文本\",\n      \"explanation\": \"解释\"\n    }}\n  ]\n}}\n```",
                accumulated
            );
            match call_llm(llm, &fixup_prompt, 0.3, LlmRunKind::QuizFixup, data_dir).await {
                Ok(fixup_text) => match parse_quiz_response(&fixup_text) {
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
                        return Ok(());
                    }
                    Err(fixup_error) => {
                        let _ = tx.send(QuizStreamEvent::Error {
                            message: format!(
                                "Failed to parse quiz response after retry: {} (original: {})",
                                fixup_error, e
                            ),
                        });
                    }
                },
                Err(fixup_err) => {
                    let _ = tx.send(QuizStreamEvent::Error {
                        message: format!(
                            "Failed to parse quiz response: {} (fixup also failed: {})",
                            e, fixup_err
                        ),
                    });
                }
            }
        }
    }

    Ok(())
}

/// Prefix of `raw` for error messages, cut on a UTF-8 char boundary so
/// multibyte model output (CJK notes/prompts) cannot panic while formatting.
fn raw_preview(raw: &str) -> &str {
    let mut end = raw.len().min(200);
    while end > 0 && !raw.is_char_boundary(end) {
        end -= 1;
    }
    &raw[..end]
}

fn parse_quiz_response(raw: &str) -> Result<Vec<QuizQuestion>, AppError> {
    let json_str = extract_json_block(raw);
    let parsed: serde_json::Value = serde_json::from_str(&json_str).map_err(|e| {
        AppError::Internal(format!(
            "JSON parse error: {}. Raw: {}",
            e,
            raw_preview(raw)
        ))
    })?;

    let questions = parsed["questions"]
        .as_array()
        .ok_or(AppError::InvalidInput(
            "Missing 'questions' array in response".to_string(),
        ))?;
    if questions.is_empty() {
        return Err(AppError::InvalidInput(
            "Missing non-empty 'questions' array in response".to_string(),
        ));
    }

    let mut result = Vec::new();
    for (i, q) in questions.iter().enumerate() {
        let question = parse_quiz_question(q, i)?;
        result.push(question);
    }
    Ok(result)
}

fn parse_quiz_question(q: &Value, index: usize) -> Result<QuizQuestion, AppError> {
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
        return Err(AppError::InvalidInput(format!(
            "Question {} options are required for choice questions",
            index + 1
        )));
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
) -> Result<(), AppError> {
    if !matches!(question_type, QuestionType::Single | QuestionType::Multiple) {
        return Ok(());
    }

    let letters = choice_letters_from_answer(answer);
    if !letters.is_empty() {
        if matches!(question_type, QuestionType::Single) && letters.len() != 1 {
            return Err(AppError::InvalidInput(format!(
                "Question {} single choice answer must contain exactly one option",
                index + 1
            )));
        }

        for letter in letters {
            let option_index = (letter as u8).saturating_sub(b'A') as usize;
            if option_index >= options.len() {
                return Err(AppError::InvalidInput(format!(
                    "Question {} answer '{}' is outside the available options",
                    index + 1,
                    answer
                )));
            }
        }
        return Ok(());
    }

    let normalized_answer = normalize_answer_text(answer);
    if option_text_matches(options, &normalized_answer) {
        return Ok(());
    }

    let normalized_answers = split_answer_text(answer, options);
    if matches!(question_type, QuestionType::Single) && normalized_answers.len() != 1 {
        return Err(AppError::InvalidInput(format!(
            "Question {} single choice answer must contain exactly one option",
            index + 1
        )));
    }

    if normalized_answers
        .iter()
        .all(|normalized_answer| option_text_matches(options, normalized_answer))
    {
        return Ok(());
    }

    Err(AppError::InvalidInput(format!(
        "Question {} answer '{}' does not match any available options",
        index + 1,
        answer
    )))
}

fn option_text_matches(options: &[String], normalized_answer: &str) -> bool {
    options.iter().any(|option| {
        normalize_answer_text(option) == normalized_answer
            || normalize_answer_text(&strip_option_label(option)) == normalized_answer
    })
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

fn split_answer_text(answer: &str, options: &[String]) -> Vec<String> {
    let punctuation_parts = answer
        .split([',', ';', '，', '；', '、'])
        .map(normalize_answer_text)
        .filter(|value| !value.is_empty())
        .collect::<Vec<_>>();

    let parts = punctuation_parts
        .into_iter()
        .flat_map(|part| {
            if option_text_matches(options, &part) {
                vec![part]
            } else {
                split_word_joined_answer_text(&part)
            }
        })
        .collect::<Vec<_>>();

    if parts.is_empty() {
        vec![normalize_answer_text(answer)]
    } else {
        parts
    }
}

fn split_word_joined_answer_text(answer: &str) -> Vec<String> {
    let mut parts = Vec::new();
    let mut current = Vec::new();

    for word in answer.split_whitespace() {
        if word.eq_ignore_ascii_case("and") {
            if !current.is_empty() {
                parts.push(normalize_answer_text(&current.join(" ")));
                current.clear();
            }
        } else {
            current.push(word);
        }
    }

    if !current.is_empty() {
        parts.push(normalize_answer_text(&current.join(" ")));
    }

    parts
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

fn parse_question_type(raw: &str, index: usize) -> Result<QuestionType, AppError> {
    match raw.trim().to_ascii_lowercase().as_str() {
        "single" => Ok(QuestionType::Single),
        "multiple" => Ok(QuestionType::Multiple),
        "short" => Ok(QuestionType::Short),
        other => Err(AppError::InvalidInput(format!(
            "Question {} has unsupported question type: {}",
            index + 1,
            other
        ))),
    }
}

fn required_string(
    value: &Value,
    field: &str,
    index: usize,
    label: &str,
) -> Result<String, AppError> {
    value[field]
        .as_str()
        .map(str::trim)
        .filter(|raw| !raw.is_empty())
        .map(String::from)
        .ok_or_else(|| AppError::InvalidInput(format!("Question {} missing {}", index + 1, label)))
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
) -> Result<DiagnosisRound, AppError> {
    let resolved_note = crate::services::fs_service::resolve_note_path(data_dir, note_path).await?;
    let note_content =
        crate::services::fs_service::read_file_content(&resolved_note.to_string_lossy()).await?;
    let truncated_content = note_content_for_prompt(&note_content);

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
    let _ = tx.send(DiagnosisStreamEvent::Phase {
        phase: "requesting_model".to_string(),
    });
    let response_text = call_llm(
        &settings.llm,
        &prompt,
        0.3,
        LlmRunKind::DiagnosisInitial,
        data_dir,
    )
    .await?;
    let _ = tx.send(DiagnosisStreamEvent::Phase {
        phase: "parsing_response".to_string(),
    });
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

pub(crate) fn note_content_for_prompt(raw_content: &str) -> String {
    crate::services::note_service::extract_body_content(raw_content)
        .chars()
        .take(8000)
        .collect()
}

pub async fn diagnose_follow_up(
    data_dir: &Path,
    session: &mut DiagnosisSession,
    user_reply: &str,
    tx: UnboundedSender<DiagnosisStreamEvent>,
) -> Result<(), AppError> {
    let previous_diagnosis = serde_json::to_string(&session.conversation)
        .map_err(|e| AppError::Internal(format!("Failed to serialize diagnosis: {}", e)))?;

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
    let _ = tx.send(DiagnosisStreamEvent::Phase {
        phase: "requesting_model".to_string(),
    });
    let response_text = call_llm(
        &settings.llm,
        &prompt,
        0.3,
        LlmRunKind::DiagnosisFollowup,
        data_dir,
    )
    .await?;
    let _ = tx.send(DiagnosisStreamEvent::Phase {
        phase: "parsing_response".to_string(),
    });
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
) -> Result<DiagnosisReport, AppError> {
    let resolved_note =
        crate::services::fs_service::resolve_note_path(data_dir, &session.note_path).await?;
    let note_content =
        crate::services::fs_service::read_file_content(&resolved_note.to_string_lossy()).await?;
    let truncated_content = note_content_for_prompt(&note_content);

    let conversation_json = serde_json::to_string(&session.conversation)
        .map_err(|e| AppError::Internal(format!("Failed to serialize conversation: {}", e)))?;

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
    let response_text = call_llm(
        &settings.llm,
        &prompt,
        0.3,
        LlmRunKind::DiagnosisReport,
        data_dir,
    )
    .await?;
    parse_diagnosis_report(&response_text)
}

/// Whether the non-streaming LLM call should request `response_format:
/// json_object`. A probed capability wins; unprobed endpoints fall back to
/// the legacy base_url heuristic.
fn wants_json_mode(llm: &LlmConfig) -> bool {
    match llm.supports_response_format {
        Some(supported) => supported,
        None => llm.base_url.contains("openai") || llm.base_url.contains("localhost:11434"),
    }
}

const DEFAULT_SYSTEM_PROMPT: &str =
    "你是一位严格的学科导师。请用中文输出，并严格按 JSON 格式返回。";

/// Public entry point: performs the request AND records the run for
/// observability. The business logic lives in `llm_exchange`; this wrapper
/// adds timing, prompt/response file recording and the `llm_runs.jsonl` entry.
///
/// Recording is best effort — a failure to write the run record never fails the
/// actual LLM call.
pub(crate) async fn call_llm(
    settings: &LlmConfig,
    prompt: &str,
    temperature: f64,
    kind: LlmRunKind,
    data_dir: &Path,
) -> Result<String, AppError> {
    call_llm_with_system(
        settings,
        DEFAULT_SYSTEM_PROMPT,
        prompt,
        temperature,
        kind,
        data_dir,
    )
    .await
}

/// Same instrumentation as [`call_llm`], with an explicit system prompt.
/// Used by features whose output language must follow the source content
/// instead of the shared Chinese tutor persona.
pub(crate) async fn call_llm_with_system(
    settings: &LlmConfig,
    system_prompt: &str,
    prompt: &str,
    temperature: f64,
    kind: LlmRunKind,
    data_dir: &Path,
) -> Result<String, AppError> {
    let started = std::time::Instant::now();
    let mut record = LlmRunRecord {
        id: uuid::Uuid::new_v4().simple().to_string(),
        kind,
        ts_start: chrono::Local::now().to_rfc3339(),
        ts_end: String::new(),
        duration_ms: 0,
        model: settings.model.clone(),
        endpoint_host: endpoint_host_of(&settings.base_url),
        request_params: serde_json::json!({
            "temperature": temperature,
            "max_tokens": settings.max_tokens,
            "stream": false,
            "response_format": wants_json_mode(settings),
        }),
        usage: None,
        usage_source: LlmUsageSource::Unavailable,
        http_status: None,
        error: None,
        prompt_file: None,
        response_file: None,
    };
    let mut raw_body: Option<String> = None;

    let outcome = llm_exchange(
        settings,
        system_prompt,
        prompt,
        temperature,
        &mut record,
        &mut raw_body,
    )
    .await;

    record.ts_end = chrono::Local::now().to_rfc3339();
    record.duration_ms = started.elapsed().as_millis() as u64;
    if let Err(e) = &outcome {
        record.error = Some(e.to_string());
    }
    if settings.record_llm_io {
        record.prompt_file =
            llm_run_store::write_debug_file(data_dir, kind.as_str(), "prompt", prompt);
        if let Some(raw) = raw_body.as_deref() {
            record.response_file = save_llm_debug_log(data_dir, kind.as_str(), raw);
        }
    }
    let _ = llm_run_store::record_llm_run(data_dir, &record);
    outcome
}

/// The actual HTTP exchange. Populates `record.http_status` / `record.usage` and
/// `raw_body` as it goes; never writes files or records runs itself.
async fn llm_exchange(
    settings: &LlmConfig,
    system_prompt: &str,
    prompt: &str,
    temperature: f64,
    record: &mut LlmRunRecord,
    raw_body: &mut Option<String>,
) -> Result<String, AppError> {
    let client = http_client();

    let mut request_body = serde_json::json!({
        "model": settings.model,
        "messages": [
            { "role": "system", "content": system_prompt },
            { "role": "user", "content": prompt }
        ],
        "stream": false,
        "temperature": temperature,
        "max_tokens": settings.max_tokens,
    });

    if wants_json_mode(settings) {
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
        .map_err(|e| AppError::Llm(format!("LLM API request failed: {}", e)))?;

    record.http_status = Some(response.status().as_u16());

    if !response.status().is_success() {
        let status = response.status();
        let body = response.text().await.unwrap_or_default();
        *raw_body = Some(body.clone());
        return Err(AppError::Llm(format!("LLM API error {}: {}", status, body)));
    }

    // Read as text first so the raw body can be recorded, then parse.
    let body_text = response
        .text()
        .await
        .map_err(|e| AppError::Llm(format!("Failed to read LLM response body: {}", e)))?;
    *raw_body = Some(body_text.clone());

    let json: Value = serde_json::from_str(&body_text)
        .map_err(|e| AppError::Llm(format!("Failed to parse LLM response: {}", e)))?;

    let (usage, usage_source) = parse_usage(&json);
    record.usage = usage;
    record.usage_source = usage_source;

    json["choices"][0]["message"]["content"]
        .as_str()
        .map(String::from)
        .ok_or_else(|| AppError::Llm("No content in LLM response".to_string()))
}

/// Extract token usage from an OpenAI-style response body. Returns
/// `(None, Unavailable)` when the endpoint did not report usage.
fn parse_usage(json: &Value) -> (Option<LlmUsage>, LlmUsageSource) {
    let usage = json.get("usage").filter(|u| u.is_object());
    match usage {
        Some(u) => {
            let parsed = LlmUsage {
                prompt_tokens: u["prompt_tokens"].as_u64().map(|v| v as u32),
                completion_tokens: u["completion_tokens"].as_u64().map(|v| v as u32),
                total_tokens: u["total_tokens"].as_u64().map(|v| v as u32),
            };
            if parsed.is_empty() {
                (None, LlmUsageSource::Unavailable)
            } else {
                (Some(parsed), LlmUsageSource::Body)
            }
        }
        None => (None, LlmUsageSource::Unavailable),
    }
}

#[derive(Debug)]
struct InitialDiagnosis {
    answer_analysis: String,
    blind_spots: Vec<BlindSpot>,
    follow_up_question: Option<String>,
}

fn parse_diagnosis_initial(raw: &str) -> Result<InitialDiagnosis, AppError> {
    let json_str = extract_json_block(raw);
    let parsed: Value = serde_json::from_str(&json_str).map_err(|e| {
        AppError::Internal(format!(
            "Failed to parse diagnosis: {}. Raw: {}",
            e,
            raw_preview(raw)
        ))
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

fn parse_follow_up(raw: &str) -> Result<FollowUpResponse, AppError> {
    let json_str = extract_json_block(raw);
    let parsed: Value = serde_json::from_str(&json_str)
        .map_err(|e| AppError::Internal(format!("Failed to parse follow-up: {}", e)))?;
    let should_continue = parsed["should_continue"].as_bool().ok_or_else(|| {
        AppError::InvalidInput("Diagnosis response missing should_continue".to_string())
    })?;
    let follow_up_question = string_field(&parsed, "follow_up_question").unwrap_or_default();

    if should_continue && follow_up_question.is_empty() {
        return Err(AppError::InvalidInput(
            "Diagnosis response missing follow_up_question".to_string(),
        ));
    }

    Ok(FollowUpResponse {
        progress_assessment: required_diagnosis_string(&parsed, "progress_assessment")?,
        new_blind_spots: parse_blind_spots_array(&parsed["new_blind_spots"], "new_blind_spots")?,
        should_continue,
        follow_up_question,
    })
}

fn parse_diagnosis_report(raw: &str) -> Result<DiagnosisReport, AppError> {
    let json_str = extract_json_block(raw);
    let parsed: Value = serde_json::from_str(&json_str)
        .map_err(|e| AppError::Internal(format!("Failed to parse report: {}", e)))?;
    let next_steps = parse_non_empty_string_array(&parsed["next_steps"], "next_steps")?;

    Ok(DiagnosisReport {
        summary: required_diagnosis_string(&parsed, "summary")?,
        blind_spots: parse_blind_spots_array(&parsed["blind_spots"], "blind_spots")?,
        overall_level: required_diagnosis_string(&parsed, "overall_level")?,
        next_steps,
    })
}

fn parse_blind_spot(value: &Value, label: &str) -> Result<BlindSpot, AppError> {
    let tag = string_field(value, "tag")
        .or_else(|| string_field(value, "type"))
        .or_else(|| string_field(value, "concept"))
        .ok_or_else(|| {
            AppError::InvalidInput(format!("Diagnosis response missing {} tag", label))
        })?;
    let description = string_field(value, "description").ok_or_else(|| {
        AppError::InvalidInput(format!("Diagnosis response missing {} description", label))
    })?;

    Ok(BlindSpot {
        tag,
        severity: string_field(value, "severity").unwrap_or_else(|| "medium".to_string()),
        description,
        note_reference: string_field(value, "note_reference").unwrap_or_default(),
        suggestion: string_field(value, "suggestion").unwrap_or_default(),
    })
}

fn parse_initial_blind_spots(parsed: &Value) -> Result<Vec<BlindSpot>, AppError> {
    if let Some(values) = parsed["blind_spots"].as_array() {
        let spots = values
            .iter()
            .enumerate()
            .map(|(index, value)| parse_blind_spot(value, &format!("blind_spots[{}]", index + 1)))
            .collect::<Result<Vec<_>, _>>()?;

        if spots.is_empty() {
            Err(AppError::InvalidInput(
                "Diagnosis response missing blind_spots".to_string(),
            ))
        } else {
            Ok(spots)
        }
    } else {
        parse_blind_spot(&parsed["blind_spot"], "blind_spot").map(|blind_spot| vec![blind_spot])
    }
}

fn parse_blind_spots_array(value: &Value, label: &str) -> Result<Vec<BlindSpot>, AppError> {
    let Some(values) = value.as_array() else {
        return Ok(vec![]);
    };

    values
        .iter()
        .enumerate()
        .map(|(index, value)| parse_blind_spot(value, &format!("{}[{}]", label, index + 1)))
        .collect()
}

fn required_diagnosis_string(value: &Value, field: &str) -> Result<String, AppError> {
    value[field]
        .as_str()
        .map(str::trim)
        .filter(|raw| !raw.is_empty())
        .map(String::from)
        .ok_or_else(|| AppError::InvalidInput(format!("Diagnosis response missing {}", field)))
}

fn string_field(value: &Value, field: &str) -> Option<String> {
    value[field]
        .as_str()
        .map(str::trim)
        .filter(|raw| !raw.is_empty())
        .map(String::from)
}

fn parse_non_empty_string_array(value: &Value, field: &str) -> Result<Vec<String>, AppError> {
    let items = value
        .as_array()
        .ok_or_else(|| AppError::InvalidInput(format!("Diagnosis response missing {}", field)))?
        .iter()
        .filter_map(|item| item.as_str())
        .map(str::trim)
        .filter(|item| !item.is_empty())
        .map(String::from)
        .collect::<Vec<_>>();

    if items.is_empty() {
        Err(AppError::InvalidInput(format!(
            "Diagnosis response missing {}",
            field
        )))
    } else {
        Ok(items)
    }
}

/// Record a completed streaming quiz generation run for observability.
///
/// The streaming path does not go through `call_llm`, so it assembles its own
/// `LlmRunRecord`. Best effort — a failure here never fails generation.
#[allow(clippy::too_many_arguments)]
fn record_quiz_stream_run(
    data_dir: &Path,
    llm: &LlmConfig,
    prompt: &str,
    started: std::time::Instant,
    ts_start: &str,
    http_status: Option<u16>,
    usage: Option<LlmUsage>,
    usage_source: LlmUsageSource,
    error: Option<String>,
    raw: Option<&str>,
    record_io: bool,
) {
    let mut record = LlmRunRecord {
        id: uuid::Uuid::new_v4().simple().to_string(),
        kind: LlmRunKind::Quiz,
        ts_start: ts_start.to_string(),
        ts_end: chrono::Local::now().to_rfc3339(),
        duration_ms: started.elapsed().as_millis() as u64,
        model: llm.model.clone(),
        endpoint_host: endpoint_host_of(&llm.base_url),
        request_params: serde_json::json!({
            "temperature": llm.temperature,
            "max_tokens": llm.max_tokens,
            "stream": true,
            "stream_options": { "include_usage": true },
        }),
        usage,
        usage_source,
        http_status,
        error,
        prompt_file: None,
        response_file: None,
    };
    if record_io {
        record.prompt_file =
            llm_run_store::write_debug_file(data_dir, LlmRunKind::Quiz.as_str(), "prompt", prompt);
        if let Some(raw) = raw {
            record.response_file = save_llm_debug_log(data_dir, LlmRunKind::Quiz.as_str(), raw);
        }
    }
    let _ = llm_run_store::record_llm_run(data_dir, &record);
}

/// Persist a raw LLM response under `debug/` and return its bare filename.
///
/// Delegates to `llm_run_store::write_debug_file` so naming and retention are
/// shared with the prompt files that `llm_run_store` writes. `None` means the
/// file could not be written (best effort — never fails the caller).
fn save_llm_debug_log(data_dir: &Path, kind: &str, raw: &str) -> Option<String> {
    crate::services::llm_run_store::write_debug_file(data_dir, kind, "raw", raw)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn temp_data_dir(test_name: &str) -> PathBuf {
        let dir = std::env::temp_dir()
            .join("knowtequiz-quiz-engine-tests")
            .join(test_name)
            .join(uuid::Uuid::new_v4().to_string());
        std::fs::create_dir_all(&dir).expect("test temp dir should be created");
        dir
    }

    fn write_settings_with_root(data_dir: &Path, root: &Path) {
        let settings = serde_json::json!({
            "version": "1.0.0",
            "theme": "obsidian-dark",
            "ui_language": "zh-CN",
            "llm": {
                "provider": "openai-compatible",
                "base_url": "http://localhost:11434/v1",
                "api_key": "",
                "model": "qwen2.5:7b",
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
            },
            "workspace": {
                "root_path": root.to_string_lossy()
            }
        });
        std::fs::write(data_dir.join("settings.json"), settings.to_string()).unwrap();
    }

    #[tokio::test]
    async fn generate_quiz_stream_rejects_paths_outside_note_root() {
        let data_dir = temp_data_dir("generate_quiz_stream_rejects_paths_outside_data");
        let root = temp_data_dir("generate_quiz_stream_rejects_paths_outside_notes");
        let outside = temp_data_dir("generate_quiz_stream_rejects_paths_outside_outside");
        std::fs::write(outside.join("secret.md"), "# secret").unwrap();
        write_settings_with_root(&data_dir, &root);

        let params = QuizStreamParams {
            path: outside.join("secret.md").to_string_lossy().to_string(),
            types: vec![QuestionType::Single],
            count: 5,
            difficulty: "medium".to_string(),
            lang: "zh".to_string(),
        };
        let (tx, _rx) = tokio::sync::mpsc::unbounded_channel();

        let result = generate_quiz_stream(&data_dir, &params, tx).await;

        let err = result.expect_err("path outside the note root should be rejected");
        assert!(err.to_string().contains("outside the configured note root"));
    }

    #[tokio::test]
    async fn generate_quiz_stream_rejects_missing_files_inside_root() {
        let data_dir = temp_data_dir("generate_quiz_stream_rejects_missing_data");
        let root = temp_data_dir("generate_quiz_stream_rejects_missing_notes");
        write_settings_with_root(&data_dir, &root);

        let params = QuizStreamParams {
            path: root.join("no-such-note.md").to_string_lossy().to_string(),
            types: vec![QuestionType::Single],
            count: 5,
            difficulty: "medium".to_string(),
            lang: "zh".to_string(),
        };
        let (tx, _rx) = tokio::sync::mpsc::unbounded_channel();

        let result = generate_quiz_stream(&data_dir, &params, tx).await;

        let err = result.expect_err("missing note should be reported");
        assert!(err.to_string().contains("File does not exist"));
    }

    #[test]
    fn debug_log_writes_raw_response_to_debug_directory() {
        let dir = temp_data_dir("debug_log_writes_raw_response");
        let raw = r#"{"questions": [{"question": "Q?", "answer": "A"}]}"#;

        save_llm_debug_log(&dir, "quiz", raw);

        let debug_dir = dir.join("debug");
        assert!(debug_dir.exists(), "debug directory should be created");

        let entries: Vec<_> = std::fs::read_dir(&debug_dir)
            .expect("should read debug dir")
            .filter_map(|e| e.ok())
            .collect();
        assert_eq!(entries.len(), 1, "one debug file should be written");

        let content = std::fs::read_to_string(entries[0].path()).expect("should read debug file");
        assert_eq!(content, raw);
    }

    #[test]
    fn debug_log_does_not_panic_when_data_dir_is_readonly() {
        // Use a path that can't be created (e.g., inside a file)
        let dir = std::path::Path::new("/nonexistent/readonly/debug_test");
        save_llm_debug_log(dir, "quiz", "test"); // should not panic
    }

    #[test]
    fn raw_preview_never_splits_multibyte_characters() {
        // 67 CJK chars = 201 bytes; a naive 200-byte cut splits the last char.
        let raw = "汉".repeat(67);
        assert_eq!(raw.len(), 201);
        assert_eq!(raw_preview(&raw), "汉".repeat(66));
        assert_eq!(raw_preview("short"), "short");
    }

    #[test]
    fn parse_quiz_response_with_long_chinese_garbage_errors_without_panicking() {
        let raw = format!("模型输出不是JSON：{}", "汉".repeat(120));
        let err = parse_quiz_response(&raw).expect_err("garbage input should fail to parse");
        assert!(err.to_string().contains("JSON parse error"));
        assert!(err.to_string().contains("汉"));
    }

    #[test]
    fn parse_diagnosis_initial_with_long_chinese_garbage_errors_without_panicking() {
        let raw = format!("诊断输出不是JSON：{}", "汉".repeat(120));
        let err = parse_diagnosis_initial(&raw).expect_err("garbage input should fail to parse");
        assert!(err.to_string().contains("Failed to parse diagnosis"));
    }

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
    fn wants_json_mode_prefers_probed_capability_over_url_heuristic() {
        let mut llm = LlmConfig {
            provider: "openai-compatible".to_string(),
            base_url: "http://127.0.0.1:1234/v1".to_string(),
            api_key: String::new(),
            model: "test".to_string(),
            max_tokens: 1024,
            temperature: 0.7,
            supports_response_format: None,
            record_llm_io: true,
            verified_at: None,
        };
        // Unprobed: legacy URL heuristic (non-ollama, non-openai host).
        assert!(!wants_json_mode(&llm));

        llm.supports_response_format = Some(true);
        assert!(wants_json_mode(&llm));

        llm.supports_response_format = Some(false);
        assert!(!wants_json_mode(&llm));

        // Unprobed but heuristic-matching host.
        llm.supports_response_format = None;
        llm.base_url = "http://localhost:11434/v1".to_string();
        assert!(wants_json_mode(&llm));
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
    fn parse_quiz_response_accepts_single_choice_answer_text_with_and() {
        let raw = r#"{
            "questions": [
                {
                    "id": "q1",
                    "question_type": "single",
                    "question": "Which department owns this?",
                    "options": ["A. Research and Development", "B. Operations"],
                    "answer": "Research and Development",
                    "explanation": "R&D owns this."
                }
            ]
        }"#;

        let questions =
            parse_quiz_response(raw).expect("single choice option text with and should parse");

        assert_eq!(questions.len(), 1);
        assert_eq!(questions[0].answer, "Research and Development");
    }

    #[test]
    fn parse_quiz_response_accepts_single_choice_answer_text_with_comma() {
        let raw = r#"{
            "questions": [
                {
                    "id": "q1",
                    "question_type": "single",
                    "question": "Which label is correct?",
                    "options": ["A. Research, Development", "B. Operations"],
                    "answer": "Research, Development",
                    "explanation": "The comma is part of the label."
                }
            ]
        }"#;

        let questions =
            parse_quiz_response(raw).expect("single choice option text with comma should parse");

        assert_eq!(questions.len(), 1);
        assert_eq!(questions[0].answer, "Research, Development");
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
    fn parse_quiz_response_accepts_multiple_choice_answer_text_joined_by_and() {
        let raw = r#"{
            "questions": [
                {
                    "id": "q1",
                    "question_type": "multiple",
                    "question": "Which claims are true?",
                    "options": ["A. Alpha", "B. Beta", "C. Gamma"],
                    "answer": "Alpha and Gamma",
                    "explanation": "Alpha and Gamma are true."
                }
            ]
        }"#;

        let questions = parse_quiz_response(raw)
            .expect("multiple choice answer text joined by and should parse");

        assert_eq!(questions.len(), 1);
        assert_eq!(questions[0].answer, "Alpha and Gamma");
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
    fn note_content_for_prompt_strips_frontmatter_before_prompting() {
        let content = note_content_for_prompt(
            "---\ntitle: Rust Ownership\ntags: rust\n---\n\n# Ownership\n\nMoves transfer values.",
        );

        assert_eq!(content, "# Ownership\n\nMoves transfer values.");
        assert!(!content.contains("tags: rust"));
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

    #[test]
    fn diagnosis_stream_event_wire_names_match_frontend() {
        use crate::services::quiz_engine::DiagnosisStreamEvent;

        let phase = DiagnosisStreamEvent::Phase {
            phase: "requesting_model".to_string(),
        };
        let json = serde_json::to_string(&phase).unwrap();
        assert!(json.contains(r#""event":"phase""#), "got: {json}");

        let initial = DiagnosisStreamEvent::Initial {
            role: "ai".to_string(),
            content: String::new(),
            blind_spots: vec![],
            follow_up: None,
        };
        let json = serde_json::to_string(&initial).unwrap();
        assert!(json.contains(r#""event":"initial""#), "got: {json}");

        let follow_up = DiagnosisStreamEvent::FollowUp {
            question: "q".to_string(),
            blind_spots: vec![],
        };
        let json = serde_json::to_string(&follow_up).unwrap();
        assert!(json.contains(r#""event":"follow_up""#), "got: {json}");

        let report = DiagnosisStreamEvent::Report {
            summary: String::new(),
            blind_spots: vec![],
            overall_level: String::new(),
            next_steps: vec![],
        };
        let json = serde_json::to_string(&report).unwrap();
        assert!(json.contains(r#""event":"report""#), "got: {json}");

        let error = DiagnosisStreamEvent::Error {
            message: String::new(),
        };
        let json = serde_json::to_string(&error).unwrap();
        assert!(json.contains(r#""event":"error""#), "got: {json}");
    }
}
