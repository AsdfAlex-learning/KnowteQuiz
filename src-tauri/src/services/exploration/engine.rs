use crate::errors::AppError;
use crate::models::exploration::{
    Concept, Domain, ExplorationSession, KnowledgeGraph, NextAction, NoteDigest, Question,
    SessionStatus, TurnGroup, TurnResult, Verdict, WhiteboardStep,
};
use crate::models::llm_run::LlmRunKind;
use crate::services::exploration::{graph, prompt, session_store};
use crate::services::{config, fs_service, quiz_engine};
use serde_json::Value;
use std::collections::HashSet;
use std::path::Path;
use tokio::sync::mpsc::UnboundedSender;

/// Events streamed to the UI while a session initializes (and, later, while a
/// turn runs). Shape matches the quiz/diagnosis streams: `{event, data}`.
#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "snake_case", tag = "event", content = "data")]
pub enum ExplorationStreamEvent {
    Phase {
        phase: String,
    },
    DomainWarning {
        domains: Vec<Domain>,
        warning: String,
    },
    GraphReady {
        graph: KnowledgeGraph,
    },
    Error {
        message: String,
    },
    Done {
        session_id: String,
    },
}

fn send(tx: &UnboundedSender<ExplorationStreamEvent>, event: ExplorationStreamEvent) {
    let _ = tx.send(event);
}

/// Map-reduce initialization: read every source note, digest each one, then
/// merge the digests into one cross-document graph.
///
/// Re-running on an already-initialized session is a no-op that replays the
/// saved state, so a retry after a network failure is always safe.
pub async fn initialize_session(
    data_dir: &Path,
    session_id: &str,
    tx: UnboundedSender<ExplorationStreamEvent>,
) -> Result<(), AppError> {
    let mut session = session_store::load_session(data_dir, session_id)?;

    if !session.graph.concepts.is_empty() {
        replay_graph(&session, &tx);
        send(
            &tx,
            ExplorationStreamEvent::Done {
                session_id: session.id.clone(),
            },
        );
        return Ok(());
    }

    let settings = config::get_settings_path(data_dir)?;
    if settings.llm.base_url.trim().is_empty() {
        return Err(AppError::InvalidInput(
            "No LLM endpoint configured".to_string(),
        ));
    }

    send(
        &tx,
        ExplorationStreamEvent::Phase {
            phase: "reading".to_string(),
        },
    );

    let mut inputs: Vec<(String, String)> = Vec::new();
    for path in &session.source_paths {
        let resolved = fs_service::resolve_note_path(data_dir, path).await?;
        let content = fs_service::read_file_content(&resolved.to_string_lossy()).await?;
        inputs.push((path.clone(), content));
    }

    let mut digests: Vec<NoteDigest> = Vec::with_capacity(inputs.len());
    for (index, (path, content)) in inputs.iter().enumerate() {
        send(
            &tx,
            ExplorationStreamEvent::Phase {
                phase: format!("summarizing:{}/{}", index + 1, inputs.len()),
            },
        );
        digests.push(summarize_note(&settings.llm, data_dir, path, content).await);
    }

    send(
        &tx,
        ExplorationStreamEvent::Phase {
            phase: "merging".to_string(),
        },
    );

    let digests_json = serde_json::to_string(&digests)
        .map_err(|e| AppError::Internal(format!("Failed to serialize digests: {e}")))?;
    let raw = quiz_engine::call_llm_with_system(
        &settings.llm,
        prompt::EXPLORATION_SYSTEM_PROMPT,
        &prompt::graph_prompt(&digests_json),
        0.2,
        LlmRunKind::ExplorationGraph,
        data_dir,
    )
    .await?;
    let value = graph::parse_json_object(&raw)?;
    let mut knowledge = graph::normalize_graph(&value, &session.source_paths)?;
    if graph::first_concept_index(&knowledge).is_none() {
        return Err(AppError::Llm(
            "Knowledge graph has no startable concept".to_string(),
        ));
    }
    let warning = graph::domain_warning(&knowledge);

    session.source_digests = digests;
    session.domain_warning = warning.clone();
    knowledge.domains_confirmed = warning.is_none();
    session.graph = knowledge;
    session.status = if warning.is_some() {
        SessionStatus::AwaitingDomainConfirm
    } else {
        SessionStatus::Active
    };
    session.updated_at = chrono::Local::now().to_rfc3339();
    session_store::save_session(data_dir, &session)?;

    if session.status == SessionStatus::Active {
        match ensure_current_question(data_dir, &mut session).await {
            Ok(()) => {
                session_store::save_session(data_dir, &session)?;
            }
            // The graph is already saved; report the question failure but keep
            // the session usable so the learner can retry.
            Err(err) => send(
                &tx,
                ExplorationStreamEvent::Error {
                    message: err.to_string(),
                },
            ),
        }
    }

    replay_graph(&session, &tx);
    send(
        &tx,
        ExplorationStreamEvent::Done {
            session_id: session.id.clone(),
        },
    );
    Ok(())
}

fn replay_graph(session: &ExplorationSession, tx: &UnboundedSender<ExplorationStreamEvent>) {
    match &session.domain_warning {
        Some(warning) => send(
            tx,
            ExplorationStreamEvent::DomainWarning {
                domains: session.graph.domains.clone(),
                warning: warning.clone(),
            },
        ),
        None => send(
            tx,
            ExplorationStreamEvent::GraphReady {
                graph: session.graph.clone(),
            },
        ),
    }
}

/// Applies the learner's domain confirmation and activates the session.
pub async fn confirm_domains(
    data_dir: &Path,
    session_id: &str,
    domains: Vec<Domain>,
    tx: UnboundedSender<ExplorationStreamEvent>,
) -> Result<(), AppError> {
    let mut session = session_store::load_session(data_dir, session_id)?;
    if session.graph.concepts.is_empty() {
        return Err(AppError::InvalidInput(
            "Session has no knowledge graph yet".to_string(),
        ));
    }

    session.graph.domains = sanitize_domains(&session.graph, domains)?;
    session.graph.domains_confirmed = true;
    session.domain_warning = None;
    session.status = SessionStatus::Active;
    session.updated_at = chrono::Local::now().to_rfc3339();
    session_store::save_session(data_dir, &session)?;

    match ensure_current_question(data_dir, &mut session).await {
        Ok(()) => {
            session_store::save_session(data_dir, &session)?;
        }
        Err(err) => send(
            &tx,
            ExplorationStreamEvent::Error {
                message: err.to_string(),
            },
        ),
    }

    send(
        &tx,
        ExplorationStreamEvent::GraphReady {
            graph: session.graph.clone(),
        },
    );
    send(
        &tx,
        ExplorationStreamEvent::Done {
            session_id: session.id.clone(),
        },
    );
    Ok(())
}

fn sanitize_domains(graph: &KnowledgeGraph, domains: Vec<Domain>) -> Result<Vec<Domain>, AppError> {
    let known: HashSet<&str> = graph.concepts.iter().map(|c| c.id.as_str()).collect();
    let mut sanitized: Vec<Domain> = Vec::new();

    for domain in domains {
        let mut concept_ids: Vec<String> = Vec::new();
        for id in domain.concept_ids {
            if known.contains(id.as_str()) && !concept_ids.contains(&id) {
                concept_ids.push(id);
            }
        }
        if concept_ids.is_empty() {
            continue;
        }
        let title = if domain.title.trim().is_empty() {
            format!("Domain {}", sanitized.len() + 1)
        } else {
            domain.title.trim().to_string()
        };
        sanitized.push(Domain {
            id: format!("d{}", sanitized.len() + 1),
            title,
            concept_ids,
        });
    }

    if sanitized.is_empty() {
        return Err(AppError::InvalidInput(
            "At least one domain is required".to_string(),
        ));
    }
    Ok(sanitized)
}

/// Generates the pending question when the session is active but has none.
/// Returns the unchanged session when a question is already pending.
pub async fn next_question(
    data_dir: &Path,
    session_id: &str,
) -> Result<ExplorationSession, AppError> {
    let mut session = session_store::load_session(data_dir, session_id)?;
    if session.status != SessionStatus::Active {
        return Ok(session);
    }
    if session.active_group < session.groups.len() {
        return Ok(session);
    }

    ensure_current_question(data_dir, &mut session).await?;
    session.updated_at = chrono::Local::now().to_rfc3339();
    session_store::save_session(data_dir, &session)?;
    Ok(session)
}

/// Applies one learner answer and advances the timeline.
pub async fn ask_turn(
    data_dir: &Path,
    session_id: &str,
    answer: &str,
) -> Result<ExplorationSession, AppError> {
    let answer = answer.trim();
    if answer.is_empty() {
        return Err(AppError::InvalidInput(
            "Answer must not be empty".to_string(),
        ));
    }

    let mut session = session_store::load_session(data_dir, session_id)?;
    match session.status {
        SessionStatus::Active => {}
        SessionStatus::Completed => {
            return Err(AppError::InvalidInput(
                "This session is already completed".to_string(),
            ))
        }
        _ => {
            return Err(AppError::InvalidInput(
                "Session is not ready for answers".to_string(),
            ))
        }
    }
    if session.graph.concepts.is_empty() {
        return Err(AppError::InvalidInput(
            "Session has no knowledge graph yet".to_string(),
        ));
    }
    if session.active_group >= session.groups.len() {
        return Err(AppError::InvalidInput(
            "No question is pending; generate the next question first".to_string(),
        ));
    }

    let settings = config::get_settings_path(data_dir)?;
    if settings.llm.base_url.trim().is_empty() {
        return Err(AppError::InvalidInput(
            "No LLM endpoint configured".to_string(),
        ));
    }

    let current_index = session.active_group;
    // Re-answering after /undo replaces the discarded future.
    session.groups.truncate(current_index + 1);
    session
        .asides
        .retain(|aside| aside.group_index <= current_index);
    let concept_id = session.groups[current_index].concept_id.clone();
    let concept = session
        .graph
        .concepts
        .iter()
        .find(|concept| concept.id == concept_id)
        .cloned()
        .ok_or_else(|| AppError::Internal("Current concept is missing".to_string()))?;

    let context = turn_context(&session, &concept, answer);
    let context_json = serde_json::to_string(&context)
        .map_err(|e| AppError::Internal(format!("Failed to serialize turn context: {e}")))?;
    let raw = quiz_engine::call_llm_with_system(
        &settings.llm,
        prompt::EXPLORATION_SYSTEM_PROMPT,
        &prompt::turn_prompt(&context_json),
        0.4,
        LlmRunKind::ExplorationTurn,
        data_dir,
    )
    .await?;

    let value = match graph::parse_json_object(&raw) {
        Ok(value) => value,
        Err(error) => {
            let repaired = quiz_engine::call_llm_with_system(
                &settings.llm,
                prompt::EXPLORATION_SYSTEM_PROMPT,
                &prompt::repair_prompt(&raw, &error.to_string()),
                0.1,
                LlmRunKind::ExplorationRepair,
                data_dir,
            )
            .await?;
            graph::parse_json_object(&repaired)?
        }
    };

    let response = parse_turn(&value)?;
    apply_turn(&mut session, answer, response);
    session.updated_at = chrono::Local::now().to_rfc3339();
    session_store::save_session(data_dir, &session)?;
    Ok(session)
}

/// Moves the cursor back one step. The answered data is kept so `/redo` can
/// restore the step; coverage is derived from the cursor, so it rolls back too.
pub fn undo(data_dir: &Path, session_id: &str) -> Result<ExplorationSession, AppError> {
    let mut session = session_store::load_session(data_dir, session_id)?;
    if session.active_group == 0 {
        return Ok(session);
    }
    session.active_group -= 1;
    if session.status == SessionStatus::Completed {
        session.status = SessionStatus::Active;
        session.completed_early = false;
    }
    session.updated_at = chrono::Local::now().to_rfc3339();
    session_store::save_session(data_dir, &session)?;
    Ok(session)
}

/// Moves the cursor forward through the retained tail.
pub fn redo(data_dir: &Path, session_id: &str) -> Result<ExplorationSession, AppError> {
    let mut session = session_store::load_session(data_dir, session_id)?;
    if session.status == SessionStatus::Completed {
        return Ok(session);
    }
    if session.active_group + 1 >= session.groups.len() {
        return Ok(session);
    }
    session.active_group += 1;
    session.updated_at = chrono::Local::now().to_rfc3339();
    session_store::save_session(data_dir, &session)?;
    Ok(session)
}

/// Ends the session without covering every concept.
pub fn finish_early(data_dir: &Path, session_id: &str) -> Result<ExplorationSession, AppError> {
    let mut session = session_store::load_session(data_dir, session_id)?;
    if session.status == SessionStatus::Completed {
        return Ok(session);
    }
    session.status = SessionStatus::Completed;
    session.completed_early = true;
    session.updated_at = chrono::Local::now().to_rfc3339();
    session_store::save_session(data_dir, &session)?;
    Ok(session)
}

/// Skips the current concept: it stays uncovered, but the path moves on.
pub async fn skip_concept(
    data_dir: &Path,
    session_id: &str,
) -> Result<ExplorationSession, AppError> {
    let mut session = session_store::load_session(data_dir, session_id)?;
    if session.status != SessionStatus::Active || session.active_group >= session.groups.len() {
        return Ok(session);
    }

    let concept_id = session.groups[session.active_group].concept_id.clone();
    if !session.skipped.contains(&concept_id) {
        session.skipped.push(concept_id);
    }
    // Drop the pending question so the transcript never shows an unanswered
    // question that was explicitly skipped.
    session.groups.truncate(session.active_group);
    session
        .asides
        .retain(|aside| aside.group_index < session.active_group);

    ensure_current_question(data_dir, &mut session).await?;
    session.updated_at = chrono::Local::now().to_rfc3339();
    session_store::save_session(data_dir, &session)?;
    Ok(session)
}

/// `/btw`: answers a side question without advancing the path.
pub async fn ask_aside(
    data_dir: &Path,
    session_id: &str,
    question: &str,
) -> Result<ExplorationSession, AppError> {
    let question = question.trim();
    if question.is_empty() {
        return Err(AppError::InvalidInput(
            "Question must not be empty".to_string(),
        ));
    }

    let mut session = session_store::load_session(data_dir, session_id)?;
    if session.graph.concepts.is_empty() {
        return Err(AppError::InvalidInput(
            "Session has no knowledge graph yet".to_string(),
        ));
    }
    let settings = config::get_settings_path(data_dir)?;
    if settings.llm.base_url.trim().is_empty() {
        return Err(AppError::InvalidInput(
            "No LLM endpoint configured".to_string(),
        ));
    }

    let current_concept = session
        .groups
        .get(session.active_group)
        .and_then(|group| {
            session
                .graph
                .concepts
                .iter()
                .find(|concept| concept.id == group.concept_id)
        })
        .cloned();
    let context = serde_json::json!({
        "current_concept": current_concept,
        "recent_turns": session
            .active_groups()
            .iter()
            .rev()
            .take(3)
            .rev()
            .map(|group| serde_json::json!({
                "question": group.question.prompt_md,
                "answer": group.answer,
            }))
            .collect::<Vec<_>>(),
        "note_summaries": session
            .source_digests
            .iter()
            .map(|digest| serde_json::json!({
                "path": digest.path,
                "title": digest.title,
                "summary": digest.summary,
            }))
            .collect::<Vec<_>>(),
        "question": question,
    });
    let context_json = serde_json::to_string(&context)
        .map_err(|e| AppError::Internal(format!("Failed to serialize aside context: {e}")))?;
    let raw = quiz_engine::call_llm_with_system(
        &settings.llm,
        prompt::EXPLORATION_SYSTEM_PROMPT,
        &prompt::aside_prompt(&context_json),
        0.3,
        LlmRunKind::ExplorationAside,
        data_dir,
    )
    .await?;
    let answer_md = clean_question(&raw);
    if answer_md.is_empty() {
        return Err(AppError::Llm("Model returned an empty answer".to_string()));
    }

    let now = chrono::Local::now().to_rfc3339();
    session.asides.push(crate::models::exploration::AsideTurn {
        id: uuid::Uuid::new_v4().simple().to_string(),
        question: question.to_string(),
        answer_md,
        group_index: session.active_group,
        created_at: now.clone(),
    });
    session.updated_at = now;
    session_store::save_session(data_dir, &session)?;
    Ok(session)
}

struct TurnResponse {
    verdict: Verdict,
    feedback_md: String,
    next_action: NextAction,
    hint: Option<String>,
    next_concept_id: Option<String>,
    next_question: Option<String>,
    whiteboard_md: Option<String>,
    covered_concepts: Vec<String>,
}

fn parse_turn(value: &Value) -> Result<TurnResponse, AppError> {
    let verdict = match value
        .get("verdict")
        .and_then(|v| v.as_str())
        .unwrap_or_default()
    {
        "correct" => Verdict::Correct,
        "partial" => Verdict::Partial,
        "wrong" => Verdict::Wrong,
        "unknown" => Verdict::Unknown,
        other => {
            return Err(AppError::Llm(format!(
                "Turn response has an unknown verdict: {other:?}"
            )))
        }
    };
    let next_action = match value
        .get("next_action")
        .and_then(|v| v.as_str())
        .unwrap_or_default()
    {
        "question" => NextAction::Question,
        "hint" => NextAction::Hint,
        "advance" => NextAction::Advance,
        "complete" => NextAction::Complete,
        other => {
            return Err(AppError::Llm(format!(
                "Turn response has an unknown next_action: {other:?}"
            )))
        }
    };

    Ok(TurnResponse {
        verdict,
        feedback_md: value
            .get("feedback")
            .and_then(|v| v.as_str())
            .unwrap_or_default()
            .trim()
            .to_string(),
        next_action,
        hint: optional_string(value.get("hint")),
        next_concept_id: optional_string(value.get("next_concept_id")),
        next_question: optional_string(value.get("next_question")),
        whiteboard_md: optional_string(value.get("whiteboard_md")),
        covered_concepts: value
            .get("covered_concepts")
            .and_then(|v| v.as_array())
            .map(|items| {
                items
                    .iter()
                    .filter_map(|item| item.as_str())
                    .map(str::to_string)
                    .collect()
            })
            .unwrap_or_default(),
    })
}

fn optional_string(value: Option<&Value>) -> Option<String> {
    value
        .and_then(|v| v.as_str())
        .map(str::trim)
        .filter(|text| !text.is_empty())
        .map(str::to_string)
}

fn apply_turn(session: &mut ExplorationSession, answer: &str, response: TurnResponse) {
    let current_index = session.active_group;
    let concept_id = session.groups[current_index].concept_id.clone();
    let concept_title = session
        .graph
        .concepts
        .iter()
        .find(|concept| concept.id == concept_id)
        .map(|concept| concept.title.clone())
        .unwrap_or_default();

    let mut covered: Vec<String> = response
        .covered_concepts
        .into_iter()
        .filter(|id| session.graph.concepts.iter().any(|c| &c.id == id))
        .collect();

    let mut next_action = response.next_action;
    if next_action == NextAction::Hint && consecutive_hints(session, &concept_id) >= 3 {
        // Hint budget exhausted: move on instead of looping forever.
        next_action = NextAction::Advance;
    }
    if next_action == NextAction::Advance && !covered.contains(&concept_id) {
        covered.push(concept_id.clone());
    }

    let whiteboard = response.whiteboard_md.map(|content_md| WhiteboardStep {
        id: uuid::Uuid::new_v4().simple().to_string(),
        concept_id: concept_id.clone(),
        title: concept_title,
        content_md,
        group_index: current_index,
    });

    {
        let group = &mut session.groups[current_index];
        group.answer = Some(answer.to_string());
        group.result = Some(TurnResult {
            verdict: response.verdict,
            feedback_md: response.feedback_md,
            next_action,
            hint: response.hint,
        });
        group.whiteboard = whiteboard;
        group.covered_after = covered;
    }

    let mut covered_set: HashSet<String> = session.covered_concept_ids().into_iter().collect();
    // The current group is not yet part of `active_groups()`, so fold in the
    // coverage it just earned before deciding whether the session is finished.
    for id in &session.groups[current_index].covered_after {
        covered_set.insert(id.clone());
    }
    let all_covered = session
        .graph
        .concepts
        .iter()
        .all(|concept| covered_set.contains(&concept.id));
    if next_action == NextAction::Complete || all_covered {
        session.status = SessionStatus::Completed;
        return;
    }

    // Advance the cursor first so coverage lookups include the group we just
    // filled; otherwise the concept we just covered still looks uncovered.
    session.active_group = current_index + 1;

    // Only an explicit `advance` moves to another concept; every other action
    // re-asks the same one.
    let next_concept_id = if next_action == NextAction::Advance {
        response
            .next_concept_id
            .filter(|id| {
                !covered_set.contains(id)
                    && session
                        .graph
                        .concepts
                        .iter()
                        .any(|concept| &concept.id == id)
            })
            .or_else(|| next_uncovered_id(session))
            .unwrap_or_else(|| concept_id.clone())
    } else {
        concept_id.clone()
    };

    let fallback_prompt = session.groups[current_index].question.prompt_md.clone();
    let prompt_md = response.next_question.unwrap_or(fallback_prompt);
    session.groups.push(TurnGroup {
        concept_id: next_concept_id,
        question: Question {
            prompt_md,
            hint_used: next_action == NextAction::Hint,
        },
        answer: None,
        result: None,
        whiteboard: None,
        covered_after: Vec::new(),
    });
}

fn consecutive_hints(session: &ExplorationSession, concept_id: &str) -> u32 {
    // Includes the current (pending) group: a question created by a hint counts
    // as one hint already given.
    let end = (session.active_group + 1).min(session.groups.len());
    session.groups[..end]
        .iter()
        .rev()
        .take_while(|group| group.concept_id == concept_id && group.question.hint_used)
        .count() as u32
}

fn next_uncovered_id(session: &ExplorationSession) -> Option<String> {
    let covered: HashSet<String> = session.covered_concept_ids().into_iter().collect();
    for id in &session.graph.root_order {
        if !covered.contains(id)
            && !session.skipped.contains(id)
            && session.graph.concepts.iter().any(|c| &c.id == id)
        {
            return Some(id.clone());
        }
    }
    session
        .graph
        .concepts
        .iter()
        .find(|concept| !covered.contains(&concept.id) && !session.skipped.contains(&concept.id))
        .map(|concept| concept.id.clone())
}

async fn ensure_current_question(
    data_dir: &Path,
    session: &mut ExplorationSession,
) -> Result<(), AppError> {
    let Some(concept_id) = next_uncovered_id(session) else {
        session.status = SessionStatus::Completed;
        return Ok(());
    };
    let concept = session
        .graph
        .concepts
        .iter()
        .find(|concept| concept.id == concept_id)
        .cloned()
        .ok_or_else(|| AppError::Internal("Next concept is missing".to_string()))?;

    let settings = config::get_settings_path(data_dir)?;
    if settings.llm.base_url.trim().is_empty() {
        return Err(AppError::InvalidInput(
            "No LLM endpoint configured".to_string(),
        ));
    }
    let context = serde_json::json!({
        "concept": concept,
        "graph": session
            .graph
            .concepts
            .iter()
            .map(|concept| serde_json::json!({
                "id": concept.id,
                "title": concept.title,
                "depth": concept.depth,
                "covered": concept.covered,
            }))
            .collect::<Vec<_>>(),
        "note_summaries": session
            .source_digests
            .iter()
            .map(|digest| serde_json::json!({
                "path": digest.path,
                "title": digest.title,
                "summary": digest.summary,
                "headings": digest.headings,
            }))
            .collect::<Vec<_>>(),
    });
    let context_json = serde_json::to_string(&context)
        .map_err(|e| AppError::Internal(format!("Failed to serialize question context: {e}")))?;
    let raw = quiz_engine::call_llm_with_system(
        &settings.llm,
        prompt::EXPLORATION_SYSTEM_PROMPT,
        &prompt::question_prompt(&context_json),
        0.4,
        LlmRunKind::ExplorationTurn,
        data_dir,
    )
    .await?;
    let prompt_md = clean_question(&raw);
    if prompt_md.is_empty() {
        return Err(AppError::Llm(
            "Model returned an empty question".to_string(),
        ));
    }

    session.groups.push(TurnGroup {
        concept_id,
        question: Question {
            prompt_md,
            hint_used: false,
        },
        answer: None,
        result: None,
        whiteboard: None,
        covered_after: Vec::new(),
    });
    Ok(())
}

fn clean_question(raw: &str) -> String {
    let mut text = raw.trim();
    if text.starts_with("```") {
        text = text.trim_start_matches("```");
        if let Some(newline) = text.find('\n') {
            text = &text[newline + 1..];
        }
        text = text.trim_end_matches("```");
    }
    text.trim().to_string()
}

fn turn_context(session: &ExplorationSession, concept: &Concept, answer: &str) -> Value {
    let covered: HashSet<String> = session.covered_concept_ids().into_iter().collect();
    let recent: Vec<Value> = session
        .active_groups()
        .iter()
        .rev()
        .take(3)
        .rev()
        .map(|group| {
            serde_json::json!({
                "question": group.question.prompt_md,
                "answer": group.answer,
                "verdict": group.result.as_ref().map(|result| format!("{:?}", result.verdict).to_lowercase()),
                "feedback": group.result.as_ref().map(|result| result.feedback_md.clone()),
            })
        })
        .collect();

    serde_json::json!({
        "current_concept": concept,
        "concepts": session
            .graph
            .concepts
            .iter()
            .map(|concept| serde_json::json!({
                "id": concept.id,
                "title": concept.title,
                "depth": concept.depth,
                "covered": covered.contains(&concept.id),
            }))
            .collect::<Vec<_>>(),
        "recent_turns": recent,
        "hint_count_for_current": consecutive_hints(session, &concept.id),
        "user_answer": answer,
        "note_summaries": session
            .source_digests
            .iter()
            .map(|digest| serde_json::json!({
                "path": digest.path,
                "title": digest.title,
                "summary": digest.summary,
            }))
            .collect::<Vec<_>>(),
    })
}

/// One map-stage call. Never fails the whole initialization: a broken or
/// unreachable summary falls back to a heading-based digest, and the reduce
/// stage reports the real problem.
async fn summarize_note(
    llm: &crate::models::settings::LlmConfig,
    data_dir: &Path,
    path: &str,
    content: &str,
) -> NoteDigest {
    let body = quiz_engine::note_content_for_prompt(content);
    let raw = quiz_engine::call_llm_with_system(
        llm,
        prompt::EXPLORATION_SYSTEM_PROMPT,
        &prompt::summary_prompt(path, &body),
        0.1,
        LlmRunKind::ExplorationSummary,
        data_dir,
    )
    .await;

    match raw {
        Ok(text) => match graph::parse_json_object(&text) {
            Ok(value) => graph::parse_digest(&value, path, content),
            Err(_) => graph::fallback_digest(path, content),
        },
        Err(_) => graph::fallback_digest(path, content),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::exploration::Concept;

    fn graph() -> KnowledgeGraph {
        KnowledgeGraph {
            concepts: vec![
                Concept {
                    id: "c1".to_string(),
                    title: "One".to_string(),
                    summary: String::new(),
                    depth: 0,
                    prereqs: vec![],
                    source_refs: vec![],
                    covered: false,
                },
                Concept {
                    id: "c2".to_string(),
                    title: "Two".to_string(),
                    summary: String::new(),
                    depth: 1,
                    prereqs: vec!["c1".to_string()],
                    source_refs: vec![],
                    covered: false,
                },
            ],
            domains: vec![],
            root_order: vec!["c1".to_string(), "c2".to_string()],
            domains_confirmed: false,
        }
    }

    fn pending_group(concept_id: &str, hint_used: bool) -> TurnGroup {
        TurnGroup {
            concept_id: concept_id.to_string(),
            question: Question {
                prompt_md: "Why?".to_string(),
                hint_used,
            },
            answer: None,
            result: None,
            whiteboard: None,
            covered_after: vec![],
        }
    }

    fn active_session() -> ExplorationSession {
        ExplorationSession {
            id: "s1".to_string(),
            title: "Compiler".to_string(),
            format_version: 1,
            created_at: "2026-10-23T10:00:00+08:00".to_string(),
            updated_at: "2026-10-23T10:00:00+08:00".to_string(),
            status: SessionStatus::Active,
            origin: crate::models::exploration::SessionOrigin::New,
            source_paths: vec!["a.md".to_string()],
            source_digests: vec![],
            graph: graph(),
            groups: vec![pending_group("c1", false)],
            active_group: 0,
            asides: vec![],
            skipped: vec![],
            domain_warning: None,
            completed_early: false,
        }
    }

    fn response(next_action: NextAction, covered: &[&str]) -> TurnResponse {
        TurnResponse {
            verdict: if next_action == NextAction::Advance {
                Verdict::Correct
            } else {
                Verdict::Unknown
            },
            feedback_md: "Feedback.".to_string(),
            next_action,
            hint: (next_action == NextAction::Hint).then(|| "Think about tokens.".to_string()),
            next_concept_id: Some("c2".to_string()),
            next_question: Some("Next question?".to_string()),
            whiteboard_md: Some("| a | b |".to_string()),
            covered_concepts: covered.iter().map(|id| id.to_string()).collect(),
        }
    }

    #[test]
    fn parse_turn_rejects_unknown_enums() {
        let bad_verdict = serde_json::json!({ "verdict": "maybe", "next_action": "question" });
        assert!(parse_turn(&bad_verdict).is_err());

        let bad_action = serde_json::json!({ "verdict": "correct", "next_action": "skip" });
        assert!(parse_turn(&bad_action).is_err());
    }

    #[test]
    fn advance_marks_the_concept_covered_and_appends_the_next_question() {
        let mut session = active_session();

        apply_turn(
            &mut session,
            "Because.",
            response(NextAction::Advance, &["c1"]),
        );

        assert_eq!(session.status, SessionStatus::Active);
        assert_eq!(session.active_group, 1);
        assert_eq!(session.groups.len(), 2);
        assert_eq!(session.groups[0].answer.as_deref(), Some("Because."));
        assert_eq!(session.groups[0].covered_after, vec!["c1"]);
        assert!(session.groups[0].whiteboard.is_some());
        assert_eq!(session.groups[1].concept_id, "c2");
        assert_eq!(session.groups[1].question.prompt_md, "Next question?");
        assert_eq!(session.covered_concept_ids(), vec!["c1"]);
    }

    #[test]
    fn advance_always_credits_the_current_concept() {
        let mut session = active_session();

        apply_turn(&mut session, "Because.", response(NextAction::Advance, &[]));

        assert_eq!(session.groups[0].covered_after, vec!["c1"]);
    }

    #[test]
    fn hint_keeps_the_same_concept_and_marks_the_reask() {
        let mut session = active_session();

        apply_turn(
            &mut session,
            "I don't know",
            response(NextAction::Hint, &[]),
        );

        assert_eq!(session.active_group, 1);
        assert_eq!(session.groups[1].concept_id, "c1");
        assert!(session.groups[1].question.hint_used);
        assert_eq!(
            session.groups[0].result.as_ref().map(|r| r.next_action),
            Some(NextAction::Hint)
        );
        assert!(session.covered_concept_ids().is_empty());
    }

    #[test]
    fn complete_or_full_coverage_finishes_the_session() {
        let mut session = active_session();
        apply_turn(
            &mut session,
            "Done.",
            response(NextAction::Complete, &["c1"]),
        );
        assert_eq!(session.status, SessionStatus::Completed);
        assert_eq!(session.groups.len(), 1);

        let mut session = active_session();
        apply_turn(
            &mut session,
            "Both.",
            response(NextAction::Advance, &["c1", "c2"]),
        );
        assert_eq!(session.status, SessionStatus::Completed);
        assert_eq!(session.groups.len(), 1);
    }

    #[test]
    fn hint_budget_forces_progress_after_three_hints() {
        let mut session = active_session();
        session.groups = vec![
            pending_group("c1", false),
            pending_group("c1", true),
            pending_group("c1", true),
            pending_group("c1", true),
        ];
        session.active_group = 3;

        apply_turn(
            &mut session,
            "Still not sure",
            response(NextAction::Hint, &[]),
        );

        let result = session.groups[3].result.as_ref().expect("result");
        assert_eq!(result.next_action, NextAction::Advance);
        assert_eq!(session.groups[3].covered_after, vec!["c1"]);
        assert_eq!(session.groups[4].concept_id, "c2");
    }

    #[test]
    fn next_uncovered_follows_root_order() {
        let mut session = active_session();
        session.groups[0].covered_after = vec!["c1".to_string()];
        session.active_group = 1;

        assert_eq!(next_uncovered_id(&session).as_deref(), Some("c2"));

        session.groups[0].covered_after = vec!["c1".to_string(), "c2".to_string()];
        assert_eq!(next_uncovered_id(&session), None);
    }

    #[test]
    fn clean_question_strips_markdown_fences() {
        assert_eq!(
            clean_question("```markdown\nWhy tokens?\n```"),
            "Why tokens?"
        );
        assert_eq!(clean_question("  Why tokens?  "), "Why tokens?");
    }

    #[test]
    fn sanitize_domains_drops_unknown_concepts_and_renumbers() {
        let domains = vec![
            Domain {
                id: "zzz".to_string(),
                title: "  ".to_string(),
                concept_ids: vec!["c1".to_string(), "missing".to_string(), "c1".to_string()],
            },
            Domain {
                id: "empty".to_string(),
                title: "Empty".to_string(),
                concept_ids: vec!["missing".to_string()],
            },
        ];

        let sanitized = sanitize_domains(&graph(), domains).expect("domains should sanitize");

        assert_eq!(sanitized.len(), 1);
        assert_eq!(sanitized[0].id, "d1");
        assert_eq!(sanitized[0].title, "Domain 1");
        assert_eq!(sanitized[0].concept_ids, vec!["c1"]);
    }

    #[test]
    fn sanitize_domains_rejects_a_list_with_no_usable_domain() {
        let domains = vec![Domain {
            id: "d1".to_string(),
            title: "Nope".to_string(),
            concept_ids: vec!["missing".to_string()],
        }];

        assert!(sanitize_domains(&graph(), domains).is_err());
    }

    fn temp_data_dir(test_name: &str) -> std::path::PathBuf {
        let dir = std::env::temp_dir()
            .join("knowtequiz-exploration-engine-tests")
            .join(test_name)
            .join(uuid::Uuid::new_v4().to_string());
        std::fs::create_dir_all(&dir).expect("test temp dir should be created");
        dir
    }

    fn persisted_session(dir: &std::path::Path, session: &ExplorationSession) {
        session_store::save_session(dir, session).expect("session should save");
    }

    fn session_with_two_groups() -> ExplorationSession {
        let mut session = active_session();
        session.groups = vec![
            TurnGroup {
                concept_id: "c1".to_string(),
                question: Question {
                    prompt_md: "Why?".to_string(),
                    hint_used: false,
                },
                answer: Some("Because.".to_string()),
                result: Some(TurnResult {
                    verdict: Verdict::Correct,
                    feedback_md: "Yes.".to_string(),
                    next_action: NextAction::Advance,
                    hint: None,
                }),
                whiteboard: None,
                covered_after: vec!["c1".to_string()],
            },
            pending_group("c2", false),
        ];
        session.active_group = 1;
        session
    }

    #[test]
    fn undo_rolls_back_the_cursor_and_redo_restores_it() {
        let dir = temp_data_dir("undo_rolls_back_the_cursor_and_redo_restores_it");
        let session = session_with_two_groups();
        persisted_session(&dir, &session);

        let undone = undo(&dir, &session.id).expect("undo should succeed");
        assert_eq!(undone.active_group, 0);
        assert!(undone.covered_concept_ids().is_empty());
        assert_eq!(undone.status, SessionStatus::Active);

        let redone = redo(&dir, &session.id).expect("redo should succeed");
        assert_eq!(redone.active_group, 1);
        assert_eq!(redone.covered_concept_ids(), vec!["c1"]);
    }

    #[test]
    fn undo_reopens_a_completed_session() {
        let dir = temp_data_dir("undo_reopens_a_completed_session");
        let mut session = session_with_two_groups();
        session.status = SessionStatus::Completed;
        session.completed_early = true;
        persisted_session(&dir, &session);

        let undone = undo(&dir, &session.id).expect("undo should succeed");

        assert_eq!(undone.status, SessionStatus::Active);
        assert!(!undone.completed_early);
        assert_eq!(undone.active_group, 0);
    }

    #[test]
    fn redo_does_nothing_without_a_retained_tail() {
        let dir = temp_data_dir("redo_does_nothing_without_a_retained_tail");
        let session = active_session();
        persisted_session(&dir, &session);

        let redone = redo(&dir, &session.id).expect("redo should be a no-op");

        assert_eq!(redone.active_group, 0);
        assert_eq!(redone.groups.len(), 1);
    }

    #[test]
    fn finish_early_marks_the_session_completed() {
        let dir = temp_data_dir("finish_early_marks_the_session_completed");
        let session = active_session();
        persisted_session(&dir, &session);

        let finished = finish_early(&dir, &session.id).expect("finish should succeed");

        assert_eq!(finished.status, SessionStatus::Completed);
        assert!(finished.completed_early);
    }

    #[test]
    fn next_uncovered_skips_explicitly_skipped_concepts() {
        let mut session = active_session();
        session.skipped = vec!["c1".to_string()];

        assert_eq!(next_uncovered_id(&session).as_deref(), Some("c2"));

        session.skipped.push("c2".to_string());
        assert_eq!(next_uncovered_id(&session), None);
    }
}
