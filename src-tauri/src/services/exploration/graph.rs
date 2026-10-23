use crate::errors::AppError;
use crate::models::exploration::{Concept, Domain, KnowledgeGraph, NoteDigest, SourceRef};
use serde_json::Value;
use std::collections::{HashMap, HashSet, VecDeque};

pub const MAX_CONCEPTS: usize = 60;
pub const MAX_SOURCE_REFS: usize = 5;
pub const MAX_SNIPPET_CHARS: usize = 200;
pub const MAX_DIGEST_SUMMARY_CHARS: usize = 400;

/// Extracts a JSON object from a model response that may be wrapped in a
/// markdown fence or surrounded by prose.
pub fn parse_json_object(raw: &str) -> Result<Value, AppError> {
    let trimmed = raw.trim();
    if let Ok(value) = serde_json::from_str::<Value>(trimmed) {
        return Ok(value);
    }

    let without_fence = trimmed
        .trim_start_matches("```json")
        .trim_start_matches("```JSON")
        .trim_start_matches("```")
        .trim_end_matches("```")
        .trim();
    if let Ok(value) = serde_json::from_str::<Value>(without_fence) {
        return Ok(value);
    }

    if let (Some(start), Some(end)) = (trimmed.find('{'), trimmed.rfind('}')) {
        if end > start {
            if let Ok(value) = serde_json::from_str::<Value>(&trimmed[start..=end]) {
                return Ok(value);
            }
        }
    }

    Err(AppError::Llm(
        "Model response did not contain a JSON object".to_string(),
    ))
}

/// Best-effort digest used when the summary call fails or returns garbage.
/// Keeps initialization moving because the graph stage only needs headings and
/// key phrases, not a perfect abstract.
pub fn fallback_digest(path: &str, content: &str) -> NoteDigest {
    let body = crate::services::note_service::extract_body_content(content);

    let mut headings = Vec::new();
    for line in body.lines() {
        let trimmed = line.trim_start();
        if let Some(rest) = trimmed.strip_prefix('#') {
            let heading = rest.trim_start_matches('#').trim();
            if !heading.is_empty() {
                headings.push(heading.to_string());
                if headings.len() >= 20 {
                    break;
                }
            }
        }
    }

    let title = headings.first().cloned().unwrap_or_else(|| {
        path.rsplit(['/', '\\'])
            .next()
            .unwrap_or(path)
            .trim_end_matches(".md")
            .trim_end_matches(".markdown")
            .to_string()
    });

    let body_text = body
        .lines()
        .filter(|line| !line.trim_start().starts_with('#'))
        .collect::<Vec<_>>()
        .join(" ");
    let summary: String = body_text
        .chars()
        .take(MAX_DIGEST_SUMMARY_CHARS)
        .collect::<String>()
        .trim()
        .to_string();

    NoteDigest {
        path: path.to_string(),
        title,
        headings,
        summary,
        key_terms: Vec::new(),
        domain_guess: String::new(),
    }
}

pub fn parse_digest(value: &Value, path: &str, content: &str) -> NoteDigest {
    let fallback = fallback_digest(path, content);
    NoteDigest {
        path: path.to_string(),
        title: non_empty(value.get("title")).unwrap_or(fallback.title),
        headings: string_array(value.get("headings")),
        summary: non_empty(value.get("summary")).unwrap_or(fallback.summary),
        key_terms: string_array(value.get("key_terms")),
        domain_guess: non_empty(value.get("domain_guess")).unwrap_or_default(),
    }
}

struct RawConcept {
    title: String,
    summary: String,
    prereqs: Vec<String>,
    refs: Vec<SourceRef>,
}

/// Validates and normalizes a model-produced graph.
///
/// The model only supplies titles; ids, depth, prereq wiring, and display order
/// are all derived here so a plausible-looking but inconsistent response can
/// never corrupt the timeline.
pub fn normalize_graph(
    value: &Value,
    allowed_paths: &[String],
) -> Result<KnowledgeGraph, AppError> {
    let raws = collect_raw_concepts(value, allowed_paths)?;

    let mut by_title: HashMap<String, usize> = HashMap::new();
    for (index, raw) in raws.iter().enumerate() {
        by_title.insert(normalize_title(&raw.title), index);
    }

    let mut prereqs: Vec<Vec<usize>> = Vec::with_capacity(raws.len());
    for (index, raw) in raws.iter().enumerate() {
        let mut deps = Vec::new();
        for prereq in &raw.prereqs {
            let Some(&target) = by_title.get(&normalize_title(prereq)) else {
                continue;
            };
            if target != index && !deps.contains(&target) {
                deps.push(target);
            }
        }
        prereqs.push(deps);
    }

    let order = topological_order(&mut prereqs);

    // Ids follow the learning order (`c1` is learned first), so the persisted
    // graph reads the same way as the UI shows it.
    let mut id_by_raw: Vec<String> = vec![String::new(); raws.len()];
    for (position, &raw_index) in order.iter().enumerate() {
        id_by_raw[raw_index] = format!("c{}", position + 1);
    }

    let mut depth = vec![0u32; raws.len()];
    for &index in &order {
        depth[index] = prereqs[index]
            .iter()
            .map(|&dep| depth[dep] + 1)
            .max()
            .unwrap_or(0);
    }

    let concepts: Vec<Concept> = order
        .iter()
        .map(|&index| Concept {
            id: id_by_raw[index].clone(),
            title: raws[index].title.clone(),
            summary: raws[index].summary.clone(),
            depth: depth[index],
            prereqs: prereqs[index]
                .iter()
                .map(|&dep| id_by_raw[dep].clone())
                .collect(),
            source_refs: raws[index].refs.clone(),
            covered: false,
        })
        .collect();

    let domains = collect_domains(value, &by_title, &id_by_raw);
    let root_order = concepts.iter().map(|concept| concept.id.clone()).collect();

    Ok(KnowledgeGraph {
        concepts,
        domains,
        root_order,
        domains_confirmed: false,
    })
}

fn collect_raw_concepts(
    value: &Value,
    allowed_paths: &[String],
) -> Result<Vec<RawConcept>, AppError> {
    let array = value
        .get("concepts")
        .and_then(|concepts| concepts.as_array())
        .ok_or_else(|| {
            AppError::Llm("Knowledge graph is missing the concepts array".to_string())
        })?;

    let mut concepts: Vec<RawConcept> = Vec::new();
    let mut index_by_title: HashMap<String, usize> = HashMap::new();

    for item in array {
        let title = non_empty(item.get("title")).unwrap_or_default();
        if title.is_empty() {
            continue;
        }
        let key = normalize_title(&title);
        let summary = non_empty(item.get("summary")).unwrap_or_default();
        let prereqs = string_array(item.get("prereqs"));
        let refs = parse_refs(item.get("source_refs"), allowed_paths);

        if let Some(&existing) = index_by_title.get(&key) {
            let target = &mut concepts[existing];
            if target.summary.is_empty() {
                target.summary = summary;
            }
            for prereq in prereqs {
                if !target.prereqs.contains(&prereq) {
                    target.prereqs.push(prereq);
                }
            }
            for source_ref in refs {
                if !target
                    .refs
                    .iter()
                    .any(|existing| same_ref(existing, &source_ref))
                {
                    target.refs.push(source_ref);
                }
            }
            target.refs.truncate(MAX_SOURCE_REFS);
        } else {
            index_by_title.insert(key, concepts.len());
            concepts.push(RawConcept {
                title,
                summary,
                prereqs,
                refs,
            });
        }

        if concepts.len() >= MAX_CONCEPTS {
            break;
        }
    }

    if concepts.is_empty() {
        return Err(AppError::Llm(
            "Knowledge graph contains no usable concepts".to_string(),
        ));
    }
    Ok(concepts)
}

fn collect_domains(
    value: &Value,
    by_title: &HashMap<String, usize>,
    id_by_raw: &[String],
) -> Vec<Domain> {
    let Some(array) = value.get("domains").and_then(|domains| domains.as_array()) else {
        return Vec::new();
    };

    let mut domains: Vec<Domain> = Vec::new();
    for item in array {
        let mut concept_ids: Vec<String> = Vec::new();
        for title in string_array(item.get("concept_titles")) {
            let Some(&index) = by_title.get(&normalize_title(&title)) else {
                continue;
            };
            let id = id_by_raw[index].clone();
            if !concept_ids.contains(&id) {
                concept_ids.push(id);
            }
        }
        if concept_ids.is_empty() {
            continue;
        }
        let title =
            non_empty(item.get("title")).unwrap_or_else(|| format!("Domain {}", domains.len() + 1));
        domains.push(Domain {
            id: format!("d{}", domains.len() + 1),
            title,
            concept_ids,
        });
    }
    domains.sort_by(|a, b| a.id.cmp(&b.id));
    domains
}

fn topological_order(prereqs: &mut [Vec<usize>]) -> Vec<usize> {
    loop {
        let order = kahn(prereqs);
        if order.len() == prereqs.len() {
            return order;
        }
        // A cycle survived: drop the prerequisites of the nodes that could not
        // be ordered, then try again. Deterministic because node order is.
        let settled: HashSet<usize> = order.into_iter().collect();
        for (index, deps) in prereqs.iter_mut().enumerate() {
            if !settled.contains(&index) {
                deps.clear();
            }
        }
    }
}

fn kahn(prereqs: &[Vec<usize>]) -> Vec<usize> {
    let count = prereqs.len();
    let mut indegree: Vec<usize> = prereqs.iter().map(Vec::len).collect();
    let mut dependents: Vec<Vec<usize>> = vec![Vec::new(); count];
    for (index, deps) in prereqs.iter().enumerate() {
        for &dep in deps {
            dependents[dep].push(index);
        }
    }

    let mut queue: VecDeque<usize> = (0..count).filter(|&index| indegree[index] == 0).collect();
    let mut order = Vec::with_capacity(count);
    while let Some(index) = queue.pop_front() {
        order.push(index);
        for &dependent in &dependents[index] {
            indegree[dependent] -= 1;
            if indegree[dependent] == 0 {
                queue.push_back(dependent);
            }
        }
    }
    order
}

fn parse_refs(value: Option<&Value>, allowed_paths: &[String]) -> Vec<SourceRef> {
    let Some(items) = value.and_then(|v| v.as_array()) else {
        return Vec::new();
    };

    let mut refs = Vec::new();
    for item in items {
        let raw_path = non_empty(item.get("note_path")).unwrap_or_default();
        if raw_path.is_empty() {
            continue;
        }
        let Some(note_path) = resolve_allowed_path(&raw_path, allowed_paths) else {
            continue;
        };
        let snippet: String = non_empty(item.get("snippet"))
            .unwrap_or_default()
            .chars()
            .take(MAX_SNIPPET_CHARS)
            .collect();
        refs.push(SourceRef {
            note_path,
            heading: non_empty(item.get("heading")),
            snippet,
        });
        if refs.len() >= MAX_SOURCE_REFS {
            break;
        }
    }
    refs
}

fn resolve_allowed_path(raw: &str, allowed: &[String]) -> Option<String> {
    if let Some(exact) = allowed.iter().find(|path| path.as_str() == raw) {
        return Some(exact.clone());
    }
    let file = file_name(raw);
    allowed.iter().find(|path| file_name(path) == file).cloned()
}

fn file_name(path: &str) -> String {
    path.rsplit(['/', '\\'])
        .next()
        .unwrap_or(path)
        .to_ascii_lowercase()
}

fn same_ref(left: &SourceRef, right: &SourceRef) -> bool {
    left.note_path == right.note_path
        && left.heading == right.heading
        && left.snippet == right.snippet
}

fn normalize_title(title: &str) -> String {
    title.trim().to_lowercase()
}

fn non_empty(value: Option<&Value>) -> Option<String> {
    value
        .and_then(|v| v.as_str())
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
}

fn string_array(value: Option<&Value>) -> Vec<String> {
    value
        .and_then(|v| v.as_array())
        .map(|items| {
            items
                .iter()
                .filter_map(|item| item.as_str())
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect()
        })
        .unwrap_or_default()
}

/// True when the notes span clearly different subject areas. Returned as a
/// stable code (not prose) so the UI can localize it.
pub fn domain_warning(graph: &KnowledgeGraph) -> Option<String> {
    let total = graph.concepts.len();
    if total < 4 || graph.domains.len() < 2 {
        return None;
    }
    let threshold = ((total as f64) * 0.25).ceil() as usize;
    let qualifying = graph
        .domains
        .iter()
        .filter(|domain| domain.concept_ids.len() >= threshold.max(2))
        .count();
    (qualifying >= 2).then(|| "multiple_domains".to_string())
}

/// Index of the concept the first question should cover.
pub fn first_concept_index(graph: &KnowledgeGraph) -> Option<usize> {
    if let Some(id) = graph.root_order.first() {
        if let Some(index) = graph.concepts.iter().position(|concept| &concept.id == id) {
            return Some(index);
        }
    }
    graph
        .concepts
        .iter()
        .enumerate()
        .min_by_key(|(_, concept)| concept.depth)
        .map(|(index, _)| index)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn allowed() -> Vec<String> {
        vec![
            "C:/notes/compiler.md".to_string(),
            "C:/notes/linker.md".to_string(),
        ]
    }

    fn graph_json() -> Value {
        serde_json::json!({
            "domains": [
                { "title": "Compilers", "concept_titles": ["Symbols", "Lexing", "Linking"] }
            ],
            "concepts": [
                {
                    "title": "Lexing",
                    "summary": "Turns characters into tokens.",
                    "depth": 0,
                    "prereqs": ["Symbols"],
                    "source_refs": [
                        { "note_path": "compiler.md", "heading": "1.2", "snippet": "Tokens" }
                    ]
                },
                {
                    "title": "Symbols",
                    "summary": "Human-facing names.",
                    "depth": 0,
                    "prereqs": [],
                    "source_refs": [
                        { "note_path": "C:/notes/compiler.md", "heading": "1.1", "snippet": "Names" }
                    ]
                },
                {
                    "title": "Linking",
                    "summary": "Fixes addresses.",
                    "depth": 2,
                    "prereqs": ["Lexing", "Unknown"],
                    "source_refs": [
                        { "note_path": "linker.md", "heading": "2", "snippet": "Relocate" },
                        { "note_path": "elsewhere.md", "heading": "x", "snippet": "dropped" }
                    ]
                }
            ],
            "root_order": ["Lexing"]
        })
    }

    #[test]
    fn parses_json_wrapped_in_fences_or_prose() {
        assert!(parse_json_object("```json\n{\"a\":1}\n```").is_ok());
        assert_eq!(
            parse_json_object("Here you go: {\"a\":1} done").unwrap()["a"],
            1
        );
        assert!(parse_json_object("no json here").is_err());
    }

    #[test]
    fn normalize_derives_ids_depth_and_order_from_titles() {
        let graph = normalize_graph(&graph_json(), &allowed()).expect("graph should normalize");

        assert_eq!(graph.concepts.len(), 3);
        // Symbols has no prereqs, so it must come first in the derived order.
        assert_eq!(graph.concepts[0].title, "Symbols");
        assert_eq!(graph.concepts[0].id, "c1");
        assert_eq!(graph.concepts[0].depth, 0);
        assert_eq!(graph.concepts[1].title, "Lexing");
        assert_eq!(graph.concepts[1].prereqs, vec!["c1"]);
        assert_eq!(graph.concepts[1].depth, 1);
        // "Unknown" is dropped, "Linking" keeps only the real prerequisite.
        assert_eq!(graph.concepts[2].title, "Linking");
        assert_eq!(graph.concepts[2].prereqs, vec!["c2"]);
        assert_eq!(graph.concepts[2].depth, 2);
        assert_eq!(graph.root_order, vec!["c1", "c2", "c3"]);
        assert_eq!(graph.domains.len(), 1);
        assert_eq!(graph.domains[0].concept_ids, vec!["c1", "c2", "c3"]);
        assert!(!graph.domains_confirmed);
    }

    #[test]
    fn normalize_drops_unrelated_source_refs_and_resolves_basenames() {
        let graph = normalize_graph(&graph_json(), &allowed()).expect("graph should normalize");

        let linking = &graph.concepts[2];
        assert_eq!(linking.source_refs.len(), 1);
        assert_eq!(linking.source_refs[0].note_path, "C:/notes/linker.md");
    }

    #[test]
    fn normalize_merges_duplicate_titles_and_caps_refs() {
        let duplicate = serde_json::json!({
            "concepts": [
                {
                    "title": "Loop",
                    "summary": "",
                    "prereqs": [],
                    "source_refs": [
                        { "note_path": "compiler.md", "heading": "a", "snippet": "one" },
                        { "note_path": "compiler.md", "heading": "b", "snippet": "two" },
                        { "note_path": "linker.md", "heading": "c", "snippet": "three" },
                        { "note_path": "linker.md", "heading": "d", "snippet": "four" },
                        { "note_path": "linker.md", "heading": "e", "snippet": "five" },
                        { "note_path": "linker.md", "heading": "f", "snippet": "six" }
                    ]
                },
                {
                    "title": " loop ",
                    "summary": "A loop.",
                    "prereqs": [],
                    "source_refs": []
                }
            ]
        });

        let graph = normalize_graph(&duplicate, &allowed()).expect("graph should normalize");

        assert_eq!(graph.concepts.len(), 1);
        assert_eq!(graph.concepts[0].summary, "A loop.");
        assert_eq!(graph.concepts[0].id, "c1");
        assert_eq!(graph.concepts[0].source_refs.len(), MAX_SOURCE_REFS);
    }

    #[test]
    fn normalize_breaks_cycles_instead_of_hanging() {
        let cyclic = serde_json::json!({
            "concepts": [
                { "title": "A", "prereqs": ["B"] },
                { "title": "B", "prereqs": ["A"] }
            ]
        });

        let graph = normalize_graph(&cyclic, &allowed()).expect("graph should normalize");

        assert_eq!(graph.concepts.len(), 2);
        assert_eq!(graph.root_order.len(), 2);
        assert!(graph.concepts.iter().all(|concept| concept.depth == 0));
    }

    #[test]
    fn normalize_rejects_a_graph_without_concepts() {
        let empty = serde_json::json!({ "concepts": [] });
        assert!(normalize_graph(&empty, &allowed()).is_err());

        let missing = serde_json::json!({});
        assert!(normalize_graph(&missing, &allowed()).is_err());
    }

    #[test]
    fn domain_warning_needs_two_material_domains() {
        let mut graph = normalize_graph(&graph_json(), &allowed()).expect("graph should normalize");
        assert_eq!(domain_warning(&graph), None);

        graph.concepts.push(graph.concepts[0].clone());
        graph.domains = vec![
            Domain {
                id: "d1".to_string(),
                title: "Compilers".to_string(),
                concept_ids: graph.concepts.iter().map(|c| c.id.clone()).collect(),
            },
            Domain {
                id: "d2".to_string(),
                title: "Cooking".to_string(),
                concept_ids: vec!["c1".to_string(), "c2".to_string()],
            },
        ];
        assert_eq!(domain_warning(&graph), Some("multiple_domains".to_string()));
    }

    #[test]
    fn fallback_digest_keeps_headings_and_truncates_the_summary() {
        let content = "---\ntitle: Meta\n---\n# Lexing\n\nTurns characters into tokens.\n\n## Tokens\n\nMore text.";
        let digest = fallback_digest("C:/notes/compiler.md", content);

        assert_eq!(digest.title, "Lexing");
        assert_eq!(digest.headings, vec!["Lexing", "Tokens"]);
        assert!(digest.summary.contains("Turns characters into tokens."));
        assert!(!digest.summary.contains("# Lexing"));
        assert_eq!(digest.path, "C:/notes/compiler.md");
    }

    #[test]
    fn first_concept_follows_root_order_then_depth() {
        let mut graph = normalize_graph(&graph_json(), &allowed()).expect("graph should normalize");
        assert_eq!(first_concept_index(&graph), Some(0));

        graph.root_order.clear();
        assert_eq!(first_concept_index(&graph), Some(0));
    }
}
