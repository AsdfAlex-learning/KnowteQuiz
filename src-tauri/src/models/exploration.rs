use serde::{Deserialize, Serialize};

/// Version stamped on every persisted exploration session and learning-history
/// export. Bump only when the on-disk shape changes incompatibly.
pub const EXPLORATION_FORMAT_VERSION: u32 = 1;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SessionStatus {
    Initializing,
    AwaitingDomainConfirm,
    Active,
    Completed,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", tag = "kind")]
pub enum SessionOrigin {
    New,
    Imported { source_path: String },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SourceRef {
    pub note_path: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub heading: Option<String>,
    pub snippet: String,
}

/// Per-note extraction produced by the map stage. Stored in the session so the
/// turn engine can ground questions without re-reading every note.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NoteDigest {
    pub path: String,
    #[serde(default)]
    pub title: String,
    #[serde(default)]
    pub headings: Vec<String>,
    #[serde(default)]
    pub summary: String,
    #[serde(default)]
    pub key_terms: Vec<String>,
    #[serde(default)]
    pub domain_guess: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Concept {
    pub id: String,
    pub title: String,
    #[serde(default)]
    pub summary: String,
    pub depth: u32,
    #[serde(default)]
    pub prereqs: Vec<String>,
    #[serde(default)]
    pub source_refs: Vec<SourceRef>,
    #[serde(default)]
    pub covered: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Domain {
    pub id: String,
    pub title: String,
    #[serde(default)]
    pub concept_ids: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct KnowledgeGraph {
    #[serde(default)]
    pub concepts: Vec<Concept>,
    #[serde(default)]
    pub domains: Vec<Domain>,
    #[serde(default)]
    pub root_order: Vec<String>,
    #[serde(default)]
    pub domains_confirmed: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Question {
    pub prompt_md: String,
    #[serde(default)]
    pub hint_used: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Verdict {
    Correct,
    Partial,
    Wrong,
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NextAction {
    Question,
    Hint,
    Advance,
    Complete,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TurnResult {
    pub verdict: Verdict,
    #[serde(default)]
    pub feedback_md: String,
    pub next_action: NextAction,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub hint: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WhiteboardStep {
    pub id: String,
    pub concept_id: String,
    #[serde(default)]
    pub title: String,
    pub content_md: String,
    pub group_index: usize,
}

/// A `/btw` question. Kept out of the timeline: it never advances the cursor
/// and never marks anything as covered.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AsideTurn {
    pub id: String,
    pub question: String,
    pub answer_md: String,
    /// Timeline position the aside belongs to (the pending group index).
    pub group_index: usize,
    pub created_at: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TurnGroup {
    pub concept_id: String,
    pub question: Question,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub answer: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub result: Option<TurnResult>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub whiteboard: Option<WhiteboardStep>,
    #[serde(default)]
    pub covered_after: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ExplorationSession {
    pub id: String,
    pub title: String,
    #[serde(default = "default_format_version")]
    pub format_version: u32,
    pub created_at: String,
    pub updated_at: String,
    pub status: SessionStatus,
    pub origin: SessionOrigin,
    /// Learner baseline chosen before initialization (0 = from zero, 100 =
    /// already advanced). Older sessions default to the neutral midpoint.
    #[serde(default = "crate::models::settings::default_baseline_level")]
    pub baseline: u32,
    #[serde(default)]
    pub source_paths: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub source_digests: Vec<NoteDigest>,
    #[serde(default)]
    pub graph: KnowledgeGraph,
    #[serde(default)]
    pub groups: Vec<TurnGroup>,
    #[serde(default)]
    pub active_group: usize,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub asides: Vec<AsideTurn>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub skipped: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub domain_warning: Option<String>,
    #[serde(default)]
    pub completed_early: bool,
}

fn default_format_version() -> u32 {
    EXPLORATION_FORMAT_VERSION
}

/// Payload for creating an exploration session from a set of source notes.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CreateExplorationSessionRequest {
    pub source_paths: Vec<String>,
    #[serde(default)]
    pub title: Option<String>,
    /// 0-100. Omitted uses the neutral midpoint.
    #[serde(default)]
    pub baseline: Option<u32>,
}

/// Payload for confirming (possibly edited) domains after initialization
/// detected several subject areas.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ConfirmExplorationDomainsRequest {
    pub domains: Vec<Domain>,
}

/// Payload for submitting one learner answer.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AskExplorationRequest {
    pub answer: String,
}

/// Payload for a `/btw` side question.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AsideExplorationRequest {
    pub question: String,
}

/// Payload for importing a learning-history Markdown file.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ImportExplorationRequest {
    pub markdown: String,
    #[serde(default)]
    pub source_name: Option<String>,
}

/// Payload for saving a history into the note vault.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SaveExplorationToVaultRequest {
    pub file_name: String,
}

/// Lightweight row used by the session list endpoint.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExplorationSessionSummary {
    pub id: String,
    pub title: String,
    pub status: SessionStatus,
    pub created_at: String,
    pub updated_at: String,
    pub source_paths: Vec<String>,
    pub concepts_total: usize,
    pub concepts_covered: usize,
    pub group_count: usize,
    pub completed_early: bool,
}

impl ExplorationSession {
    /// Groups that are still part of the active timeline. Groups after
    /// `active_group` are kept only so `/redo` can restore them.
    pub fn active_groups(&self) -> &[TurnGroup] {
        let end = self.active_group.min(self.groups.len());
        &self.groups[..end]
    }

    /// Concept ids covered so far, in first-covered order. Derived from the
    /// timeline rather than stored separately so undo/redo can never drift.
    pub fn covered_concept_ids(&self) -> Vec<String> {
        let mut covered: Vec<String> = Vec::new();
        for group in self.active_groups() {
            for id in &group.covered_after {
                if !covered.contains(id) {
                    covered.push(id.clone());
                }
            }
        }
        covered
    }

    pub fn summary(&self) -> ExplorationSessionSummary {
        let covered = self.covered_concept_ids();
        let concepts_covered = self
            .graph
            .concepts
            .iter()
            .filter(|c| covered.iter().any(|id| id == &c.id))
            .count();

        ExplorationSessionSummary {
            id: self.id.clone(),
            title: self.title.clone(),
            status: self.status,
            created_at: self.created_at.clone(),
            updated_at: self.updated_at.clone(),
            source_paths: self.source_paths.clone(),
            concepts_total: self.graph.concepts.len(),
            concepts_covered,
            group_count: self.active_groups().len(),
            completed_early: self.completed_early,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn group(concept_id: &str, covered: &[&str], group_index: usize) -> TurnGroup {
        TurnGroup {
            concept_id: concept_id.to_string(),
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
            whiteboard: Some(WhiteboardStep {
                id: format!("wb-{group_index}"),
                concept_id: concept_id.to_string(),
                title: "Board".to_string(),
                content_md: "| a | b |".to_string(),
                group_index,
            }),
            covered_after: covered.iter().map(|s| s.to_string()).collect(),
        }
    }

    fn session() -> ExplorationSession {
        let now = "2026-10-23T10:00:00+08:00".to_string();
        ExplorationSession {
            id: "abc".to_string(),
            title: "Compiler".to_string(),
            format_version: EXPLORATION_FORMAT_VERSION,
            created_at: now.clone(),
            updated_at: now,
            status: SessionStatus::Active,
            origin: SessionOrigin::New,
            baseline: 50,
            source_paths: vec!["a.md".to_string()],
            source_digests: vec![],
            graph: KnowledgeGraph {
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
                domains_confirmed: true,
            },
            groups: vec![group("c1", &["c1"], 0), group("c2", &["c2"], 1)],
            active_group: 2,
            asides: vec![],
            skipped: vec![],
            domain_warning: None,
            completed_early: false,
        }
    }

    #[test]
    fn covered_concepts_follow_the_active_timeline() {
        let mut value = session();
        assert_eq!(value.covered_concept_ids(), vec!["c1", "c2"]);

        value.active_group = 1;
        assert_eq!(value.covered_concept_ids(), vec!["c1"]);
    }

    #[test]
    fn summary_counts_only_the_active_timeline() {
        let mut value = session();
        value.active_group = 1;

        let summary = value.summary();

        assert_eq!(summary.concepts_total, 2);
        assert_eq!(summary.concepts_covered, 1);
        assert_eq!(summary.group_count, 1);
        assert_eq!(summary.status, SessionStatus::Active);
    }

    #[test]
    fn session_roundtrips_through_json_with_snake_case_enums() {
        let value = session();
        let json = serde_json::to_value(&value).expect("session should serialize");

        assert_eq!(json["status"], "active");
        assert_eq!(json["origin"]["kind"], "new");
        assert_eq!(json["format_version"], 1);
        assert_eq!(json["groups"][0]["result"]["verdict"], "correct");

        let restored: ExplorationSession =
            serde_json::from_value(json).expect("session should deserialize");
        assert_eq!(restored, value);
    }

    #[test]
    fn legacy_payload_defaults_missing_optional_sections() {
        let json = serde_json::json!({
            "id": "x",
            "title": "Legacy",
            "created_at": "2026-10-23T10:00:00+08:00",
            "updated_at": "2026-10-23T10:00:00+08:00",
            "status": "initializing",
            "origin": { "kind": "imported", "source_path": "old.md" }
        });

        let restored: ExplorationSession =
            serde_json::from_value(json).expect("legacy session should deserialize");

        assert_eq!(restored.format_version, EXPLORATION_FORMAT_VERSION);
        assert!(restored.graph.concepts.is_empty());
        assert_eq!(restored.active_group, 0);
        assert_eq!(
            restored.origin,
            SessionOrigin::Imported {
                source_path: "old.md".to_string()
            }
        );
    }
}
