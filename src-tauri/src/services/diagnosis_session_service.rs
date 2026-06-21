use crate::models::diagnosis::DiagnosisSession;
use std::path::{Path, PathBuf};

const SESSIONS_DIR: &str = "sessions";

pub fn save_session(data_dir: &Path, session: &DiagnosisSession) -> Result<(), String> {
    let sessions_dir = sessions_dir(data_dir)?;
    let filename = session_filename(&session.session_id)?;
    crate::services::storage::write_json_path(&sessions_dir, &filename, session)
}

pub fn load_session(data_dir: &Path, session_id: &str) -> Result<DiagnosisSession, String> {
    let sessions_dir = sessions_dir(data_dir)?;
    let filename = session_filename(session_id)?;
    crate::services::storage::read_json_path(&sessions_dir, &filename)
}

pub fn delete_session(data_dir: &Path, session_id: &str) -> Result<(), String> {
    let sessions_dir = sessions_dir(data_dir)?;
    let filename = session_filename(session_id)?;
    let path = sessions_dir.join(filename);
    remove_file_if_exists(&path, session_id)
}

fn remove_file_if_exists(path: &Path, session_id: &str) -> Result<(), String> {
    if path.exists() {
        std::fs::remove_file(path)
            .map_err(|err| format!("Failed to delete diagnosis session {}: {}", session_id, err))?;
    }
    Ok(())
}

#[derive(Clone, serde::Serialize)]
pub struct SessionCleanupResult {
    pub deleted_count: u32,
    pub remaining_count: u32,
}

pub fn cleanup_expired_sessions(data_dir: &Path, max_age_days: u32) -> Result<SessionCleanupResult, String> {
    let sessions_dir = sessions_dir(data_dir)?;
    let now = std::time::SystemTime::now();
    let cutoff = std::time::Duration::from_secs(max_age_days as u64 * 86400);

    let mut deleted = 0u32;
    let mut remaining = 0u32;

    let entries = std::fs::read_dir(&sessions_dir)
        .map_err(|e| format!("Failed to read sessions directory: {}", e))?;

    for entry in entries {
        let Ok(entry) = entry else { continue };
        let path = entry.path();

        if path.extension().and_then(|e| e.to_str()) != Some("json") {
            continue;
        }

        let Ok(metadata) = entry.metadata() else { continue };
        let Ok(modified) = metadata.modified() else { continue };

        let Ok(elapsed) = now.duration_since(modified) else { continue };

        if elapsed > cutoff {
            let id = path
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("unknown");
            if remove_file_if_exists(&path, id).is_ok() {
                deleted += 1;
            } else {
                remaining += 1;
            }
        } else {
            remaining += 1;
        }
    }

    Ok(SessionCleanupResult {
        deleted_count: deleted,
        remaining_count: remaining,
    })
}

fn sessions_dir(data_dir: &Path) -> Result<PathBuf, String> {
    let dir = data_dir.join(SESSIONS_DIR);
    std::fs::create_dir_all(&dir)
        .map_err(|err| format!("Failed to create diagnosis sessions dir: {}", err))?;
    Ok(dir)
}

fn session_filename(session_id: &str) -> Result<String, String> {
    if session_id.trim().is_empty() || session_id.contains('/') || session_id.contains('\\') {
        return Err("Invalid diagnosis session id".to_string());
    }
    Ok(format!("{}.json", session_id))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::diagnosis::{BlindSpot, DiagnosisRound};

    fn temp_data_dir(test_name: &str) -> PathBuf {
        let dir = std::env::temp_dir()
            .join("knowtequiz-diagnosis-session-tests")
            .join(test_name)
            .join(uuid::Uuid::new_v4().to_string());
        std::fs::create_dir_all(&dir).expect("test temp dir should be created");
        dir
    }

    fn session(id: &str) -> DiagnosisSession {
        DiagnosisSession {
            session_id: id.to_string(),
            question: "What is ownership?".to_string(),
            user_answer: "A".to_string(),
            user_reasoning: "I thought move means copy.".to_string(),
            note_path: "D:/notes/rust.md".to_string(),
            note_content: "Ownership note".to_string(),
            conversation: vec![DiagnosisRound {
                role: "ai".to_string(),
                content: "You confused move and copy.".to_string(),
                blind_spots: vec![BlindSpot {
                    tag: "concept confusion".to_string(),
                    severity: "medium".to_string(),
                    description: "Move semantics were treated as cloning.".to_string(),
                    note_reference: "ownership.md".to_string(),
                    suggestion: "Review move examples.".to_string(),
                }],
                follow_up: Some("When does Rust clone?".to_string()),
            }],
            current_round: 1,
            max_rounds: 3,
            final_report: None,
        }
    }

    #[test]
    fn saved_session_can_be_loaded_from_disk_by_id() {
        let dir = temp_data_dir("saved_session_can_be_loaded_from_disk_by_id");
        let original = session("session-1");

        save_session(&dir, &original).expect("session should save");
        let loaded = load_session(&dir, "session-1").expect("session should load");

        assert_eq!(loaded.session_id, original.session_id);
        assert_eq!(loaded.question, original.question);
        assert_eq!(loaded.conversation.len(), 1);
        assert_eq!(
            loaded.conversation[0].blind_spots[0].tag,
            "concept confusion"
        );
    }

    #[test]
    fn cleanup_removes_sessions_older_than_cutoff() {
        let dir = temp_data_dir("cleanup_removes_sessions_older_than_cutoff");
        let old = session("old-session");
        save_session(&dir, &old).expect("old session should save");

        // Immediately after saving, cleanup with max_age_days=0 should NOT remove it
        // because max_age_days=0 means instant cutoff, i.e. anything is older
        let result =
            cleanup_expired_sessions(&dir, 0).expect("cleanup should succeed");
        assert!(result.deleted_count > 0, "session just created should be deleted with max_age=0");
        assert_eq!(result.remaining_count, 0);

        let error = load_session(&dir, "old-session").expect_err("deleted session should not load");
        assert!(error.contains("File not found"));
    }

    #[test]
    fn cleanup_keeps_recent_sessions() {
        let dir = temp_data_dir("cleanup_keeps_recent_sessions");
        let recent = session("recent-session");
        save_session(&dir, &recent).expect("session should save");

        let result = cleanup_expired_sessions(&dir, 365).expect("cleanup should succeed");
        assert_eq!(result.deleted_count, 0);
        assert_eq!(result.remaining_count, 1);

        let loaded = load_session(&dir, "recent-session").expect("recent session should load");
        assert_eq!(loaded.session_id, "recent-session");
    }

    #[test]
    fn cleanup_skips_non_json_files_in_sessions_directory() {
        let dir = temp_data_dir("cleanup_skips_non_json_files");
        let session = session("keep-me");
        save_session(&dir, &session).expect("session should save");

        // Create a non-json file that should be ignored
        let readme = dir.join("sessions").join("README.txt");
        std::fs::write(&readme, "ignore me").expect("readme should write");

        let result = cleanup_expired_sessions(&dir, 0).expect("cleanup should succeed");
        assert_eq!(result.deleted_count, 1); // only the json session
        assert!(readme.exists(), "non-json files should not be touched");
    }
}
