/// System persona for exploration calls. Unlike the quiz funnel, the output
/// language follows the learner's notes instead of being forced to Chinese.
pub const EXPLORATION_SYSTEM_PROMPT: &str = "You are a rigorous subject tutor who builds a learning path from the learner's own notes. Reply in the language used by those notes. When the instructions ask for JSON, return exactly one JSON object with no markdown fences.";

/// Map stage: one note -> factual digest.
pub fn summary_prompt(note_path: &str, content: &str) -> String {
    format!(
        "Build a factual digest of one note for a cross-document knowledge map.\n\n\
Return exactly one JSON object shaped like this:\n\
{{\n  \"title\": \"note title\",\n  \"headings\": [\"section heading\"],\n  \"summary\": \"3-6 sentences about what this note actually teaches\",\n  \"key_terms\": [\"term\"],\n  \"domain_guess\": \"one broad subject area\"\n}}\n\n\
Rules:\n\
- Base everything only on the note text below; never invent facts.\n\
- Keep the note's own language.\n\
- The summary must be concrete and specific, not a restatement of the title.\n\
- Return the JSON object only.\n\n\
NOTE PATH: {note_path}\n\n\
NOTE:\n{content}"
    )
}

/// Reduce stage: digests -> one cross-document, shallow-to-deep graph.
pub fn graph_prompt(digests_json: &str) -> String {
    format!(
        "Design how a learner should study these notes: shallow foundations first, advanced ideas last, reorganised across documents.\n\n\
Return exactly one JSON object shaped like this:\n\
{{\n  \"domains\": [{{\"title\": \"subject area\", \"concept_titles\": [\"concept title\"]}}],\n  \"concepts\": [{{\n    \"title\": \"concept title\",\n    \"summary\": \"2-4 sentences\",\n    \"depth\": 0,\n    \"prereqs\": [\"other concept title\"],\n    \"source_refs\": [{{\"note_path\": \"path from the input\", \"heading\": \"section heading\", \"snippet\": \"short quote or paraphrase from that note\"}}]\n  }}],\n  \"root_order\": [\"concept title\"]\n}}\n\n\
HARD CONSTRAINTS:\n\
- Do NOT follow the original document order. Order concepts by how they should be learned.\n\
- When two or more notes are provided, at least one concept must combine content from two different notes.\n\
- Every concept needs 1-5 source_refs, and each note_path must come from the input.\n\
- prereqs must reference titles that also appear in concepts; the first concepts have no prereqs.\n\
- Create 8-60 concepts. Keep the source language.\n\
- Return the JSON object only.\n\n\
INPUT DIGESTS (JSON):\n{digests_json}"
    )
}
