use crate::models::mistake::{MistakeEntry, MistakeFilter};
use std::path::Path;

const MISTAKES_FILE: &str = "mistakes.json";

pub fn load_mistakes(data_dir: &Path, filter: &MistakeFilter) -> Result<Vec<MistakeEntry>, String> {
    let mistakes = read_mistakes_or_empty(data_dir)?;
    Ok(filter_mistakes(&mistakes, filter))
}

pub fn save_mistake(data_dir: &Path, entry: MistakeEntry) -> Result<(), String> {
    let mistakes = read_mistakes_or_empty(data_dir)?;
    let updated = upsert_mistake(mistakes, entry);
    crate::services::storage::write_json_path(data_dir, MISTAKES_FILE, &updated)
}

fn read_mistakes_or_empty(data_dir: &Path) -> Result<Vec<MistakeEntry>, String> {
    match crate::services::storage::read_json_path(data_dir, MISTAKES_FILE) {
        Ok(mistakes) => Ok(mistakes),
        Err(error) if error.starts_with("File not found:") => Ok(vec![]),
        Err(error) => Err(error),
    }
}

pub fn upsert_mistake(existing: Vec<MistakeEntry>, incoming: MistakeEntry) -> Vec<MistakeEntry> {
    let incoming_key = mistake_key(&incoming);
    let mut updated = Vec::with_capacity(existing.len() + 1);
    updated.push(incoming);
    updated.extend(
        existing
            .into_iter()
            .filter(|entry| mistake_key(entry) != incoming_key),
    );
    updated
}

pub fn filter_mistakes(mistakes: &[MistakeEntry], filter: &MistakeFilter) -> Vec<MistakeEntry> {
    let offset = filter.offset.unwrap_or(0) as usize;
    let limit = filter.limit.map(|value| value as usize);
    let mut filtered = mistakes
        .iter()
        .filter(|entry| {
            filter
                .mode
                .as_ref()
                .map_or(true, |mode| std::mem::discriminant(&entry.mode) == std::mem::discriminant(mode))
        })
        .filter(|entry| {
            filter
                .note_path
                .as_ref()
                .map_or(true, |note_path| entry.note_path == *note_path)
        })
        .cloned()
        .collect::<Vec<_>>();

    filtered.sort_by(|a, b| b.created_at.cmp(&a.created_at));

    let paged = filtered.into_iter().skip(offset);
    match limit {
        Some(limit) => paged.take(limit).collect(),
        None => paged.collect(),
    }
}

fn mistake_key(entry: &MistakeEntry) -> String {
    format!(
        "{}\u{1f}{}\u{1f}{:?}",
        entry.note_path.trim(),
        normalize_question(&entry.question),
        std::mem::discriminant(&entry.mode),
    )
}

fn normalize_question(question: &str) -> String {
    question.split_whitespace().collect::<Vec<_>>().join(" ")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::mistake::{MistakeEntry, MistakeFilter, MistakeMode};
    use std::path::PathBuf;

    fn temp_data_dir(test_name: &str) -> PathBuf {
        let dir = std::env::temp_dir()
            .join("knowtequiz-mistake-service-tests")
            .join(test_name)
            .join(uuid::Uuid::new_v4().to_string());
        std::fs::create_dir_all(&dir).expect("test temp dir should be created");
        dir
    }

    fn mistake(
        id: &str,
        note_path: &str,
        question: &str,
        mode: MistakeMode,
        created_at: &str,
    ) -> MistakeEntry {
        MistakeEntry {
            id: id.to_string(),
            note_path: note_path.to_string(),
            note_title: "Note".to_string(),
            question: question.to_string(),
            user_answer: "A".to_string(),
            correct_answer: "B".to_string(),
            explanation: "Because.".to_string(),
            mode,
            user_reasoning: None,
            diagnosis: None,
            created_at: created_at.to_string(),
            review_count: 0,
        }
    }

    #[test]
    fn upsert_mistake_moves_duplicate_to_front_and_replaces_old_entry() {
        let existing = vec![
            mistake("old", "/notes/rust.md", "What is ownership?", MistakeMode::Basic, "2026-01-01T00:00:00Z"),
            mistake("other", "/notes/vue.md", "What is ref?", MistakeMode::Basic, "2026-01-02T00:00:00Z"),
        ];
        let incoming = mistake("new", "/notes/rust.md", " What is ownership? ", MistakeMode::Basic, "2026-01-03T00:00:00Z");

        let updated = upsert_mistake(existing, incoming);

        assert_eq!(updated.len(), 2);
        assert_eq!(updated[0].id, "new");
        assert_eq!(updated[1].id, "other");
    }

    #[test]
    fn filter_mistakes_applies_mode_note_path_offset_and_limit() {
        let mistakes = vec![
            mistake("a", "/notes/rust.md", "A?", MistakeMode::Basic, "2026-01-04T00:00:00Z"),
            mistake("b", "/notes/rust.md", "B?", MistakeMode::Advanced, "2026-01-03T00:00:00Z"),
            mistake("c", "/notes/rust.md", "C?", MistakeMode::Advanced, "2026-01-02T00:00:00Z"),
            mistake("d", "/notes/vue.md", "D?", MistakeMode::Advanced, "2026-01-01T00:00:00Z"),
        ];
        let filter = MistakeFilter {
            mode: Some(MistakeMode::Advanced),
            note_path: Some("/notes/rust.md".to_string()),
            offset: Some(1),
            limit: Some(1),
        };

        let filtered = filter_mistakes(&mistakes, &filter);

        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].id, "c");
    }

    #[test]
    fn load_mistakes_returns_empty_list_when_file_is_missing() {
        let dir = temp_data_dir("load_mistakes_returns_empty_list_when_file_is_missing");

        let mistakes = load_mistakes(&dir, &MistakeFilter::default())
            .expect("missing mistakes file should start empty");

        assert!(mistakes.is_empty());
    }

    #[test]
    fn load_mistakes_reports_corrupt_file_without_overwriting_it() {
        let dir = temp_data_dir("load_mistakes_reports_corrupt_file_without_overwriting_it");
        std::fs::write(dir.join(MISTAKES_FILE), "{ not valid json")
            .expect("corrupt mistakes file should be written");

        let error = load_mistakes(&dir, &MistakeFilter::default())
            .expect_err("corrupt mistakes file should not be treated as empty");

        assert!(error.contains("Failed to parse"));
    }
}
