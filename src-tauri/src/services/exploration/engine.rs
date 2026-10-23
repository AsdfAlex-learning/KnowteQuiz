use crate::errors::AppError;
use crate::models::exploration::{
    Domain, ExplorationSession, KnowledgeGraph, NoteDigest, SessionStatus,
};
use crate::models::llm_run::LlmRunKind;
use crate::services::exploration::{graph, prompt, session_store};
use crate::services::{config, fs_service, quiz_engine};
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
            root_order: vec!["c1".to_string()],
            domains_confirmed: false,
        }
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
}
