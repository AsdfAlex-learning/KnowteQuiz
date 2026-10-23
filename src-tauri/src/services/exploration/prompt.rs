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

/// Opening question for one concept, grounded in its source excerpts.
pub fn question_prompt(context_json: &str) -> String {
    format!(
        "Write the opening question for a guided-learning session.\n\n\
Rules:\n\
- Ground it in the note excerpts in the context below; do not invent facts.\n\
- Ask one concrete question that makes the learner reason, not recall a definition.\n\
- It must be answerable from the excerpts.\n\
- Keep the source language. At most 80 words.\n\
- Return plain text only (Markdown allowed); no JSON, no markdown fences.\n\n\
CONTEXT (JSON):\n{context_json}"
    )
}

/// One combined turn: judge the answer, give feedback, and choose the next step.
pub fn turn_prompt(context_json: &str) -> String {
    format!(
        "Continue a guided-learning session. Judge the learner's answer, give feedback, and choose the next step.\n\n\
Return exactly one JSON object shaped like this:\n\
{{\n  \"verdict\": \"correct | partial | wrong | unknown\",\n  \"feedback\": \"2-4 sentences of specific feedback\",\n  \"next_action\": \"question | hint | advance | complete\",\n  \"hint\": \"directional hint, only when next_action is hint\",\n  \"next_concept_id\": \"id from the concept list\",\n  \"next_question\": \"the next question text\",\n  \"whiteboard_md\": \"Markdown explanation of the concept just discussed\",\n  \"covered_concepts\": [\"concept id\"]\n}}\n\n\
HARD CONSTRAINTS:\n\
- If the learner says they do not know (any phrasing), verdict is unknown and next_action is hint: give ONE directional hint that does not reveal the answer, then ask a smaller sub-question about the same concept.\n\
- If the answer is wrong, next_action is question: keep the same concept and ask from a different angle.\n\
- If the answer shows enough understanding, next_action is advance and covered_concepts must include the current concept.\n\
- Advance at most one concept per turn. next_concept_id must come from the concept list and must not already be covered.\n\
- Every question must be grounded in the source excerpts in the context; do not invent content.\n\
- whiteboard_md: explain the concept just discussed as a short Markdown block (table, list, formula, or diagram-in-text). Keep the source language.\n\
- When every concept is covered, next_action is complete.\n\
- Return the JSON object only.\n\n\
CONTEXT (JSON):\n{context_json}"
    )
}

/// Repair pass used when the turn response is not valid JSON.
pub fn repair_prompt(raw_response: &str, error: &str) -> String {
    format!(
        "The previous response was not valid JSON ({error}).\n\
Rewrite it as exactly one JSON object with these keys: verdict, feedback, next_action, hint, next_concept_id, next_question, whiteboard_md, covered_concepts.\n\
Keep the same meaning; change only the formatting. Return the JSON object only.\n\n\
PREVIOUS RESPONSE:\n{raw_response}"
    )
}

/// `/btw` side question: answer only, never advance the path.
pub fn aside_prompt(context_json: &str) -> String {
    format!(
        "Answer a side question the learner asked while studying.\n\n\
Rules:\n\
- Answer from the note context below; if it is not covered there, say so plainly.\n\
- Do not advance the learning path and do not ask a question back.\n\
- Keep the source language. Be concise.\n\
- Return Markdown only; no JSON, no markdown fences.\n\n\
CONTEXT (JSON):\n{context_json}"
    )
}
