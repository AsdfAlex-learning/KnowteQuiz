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
    if path.exists() {
        std::fs::remove_file(&path)
            .map_err(|err| format!("Failed to delete diagnosis session {}: {}", session_id, err))?;
    }
    Ok(())
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
        assert_eq!(loaded.conversation[0].blind_spots[0].tag, "concept confusion");
    }

    #[test]
    fn delete_session_removes_persisted_session_file() {
        let dir = temp_data_dir("delete_session_removes_persisted_session_file");
        let original = session("session-2");
        save_session(&dir, &original).expect("session should save");

        delete_session(&dir, "session-2").expect("session should delete");

        let error = load_session(&dir, "session-2").expect_err("deleted session should not load");
        assert!(error.contains("File not found"));
    }
}
