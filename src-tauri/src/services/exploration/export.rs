use crate::errors::AppError;
use crate::models::exploration::{
    ExplorationSession, SessionOrigin, SessionStatus, EXPLORATION_FORMAT_VERSION,
};
use crate::services::config;
use crate::services::exploration::session_store;
use std::fmt::Write as _;
use std::path::Path;

const SESSION_MARKER: &str = "<!-- knowtequiz:session";

#[derive(Debug, Clone, serde::Serialize)]
pub struct ExportResult {
    pub path: String,
    pub file_name: String,
    pub markdown: String,
}

/// Renders the human-readable learning history plus a hidden JSON appendix
/// used by `import`. The appendix keeps import lossless while the visible body
/// stays pleasant to read.
pub fn render_markdown(session: &ExplorationSession) -> String {
    let mut out = String::new();

    let _ = writeln!(out, "---");
    let _ = writeln!(out, "knowtequiz: exploration/{}", session.format_version);
    let _ = writeln!(out, "title: {}", session.title);
    let _ = writeln!(out, "created_at: {}", session.created_at);
    let _ = writeln!(out, "updated_at: {}", session.updated_at);
    let _ = writeln!(
        out,
        "status: {}",
        if session.status == SessionStatus::Completed {
            "completed"
        } else {
            "active"
        }
    );
    let _ = writeln!(out, "completed_early: {}", session.completed_early);
    let source_paths = serde_json::to_string(&session.source_paths).unwrap_or_default();
    let _ = writeln!(out, "source_paths: {}", source_paths);
    let covered = session.covered_concept_ids();
    let _ = writeln!(out, "concepts_total: {}", session.graph.concepts.len());
    let _ = writeln!(
        out,
        "concepts_covered: {}",
        session
            .graph
            .concepts
            .iter()
            .filter(|concept| covered.iter().any(|id| id == &concept.id))
            .count()
    );
    let _ = writeln!(out, "---");
    let _ = writeln!(out);

    let _ = writeln!(out, "# 知识脉络");
    let _ = writeln!(out);
    for concept in &session.graph.concepts {
        let indent = "  ".repeat(concept.depth as usize);
        let _ = writeln!(
            out,
            "{}- {} (depth {})",
            indent, concept.title, concept.depth
        );
    }
    let _ = writeln!(out);

    let _ = writeln!(out, "# 学习记录");
    let _ = writeln!(out);
    for (index, group) in session.groups[..session.active_group.min(session.groups.len())]
        .iter()
        .enumerate()
    {
        let title = concept_title(session, &group.concept_id);
        let depth = concept_depth(session, &group.concept_id);
        let _ = writeln!(out, "## {:02} · {}", index + 1, title);
        let _ = writeln!(out);
        let _ = writeln!(out, "> 概念：`{}` · depth {}", group.concept_id, depth);
        if let Some(result) = &group.result {
            let _ = writeln!(out, "> 判定：{:?}", result.verdict);
        }
        if let Some(first_ref) = concept_refs(session, &group.concept_id).first() {
            let _ = writeln!(out, "> 来源：{}", first_ref);
        }
        let _ = writeln!(out);
        let _ = writeln!(out, "**问**：{}", group.question.prompt_md);
        let _ = writeln!(out);
        if let Some(answer) = &group.answer {
            let _ = writeln!(out, "**答**：{}", answer);
            let _ = writeln!(out);
        }
        if let Some(result) = &group.result {
            if !result.feedback_md.is_empty() {
                let _ = writeln!(out, "**反馈**：{}", result.feedback_md);
                let _ = writeln!(out);
            }
            if let Some(hint) = &result.hint {
                let _ = writeln!(out, "**提示**：{}", hint);
                let _ = writeln!(out);
            }
        }
        if let Some(whiteboard) = &group.whiteboard {
            let _ = writeln!(out, "**白板**：");
            let _ = writeln!(out);
            let fence = fence_for(&whiteboard.content_md);
            let _ = writeln!(out, "{fence}md");
            let _ = writeln!(out, "{}", whiteboard.content_md);
            let _ = writeln!(out, "{fence}");
            let _ = writeln!(out);
        }

        for aside in session
            .asides
            .iter()
            .filter(|aside| aside.group_index == index)
        {
            let _ = writeln!(out, "## 旁支 · {}", aside.question);
            let _ = writeln!(out);
            let _ = writeln!(out, "> 位置：{}", aside.group_index);
            let _ = writeln!(out);
            let _ = writeln!(out, "{}", aside.answer_md);
            let _ = writeln!(out);
        }
    }

    for aside in session
        .asides
        .iter()
        .filter(|aside| aside.group_index >= session.active_group)
    {
        let _ = writeln!(out, "## 旁支 · {}", aside.question);
        let _ = writeln!(out);
        let _ = writeln!(out, "> 位置：{}", aside.group_index);
        let _ = writeln!(out);
        let _ = writeln!(out, "{}", aside.answer_md);
        let _ = writeln!(out);
    }

    let uncovered: Vec<&str> = session
        .graph
        .concepts
        .iter()
        .filter(|concept| !covered.iter().any(|id| id == &concept.id))
        .map(|concept| concept.title.as_str())
        .collect();
    if !uncovered.is_empty() {
        let _ = writeln!(out, "# 未覆盖");
        let _ = writeln!(out);
        for title in uncovered {
            let _ = writeln!(out, "- [ ] {}", title);
        }
        let _ = writeln!(out);
    }

    let _ = writeln!(out, "# 附录：Mermaid（导入时忽略）");
    let _ = writeln!(out);
    let _ = writeln!(out, "```mermaid");
    let _ = writeln!(out, "flowchart TD");
    for concept in &session.graph.concepts {
        let _ = writeln!(
            out,
            "  {}[\"{}\"]",
            concept.id,
            concept.title.replace('"', "'")
        );
    }
    for concept in &session.graph.concepts {
        for prereq in &concept.prereqs {
            let _ = writeln!(out, "  {} --> {}", prereq, concept.id);
        }
    }
    let _ = writeln!(out, "```");
    let _ = writeln!(out);

    let mut appendix = session.clone();
    appendix.source_digests.clear();
    let json = serde_json::to_string(&appendix).unwrap_or_default();
    let _ = writeln!(out, "{}", SESSION_MARKER);
    let _ = writeln!(out, "{}", json);
    let _ = writeln!(out, "-->");

    out
}

/// Extracts the session from the hidden appendix. Returns an error for files
/// that are not KnowteQuiz exploration histories.
pub fn parse_import(markdown: &str) -> Result<ExplorationSession, AppError> {
    let start = markdown.find(SESSION_MARKER).ok_or_else(|| {
        AppError::InvalidInput(
            "Not a KnowteQuiz exploration history (missing session appendix)".to_string(),
        )
    })?;
    let rest = &markdown[start + SESSION_MARKER.len()..];
    let end = rest.find("-->").ok_or_else(|| {
        AppError::InvalidInput("Corrupt KnowteQuiz exploration history appendix".to_string())
    })?;
    let session: ExplorationSession =
        serde_json::from_str(rest[..end].trim()).map_err(|error| {
            AppError::InvalidInput(format!("Invalid exploration history appendix: {error}"))
        })?;
    if session.format_version > EXPLORATION_FORMAT_VERSION {
        return Err(AppError::InvalidInput(format!(
            "Unsupported exploration format version {}",
            session.format_version
        )));
    }
    Ok(session)
}

/// Imports a history as a new, completed (read-only) session.
pub fn import_session(
    data_dir: &Path,
    markdown: &str,
    source_name: Option<String>,
) -> Result<ExplorationSession, AppError> {
    let mut session = parse_import(markdown)?;
    session.id = uuid::Uuid::new_v4().simple().to_string();
    session.origin = SessionOrigin::Imported {
        source_path: source_name
            .map(|name| name.trim().to_string())
            .filter(|name| !name.is_empty())
            .unwrap_or_else(|| "imported.md".to_string()),
    };
    session.status = SessionStatus::Completed;
    session.completed_early = false;
    session.updated_at = chrono::Local::now().to_rfc3339();
    session_store::save_session(data_dir, &session)?;
    Ok(session)
}

/// Reopens an imported (or finished) session so the learner can keep going.
pub async fn continue_session(
    data_dir: &Path,
    session_id: &str,
) -> Result<ExplorationSession, AppError> {
    let mut session = session_store::load_session(data_dir, session_id)?;
    if session.status != SessionStatus::Completed {
        return Ok(session);
    }
    session.status = SessionStatus::Active;
    session.completed_early = false;
    session.active_group = session.groups.len();
    crate::services::exploration::engine::ensure_current_question(data_dir, &mut session).await?;
    session.updated_at = chrono::Local::now().to_rfc3339();
    session_store::save_session(data_dir, &session)?;
    Ok(session)
}

/// Writes the history into the configured note vault under a bare filename.
pub fn save_to_vault(
    data_dir: &Path,
    session: &ExplorationSession,
    file_name: &str,
) -> Result<String, AppError> {
    let settings = config::get_settings_path(data_dir)?;
    let root = settings
        .workspace
        .root_path
        .ok_or_else(|| AppError::InvalidInput("No note root configured".to_string()))?;
    let mut name = file_name.trim().to_string();
    if name.is_empty() || name.contains('/') || name.contains('\\') || name.contains("..") {
        return Err(AppError::InvalidInput(
            "File name must be a bare name without path separators".to_string(),
        ));
    }
    if !name.ends_with(".md") {
        name.push_str(".md");
    }
    let target = Path::new(&root).join(&name);
    if target.exists() {
        return Err(AppError::InvalidInput(format!(
            "{} already exists in the note vault",
            name
        )));
    }
    std::fs::write(&target, render_markdown(session))
        .map_err(|error| AppError::Internal(format!("Failed to save learning history: {error}")))?;
    Ok(target.to_string_lossy().to_string())
}

/// Writes the history to `<data>/exploration/exports/`.
pub fn export_session(
    data_dir: &Path,
    session: &ExplorationSession,
) -> Result<ExportResult, AppError> {
    let dir = data_dir.join("exploration").join("exports");
    std::fs::create_dir_all(&dir).map_err(|error| {
        AppError::Internal(format!("Failed to create export directory: {error}"))
    })?;
    let stamp = chrono::Local::now().format("%Y%m%d-%H%M").to_string();
    let file_name = format!("{}-{}.md", slugify(&session.title), stamp);
    let path = dir.join(&file_name);
    let markdown = render_markdown(session);
    std::fs::write(&path, &markdown)
        .map_err(|error| AppError::Internal(format!("Failed to write export: {error}")))?;
    Ok(ExportResult {
        path: path.to_string_lossy().to_string(),
        file_name,
        markdown,
    })
}

fn slugify(title: &str) -> String {
    let mut slug = String::new();
    for ch in title.chars() {
        if ch.is_alphanumeric() {
            slug.push(ch);
        } else if (ch.is_whitespace() || ch == '-' || ch == '_') && !slug.ends_with('-') {
            slug.push('-');
        }
    }
    let slug = slug.trim_matches('-');
    let slug: String = slug.chars().take(60).collect();
    if slug.is_empty() {
        "exploration".to_string()
    } else {
        slug
    }
}

fn fence_for(content: &str) -> &'static str {
    if content.contains("```") {
        "````"
    } else {
        "```"
    }
}

fn concept_title<'a>(session: &'a ExplorationSession, id: &str) -> &'a str {
    session
        .graph
        .concepts
        .iter()
        .find(|concept| concept.id == id)
        .map(|concept| concept.title.as_str())
        .unwrap_or("Concept")
}

fn concept_depth(session: &ExplorationSession, id: &str) -> u32 {
    session
        .graph
        .concepts
        .iter()
        .find(|concept| concept.id == id)
        .map(|concept| concept.depth)
        .unwrap_or(0)
}

fn concept_refs(session: &ExplorationSession, id: &str) -> Vec<String> {
    session
        .graph
        .concepts
        .iter()
        .find(|concept| concept.id == id)
        .map(|concept| {
            concept
                .source_refs
                .iter()
                .map(|reference| match &reference.heading {
                    Some(heading) => format!("{}#{}", reference.note_path, heading),
                    None => reference.note_path.clone(),
                })
                .collect()
        })
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::exploration::{
        Concept, KnowledgeGraph, NextAction, Question, TurnGroup, TurnResult, Verdict,
        WhiteboardStep,
    };

    fn session() -> ExplorationSession {
        ExplorationSession {
            id: "s1".to_string(),
            title: "Compiler Basics".to_string(),
            format_version: EXPLORATION_FORMAT_VERSION,
            created_at: "2026-10-23T10:00:00+08:00".to_string(),
            updated_at: "2026-10-23T10:30:00+08:00".to_string(),
            status: SessionStatus::Completed,
            origin: SessionOrigin::New,
            baseline: 50,
            source_paths: vec!["notes/compiler.md".to_string()],
            source_digests: vec![],
            graph: KnowledgeGraph {
                concepts: vec![
                    Concept {
                        id: "c1".to_string(),
                        title: "Symbols".to_string(),
                        summary: "Names.".to_string(),
                        depth: 0,
                        prereqs: vec![],
                        source_refs: vec![],
                        covered: true,
                    },
                    Concept {
                        id: "c2".to_string(),
                        title: "Lexing".to_string(),
                        summary: "Tokens.".to_string(),
                        depth: 1,
                        prereqs: vec!["c1".to_string()],
                        source_refs: vec![],
                        covered: false,
                    },
                ],
                domains: vec![],
                root_order: vec!["c1".to_string(), "c2".to_string()],
                domains_confirmed: true,
            },
            groups: vec![TurnGroup {
                concept_id: "c1".to_string(),
                question: Question {
                    prompt_md: "Why symbols?".to_string(),
                    hint_used: false,
                },
                answer: Some("Because names.".to_string()),
                result: Some(TurnResult {
                    verdict: Verdict::Correct,
                    feedback_md: "Good.".to_string(),
                    next_action: NextAction::Advance,
                    hint: None,
                }),
                whiteboard: Some(WhiteboardStep {
                    id: "wb1".to_string(),
                    concept_id: "c1".to_string(),
                    title: "Symbols".to_string(),
                    content_md: "| a | b |".to_string(),
                    group_index: 0,
                }),
                covered_after: vec!["c1".to_string()],
            }],
            active_group: 1,
            asides: vec![crate::models::exploration::AsideTurn {
                id: "a1".to_string(),
                question: "Why tokens?".to_string(),
                answer_md: "Because lexing.".to_string(),
                group_index: 0,
                created_at: "2026-10-23T10:05:00+08:00".to_string(),
            }],
            skipped: vec![],
            domain_warning: None,
            completed_early: true,
        }
    }

    #[test]
    fn render_contains_every_section_and_appendix() {
        let markdown = render_markdown(&session());

        assert!(markdown.starts_with("---\nknowtequiz: exploration/1"));
        assert!(markdown.contains("# 知识脉络"));
        assert!(markdown.contains("- Symbols (depth 0)"));
        assert!(markdown.contains("  - Lexing (depth 1)"));
        assert!(markdown.contains("## 01 · Symbols"));
        assert!(markdown.contains("**问**：Why symbols?"));
        assert!(markdown.contains("**答**：Because names."));
        assert!(markdown.contains("## 旁支 · Why tokens?"));
        assert!(markdown.contains("# 未覆盖"));
        assert!(markdown.contains("- [ ] Lexing"));
        assert!(markdown.contains("flowchart TD"));
        assert!(markdown.contains("c1 --> c2"));
        assert!(markdown.contains(SESSION_MARKER));
    }

    #[test]
    fn export_round_trips_through_import() {
        let markdown = render_markdown(&session());
        let restored = parse_import(&markdown).expect("history should parse");

        assert_eq!(restored.title, "Compiler Basics");
        assert_eq!(restored.graph.concepts.len(), 2);
        assert_eq!(restored.groups.len(), 1);
        assert_eq!(restored.groups[0].answer.as_deref(), Some("Because names."));
        assert_eq!(restored.asides.len(), 1);
        assert_eq!(restored.active_group, 1);
        assert!(restored.completed_early);
    }

    #[test]
    fn parse_import_rejects_foreign_markdown() {
        let error = parse_import("# Just a note\n\nNothing here.").expect_err("must reject");
        assert!(error.to_string().contains("missing session appendix"));

        let error =
            parse_import("<!-- knowtequiz:session\n{not json}\n-->").expect_err("must reject");
        assert!(error
            .to_string()
            .contains("Invalid exploration history appendix"));
    }

    #[test]
    fn import_session_creates_a_completed_copy() {
        let dir = std::env::temp_dir()
            .join("knowtequiz-exploration-export-tests")
            .join("import_session_creates_a_completed_copy")
            .join(uuid::Uuid::new_v4().to_string());
        std::fs::create_dir_all(&dir).expect("temp dir");
        let markdown = render_markdown(&session());

        let imported =
            import_session(&dir, &markdown, Some("friend.md".to_string())).expect("import");

        assert_ne!(imported.id, "s1");
        assert_eq!(imported.status, SessionStatus::Completed);
        assert_eq!(
            imported.origin,
            SessionOrigin::Imported {
                source_path: "friend.md".to_string()
            }
        );
        let loaded = session_store::load_session(&dir, &imported.id).expect("persisted");
        assert_eq!(loaded.groups.len(), 1);
    }

    #[test]
    fn export_session_writes_a_markdown_file() {
        let dir = std::env::temp_dir()
            .join("knowtequiz-exploration-export-tests")
            .join("export_session_writes_a_markdown_file")
            .join(uuid::Uuid::new_v4().to_string());
        std::fs::create_dir_all(&dir).expect("temp dir");

        let result = export_session(&dir, &session()).expect("export should succeed");

        assert!(result.file_name.ends_with(".md"));
        assert!(Path::new(&result.path).exists());
        assert!(result.markdown.contains(SESSION_MARKER));
    }

    #[test]
    fn save_to_vault_rejects_traversal_and_existing_files() {
        let dir = std::env::temp_dir()
            .join("knowtequiz-exploration-export-tests")
            .join("save_to_vault_rejects_traversal_and_existing_files")
            .join(uuid::Uuid::new_v4().to_string());
        let notes = dir.join("notes");
        std::fs::create_dir_all(&notes).expect("notes dir");
        std::fs::write(
            dir.join("settings.json"),
            serde_json::json!({
                "version": "1.0.0",
                "ui_language": "en",
                "llm": { "provider": "x", "base_url": "", "api_key": "", "model": "m", "max_tokens": 1, "temperature": 0.1 },
                "ui": { "layout": { "left_visible": true, "right_visible": true, "left_width": 280, "right_width": 360 } },
                "quiz": { "default_types": [], "default_language": "en", "default_count": 1, "default_mode": "basic", "default_difficulty": "medium", "prompt_template": "default" },
                "workspace": { "root_path": notes.to_string_lossy() }
            })
            .to_string(),
        )
        .expect("settings");

        assert!(save_to_vault(&dir, &session(), "../escape.md").is_err());

        let path = save_to_vault(&dir, &session(), "history").expect("save should succeed");
        assert!(path.ends_with("history.md"));
        assert!(save_to_vault(&dir, &session(), "history.md").is_err());
    }

    #[test]
    fn slugify_keeps_readable_names() {
        assert_eq!(slugify("Compiler Basics"), "Compiler-Basics");
        assert_eq!(slugify("  "), "exploration");
    }
}
