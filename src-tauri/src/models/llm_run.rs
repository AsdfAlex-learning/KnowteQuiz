use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Kind of LLM call that produced a run record. Serialized snake_case so the
/// JSONL on disk matches what external tools expect (see `docs/llm-runs-schema.md`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LlmRunKind {
    Quiz,
    QuizFixup,
    DiagnosisInitial,
    DiagnosisFollowup,
    DiagnosisReport,
    ConnectionTest,
    Probe,
}

impl LlmRunKind {
    pub fn as_str(&self) -> &'static str {
        match self {
            LlmRunKind::Quiz => "quiz",
            LlmRunKind::QuizFixup => "quiz_fixup",
            LlmRunKind::DiagnosisInitial => "diagnosis_initial",
            LlmRunKind::DiagnosisFollowup => "diagnosis_followup",
            LlmRunKind::DiagnosisReport => "diagnosis_report",
            LlmRunKind::ConnectionTest => "connection_test",
            LlmRunKind::Probe => "probe",
        }
    }
}

/// Token usage reported by the endpoint. Every field is optional because some
/// OpenAI-compatible endpoints omit or partially fill `usage`.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct LlmUsage {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub prompt_tokens: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub completion_tokens: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub total_tokens: Option<u32>,
}

impl LlmUsage {
    pub fn is_empty(&self) -> bool {
        self.prompt_tokens.is_none()
            && self.completion_tokens.is_none()
            && self.total_tokens.is_none()
    }
}

/// How (or whether) token usage was obtained.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LlmUsageSource {
    /// Parsed from a non-streaming response body's `usage` object.
    Body,
    /// Parsed from an SSE chunk when the endpoint honoured `stream_options.include_usage`.
    StreamOptions,
    /// The endpoint did not report usage (or we chose not to request it).
    Unavailable,
}

/// Full record for one LLM call, appended to `llm_runs/llm_runs.jsonl`.
///
/// `prompt_file` / `response_file` are bare filenames inside `<data_dir>/debug/`
/// (not paths) so a consumer can build its own URL. They are `None` when
/// `record_llm_io` is disabled or when the file could not be written.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmRunRecord {
    pub id: String,
    pub kind: LlmRunKind,
    pub ts_start: String,
    pub ts_end: String,
    pub duration_ms: u64,
    pub model: String,
    /// Host and port of the configured endpoint (e.g. `localhost:11434`).
    /// The API key is **never** stored.
    pub endpoint_host: String,
    pub request_params: Value,
    pub usage: Option<LlmUsage>,
    pub usage_source: LlmUsageSource,
    pub http_status: Option<u16>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub prompt_file: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub response_file: Option<String>,
}

/// List-view projection: everything except the large prompt/response blobs.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmRunSummary {
    pub id: String,
    pub kind: LlmRunKind,
    pub ts_start: String,
    pub ts_end: String,
    pub duration_ms: u64,
    pub model: String,
    pub usage: Option<LlmUsage>,
    pub usage_source: LlmUsageSource,
    pub http_status: Option<u16>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

impl From<&LlmRunRecord> for LlmRunSummary {
    fn from(r: &LlmRunRecord) -> Self {
        LlmRunSummary {
            id: r.id.clone(),
            kind: r.kind,
            ts_start: r.ts_start.clone(),
            ts_end: r.ts_end.clone(),
            duration_ms: r.duration_ms,
            model: r.model.clone(),
            usage: r.usage.clone(),
            usage_source: r.usage_source,
            http_status: r.http_status,
            error: r.error.clone(),
        }
    }
}

/// Derive the host:port from an endpoint URL without ever touching credentials.
/// Falls back to a best-effort parse if the string is not a valid absolute URL
/// (never an error — this is purely metadata). Deliberately dependency-free:
/// no `url` crate is added for this.
pub fn endpoint_host_of(base_url: &str) -> String {
    let trimmed = base_url.trim();
    let after_scheme = trimmed
        .strip_prefix("https://")
        .or_else(|| trimmed.strip_prefix("http://"))
        .unwrap_or(trimmed);
    let authority = after_scheme.split(['/', '?', '#']).next().unwrap_or("");
    // Strip userinfo (user:pass@host) if present.
    let host = authority.rsplit('@').next().unwrap_or("");
    host.trim().to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn llm_run_kind_serializes_snake_case() {
        assert_eq!(
            serde_json::to_string(&LlmRunKind::Quiz).unwrap(),
            "\"quiz\""
        );
        assert_eq!(
            serde_json::to_string(&LlmRunKind::QuizFixup).unwrap(),
            "\"quiz_fixup\""
        );
        assert_eq!(
            serde_json::to_string(&LlmRunKind::DiagnosisInitial).unwrap(),
            "\"diagnosis_initial\""
        );
        assert_eq!(
            serde_json::to_string(&LlmRunKind::DiagnosisFollowup).unwrap(),
            "\"diagnosis_followup\""
        );
        assert_eq!(
            serde_json::to_string(&LlmRunKind::DiagnosisReport).unwrap(),
            "\"diagnosis_report\""
        );
        assert_eq!(
            serde_json::to_string(&LlmRunKind::ConnectionTest).unwrap(),
            "\"connection_test\""
        );
        assert_eq!(
            serde_json::to_string(&LlmRunKind::Probe).unwrap(),
            "\"probe\""
        );
    }

    #[test]
    fn llm_run_kind_round_trips() {
        for kind in [
            LlmRunKind::Quiz,
            LlmRunKind::QuizFixup,
            LlmRunKind::DiagnosisInitial,
            LlmRunKind::DiagnosisFollowup,
            LlmRunKind::DiagnosisReport,
            LlmRunKind::ConnectionTest,
            LlmRunKind::Probe,
        ] {
            let s = serde_json::to_string(&kind).unwrap();
            let back: LlmRunKind = serde_json::from_str(&s).unwrap();
            assert_eq!(kind, back);
        }
    }

    #[test]
    fn llm_usage_source_serializes_snake_case() {
        assert_eq!(
            serde_json::to_string(&LlmUsageSource::Body).unwrap(),
            "\"body\""
        );
        assert_eq!(
            serde_json::to_string(&LlmUsageSource::StreamOptions).unwrap(),
            "\"stream_options\""
        );
        assert_eq!(
            serde_json::to_string(&LlmUsageSource::Unavailable).unwrap(),
            "\"unavailable\""
        );
    }

    #[test]
    fn endpoint_host_strips_scheme_and_path() {
        assert_eq!(
            endpoint_host_of("http://localhost:11434/v1"),
            "localhost:11434"
        );
        assert_eq!(
            endpoint_host_of("https://api.example.com"),
            "api.example.com"
        );
        assert_eq!(endpoint_host_of("http://127.0.0.1:8080"), "127.0.0.1:8080");
    }

    #[test]
    fn endpoint_host_never_contains_api_key() {
        // A base_url that (mis)used query credentials must not leak the key.
        let host = endpoint_host_of("http://localhost:11434/v1?api_key=SECRET");
        assert!(!host.contains("SECRET"), "host leaked credentials: {host}");
    }

    #[test]
    fn endpoint_host_strips_userinfo_credentials() {
        let host = endpoint_host_of("http://user:pass@localhost:11434/v1");
        assert_eq!(host, "localhost:11434");
        assert!(!host.contains("pass"), "host leaked credentials: {host}");
    }

    #[test]
    fn summary_omits_prompt_and_response_fields() {
        let rec = LlmRunRecord {
            id: "abc".into(),
            kind: LlmRunKind::Quiz,
            ts_start: "s".into(),
            ts_end: "e".into(),
            duration_ms: 10,
            model: "m".into(),
            endpoint_host: "localhost".into(),
            request_params: Value::Null,
            usage: None,
            usage_source: LlmUsageSource::Unavailable,
            http_status: Some(200),
            error: None,
            prompt_file: Some("p.txt".into()),
            response_file: Some("r.txt".into()),
        };
        let json = serde_json::to_string(&LlmRunSummary::from(&rec)).unwrap();
        assert!(
            !json.contains("prompt_file"),
            "summary leaked prompt_file: {json}"
        );
        assert!(
            !json.contains("response_file"),
            "summary leaked response_file: {json}"
        );
        assert!(
            !json.contains("request_params"),
            "summary leaked request_params: {json}"
        );
    }

    #[test]
    fn llm_usage_empty_detection() {
        assert!(LlmUsage::default().is_empty());
        assert!(!LlmUsage {
            prompt_tokens: Some(1),
            completion_tokens: None,
            total_tokens: None,
        }
        .is_empty());
    }
}
