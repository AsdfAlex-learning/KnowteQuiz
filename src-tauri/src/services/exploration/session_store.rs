use crate::errors::AppError;
use crate::models::exploration::{
    ExplorationSession, ExplorationSessionSummary, KnowledgeGraph, SessionOrigin, SessionStatus,
    EXPLORATION_FORMAT_VERSION,
};
use std::path::{Path, PathBuf};

const EXPLORATION_DIR: &str = "exploration";
const SESSIONS_DIR: &str = "sessions";

/// Creates and persists a new session in `Initializing` state.
///
/// The graph is filled by the initialization pipeline; a session exists before
/// the first LLM call so a failed or cancelled initialization is resumable.
pub fn create_session(
    data_dir: &Path,
    source_paths: Vec<String>,
    title: Option<String>,
    baseline: Option<u32>,
) -> Result<ExplorationSession, AppError> {
    let source_paths: Vec<String> = source_paths
        .into_iter()
        .map(|path| path.trim().to_string())
        .filter(|path| !path.is_empty())
        .collect();
    if source_paths.is_empty() {
        return Err(AppError::InvalidInput(
            "At least one source note is required".to_string(),
        ));
    }

    let now = chrono::Local::now().to_rfc3339();
    let session = ExplorationSession {
        id: uuid::Uuid::new_v4().simple().to_string(),
        title: title
            .map(|value| value.trim().to_string())
            .filter(|value| !value.is_empty())
            .unwrap_or_else(|| derive_title(&source_paths)),
        format_version: EXPLORATION_FORMAT_VERSION,
        created_at: now.clone(),
        updated_at: now,
        status: SessionStatus::Initializing,
        origin: SessionOrigin::New,
        baseline: baseline.unwrap_or(50).min(100),
        source_paths,
        source_digests: Vec::new(),
        graph: KnowledgeGraph::default(),
        groups: Vec::new(),
        active_group: 0,
        asides: vec![],
        skipped: vec![],
        domain_warning: None,
        completed_early: false,
    };

    save_session(data_dir, &session)?;
    Ok(session)
}

/// Human-readable fallback title: the first source note's file stem.
fn derive_title(source_paths: &[String]) -> String {
    source_paths
        .first()
        .map(|path| {
            let file = path
                .rsplit(['/', '\\'])
                .next()
                .unwrap_or(path)
                .trim_end_matches(".md")
                .trim_end_matches(".markdown");
            if file.is_empty() {
                "Exploration".to_string()
            } else {
                file.to_string()
            }
        })
        .unwrap_or_else(|| "Exploration".to_string())
}

pub fn save_session(data_dir: &Path, session: &ExplorationSession) -> Result<(), AppError> {
    let sessions_dir = sessions_dir(data_dir)?;
    let filename = session_filename(&session.id)?;
    crate::services::storage::write_json_path(&sessions_dir, &filename, session)
}

pub fn load_session(data_dir: &Path, session_id: &str) -> Result<ExplorationSession, AppError> {
    let sessions_dir = sessions_dir(data_dir)?;
    let filename = session_filename(session_id)?;
    crate::services::storage::read_json_path(&sessions_dir, &filename)
}

pub fn delete_session(data_dir: &Path, session_id: &str) -> Result<(), AppError> {
    let sessions_dir = sessions_dir(data_dir)?;
    let filename = session_filename(session_id)?;
    let path = sessions_dir.join(filename);
    if path.exists() {
        std::fs::remove_file(path).map_err(|err| {
            AppError::Internal(format!(
                "Failed to delete exploration session {}: {}",
                session_id, err
            ))
        })?;
    }
    Ok(())
}

/// Lists saved sessions, newest first. Unreadable files are skipped rather than
/// failing the whole list.
pub fn list_sessions(data_dir: &Path) -> Result<Vec<ExplorationSessionSummary>, AppError> {
    let sessions_dir = sessions_dir(data_dir)?;
    let mut rows: Vec<(std::time::SystemTime, ExplorationSessionSummary)> = Vec::new();

    let entries = std::fs::read_dir(&sessions_dir)
        .map_err(|e| AppError::Internal(format!("Failed to read exploration sessions: {}", e)))?;

    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) != Some("json") {
            continue;
        }
        let Ok(session) = crate::services::storage::read_json_path::<ExplorationSession>(
            &sessions_dir,
            &file_name(&path),
        ) else {
            continue;
        };
        let modified = entry
            .metadata()
            .and_then(|metadata| metadata.modified())
            .unwrap_or(std::time::SystemTime::UNIX_EPOCH);
        rows.push((modified, session.summary()));
    }

    rows.sort_by_key(|row| std::cmp::Reverse(row.0));
    Ok(rows.into_iter().map(|(_, summary)| summary).collect())
}

#[derive(Clone, serde::Serialize)]
pub struct ExplorationCleanupResult {
    pub deleted_count: u32,
    pub remaining_count: u32,
}

/// Deletes non-active sessions older than `max_age_days`. Active sessions are
/// always kept: they are resumable work, not disposable history.
pub fn cleanup_expired_sessions(
    data_dir: &Path,
    max_age_days: u32,
) -> Result<ExplorationCleanupResult, AppError> {
    let sessions_dir = sessions_dir(data_dir)?;
    let now = std::time::SystemTime::now();
    let cutoff = std::time::Duration::from_secs(max_age_days as u64 * 86400);

    let mut deleted = 0u32;
    let mut remaining = 0u32;

    let entries = std::fs::read_dir(&sessions_dir)
        .map_err(|e| AppError::Internal(format!("Failed to read exploration sessions: {}", e)))?;

    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) != Some("json") {
            continue;
        }
        let Ok(session) = crate::services::storage::read_json_path::<ExplorationSession>(
            &sessions_dir,
            &file_name(&path),
        ) else {
            remaining += 1;
            continue;
        };
        if session.status == SessionStatus::Active {
            remaining += 1;
            continue;
        }
        let elapsed = entry
            .metadata()
            .and_then(|metadata| metadata.modified())
            .ok()
            .and_then(|modified| now.duration_since(modified).ok());
        match elapsed {
            Some(elapsed) if elapsed > cutoff => {
                if std::fs::remove_file(&path).is_ok() {
                    deleted += 1;
                } else {
                    remaining += 1;
                }
            }
            _ => remaining += 1,
        }
    }

    Ok(ExplorationCleanupResult {
        deleted_count: deleted,
        remaining_count: remaining,
    })
}

fn file_name(path: &Path) -> String {
    path.file_name()
        .and_then(|name| name.to_str())
        .unwrap_or_default()
        .to_string()
}

fn sessions_dir(data_dir: &Path) -> Result<PathBuf, AppError> {
    let dir = data_dir.join(EXPLORATION_DIR).join(SESSIONS_DIR);
    std::fs::create_dir_all(&dir).map_err(|err| {
        AppError::Internal(format!(
            "Failed to create exploration sessions dir: {}",
            err
        ))
    })?;
    Ok(dir)
}

fn session_filename(session_id: &str) -> Result<String, AppError> {
    if session_id.trim().is_empty() || session_id.contains('/') || session_id.contains('\\') {
        return Err(AppError::InvalidInput(
            "Invalid exploration session id".to_string(),
        ));
    }
    Ok(format!("{}.json", session_id))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::exploration::{Concept, Question, TurnGroup};

    fn temp_data_dir(test_name: &str) -> PathBuf {
        let dir = std::env::temp_dir()
            .join("knowtequiz-exploration-session-tests")
            .join(test_name)
            .join(uuid::Uuid::new_v4().to_string());
        std::fs::create_dir_all(&dir).expect("test temp dir should be created");
        dir
    }

    fn saved_session(dir: &Path, id: &str, status: SessionStatus) -> ExplorationSession {
        let now = "2026-10-23T10:00:00+08:00".to_string();
        let session = ExplorationSession {
            id: id.to_string(),
            title: "Compiler".to_string(),
            format_version: EXPLORATION_FORMAT_VERSION,
            created_at: now.clone(),
            updated_at: now,
            status,
            origin: SessionOrigin::New,
            baseline: 50,
            source_paths: vec!["notes/compiler.md".to_string()],
            source_digests: vec![],
            graph: KnowledgeGraph::default(),
            groups: Vec::new(),
            active_group: 0,
            asides: vec![],
            skipped: vec![],
            domain_warning: None,
            completed_early: false,
        };
        save_session(dir, &session).expect("session should save");
        session
    }

    #[test]
    fn create_session_persists_an_initializing_session() {
        let dir = temp_data_dir("create_session_persists_an_initializing_session");

        let session = create_session(
            &dir,
            vec![" notes/a.md ".to_string(), String::new()],
            None,
            None,
        )
        .expect("session should be created");

        assert_eq!(session.status, SessionStatus::Initializing);
        assert_eq!(session.source_paths, vec!["notes/a.md"]);
        assert_eq!(session.title, "a");
        assert_eq!(session.format_version, EXPLORATION_FORMAT_VERSION);
        assert_eq!(session.active_group, 0);
        assert_eq!(session.baseline, 50);

        let loaded = load_session(&dir, &session.id).expect("session should load");
        assert_eq!(loaded, session);
        assert!(dir.join("exploration").join("sessions").exists());
    }

    #[test]
    fn create_session_clamps_the_baseline() {
        let dir = temp_data_dir("create_session_clamps_the_baseline");

        let high = create_session(&dir, vec!["notes/high.md".to_string()], None, Some(120))
            .expect("session should be created");
        assert_eq!(high.baseline, 100);

        let low = create_session(&dir, vec!["notes/low.md".to_string()], None, Some(15))
            .expect("session should be created");
        assert_eq!(low.baseline, 15);
    }

    #[test]
    fn create_session_rejects_empty_source_list() {
        let dir = temp_data_dir("create_session_rejects_empty_source_list");

        let error = create_session(&dir, vec![], None, None).expect_err("empty sources must fail");

        assert!(error.to_string().contains("At least one source note"));
    }

    #[test]
    fn list_sessions_returns_newest_first_summaries() {
        let dir = temp_data_dir("list_sessions_returns_newest_first_summaries");
        saved_session(&dir, "older", SessionStatus::Completed);
        std::thread::sleep(std::time::Duration::from_millis(15));
        saved_session(&dir, "newer", SessionStatus::Active);

        let rows = list_sessions(&dir).expect("sessions should list");

        assert_eq!(rows.len(), 2);
        assert_eq!(rows[0].id, "newer");
        assert_eq!(rows[1].id, "older");
        assert_eq!(rows[0].status, SessionStatus::Active);
    }

    #[test]
    fn delete_session_removes_the_file() {
        let dir = temp_data_dir("delete_session_removes_the_file");
        saved_session(&dir, "gone", SessionStatus::Completed);

        delete_session(&dir, "gone").expect("delete should succeed");

        assert!(load_session(&dir, "gone").is_err());
        assert!(delete_session(&dir, "gone").is_ok());
    }

    #[test]
    fn cleanup_keeps_active_sessions_even_when_old() {
        let dir = temp_data_dir("cleanup_keeps_active_sessions_even_when_old");
        saved_session(&dir, "active", SessionStatus::Active);
        saved_session(&dir, "finished", SessionStatus::Completed);

        let result = cleanup_expired_sessions(&dir, 0).expect("cleanup should succeed");

        assert_eq!(result.deleted_count, 1);
        assert_eq!(result.remaining_count, 1);
        assert!(load_session(&dir, "active").is_ok());
        assert!(load_session(&dir, "finished").is_err());
    }

    #[test]
    fn cleanup_skips_non_json_files() {
        let dir = temp_data_dir("cleanup_skips_non_json_files");
        saved_session(&dir, "keep", SessionStatus::Completed);
        let sessions_dir = dir.join("exploration").join("sessions");
        std::fs::write(sessions_dir.join("README.txt"), "ignore me").expect("readme should write");

        let result = cleanup_expired_sessions(&dir, 0).expect("cleanup should succeed");

        assert_eq!(result.deleted_count, 1);
        assert!(sessions_dir.join("README.txt").exists());
    }

    #[test]
    fn summary_counts_covered_concepts_from_the_timeline() {
        let dir = temp_data_dir("summary_counts_covered_concepts_from_the_timeline");
        let mut session = saved_session(&dir, "counted", SessionStatus::Active);
        session.graph.concepts = vec![
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
        ];
        session.groups = vec![TurnGroup {
            concept_id: "c1".to_string(),
            question: Question {
                prompt_md: "Why?".to_string(),
                hint_used: false,
            },
            answer: Some("Because.".to_string()),
            result: None,
            whiteboard: None,
            covered_after: vec!["c1".to_string()],
        }];
        session.active_group = 1;

        let summary = session.summary();

        assert_eq!(summary.concepts_total, 2);
        assert_eq!(summary.concepts_covered, 1);
        assert_eq!(summary.group_count, 1);
    }
}
