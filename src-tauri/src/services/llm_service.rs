use crate::errors::AppError;
use crate::models::settings::LlmConfig;
use reqwest::StatusCode;
use serde::{Deserialize, Serialize};
use std::sync::LazyLock;
use std::time::Duration;

/// Global HTTP client with connection pooling.
/// `timeout` caps whole requests; suitable for non-streaming LLM calls.
static HTTP_CLIENT: LazyLock<reqwest::Client> = LazyLock::new(|| {
    reqwest::Client::builder()
        .connect_timeout(Duration::from_secs(10))
        .timeout(Duration::from_secs(60))
        .pool_max_idle_per_host(10)
        .build()
        .expect("Failed to create HTTP client")
});

/// Client for streaming LLM calls: no total deadline, because local models
/// can legitimately think for minutes. Callers enforce a per-chunk idle
/// timeout instead (`LLM_STREAM_IDLE_TIMEOUT` in quiz_engine).
static STREAM_HTTP_CLIENT: LazyLock<reqwest::Client> = LazyLock::new(|| {
    reqwest::Client::builder()
        .connect_timeout(Duration::from_secs(10))
        .pool_max_idle_per_host(10)
        .build()
        .expect("Failed to create streaming HTTP client")
});

pub fn http_client() -> &'static reqwest::Client {
    &HTTP_CLIENT
}

pub fn streaming_http_client() -> &'static reqwest::Client {
    &STREAM_HTTP_CLIENT
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ConnectionTestResult {
    pub ok: bool,
    pub kind: String,
    pub message: String,
    pub status: Option<u16>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmCapabilities {
    pub available_models: Vec<String>,
    pub supports_streaming: bool,
    pub supports_response_format: bool,
    pub default_model: String,
}

/// Extracts model ids from an OpenAI-compatible `/models` response.
pub fn parse_model_ids(json: &serde_json::Value) -> Vec<String> {
    let mut models: Vec<String> = json["data"]
        .as_array()
        .map(|items| {
            items
                .iter()
                .filter_map(|model| model["id"].as_str())
                .map(str::to_string)
                .collect()
        })
        .unwrap_or_default();
    models.sort();
    models.dedup();
    models
}

/// Fetches the provider's model list without probing streaming/JSON support.
/// This is what the settings form calls when the user presses "Fetch models".
pub async fn list_models(llm: &LlmConfig) -> Result<Vec<String>, AppError> {
    let base = llm.base_url.trim_end_matches('/');
    if base.is_empty() {
        return Err(AppError::InvalidInput(
            "No LLM endpoint configured".to_string(),
        ));
    }

    let resp = http_client()
        .get(format!("{}/models", base))
        .header("Authorization", format!("Bearer {}", llm.api_key))
        .send()
        .await
        .map_err(|e| AppError::Llm(format!("Failed to fetch models: {e}")))?;

    if !resp.status().is_success() {
        let status = resp.status();
        let body = resp.text().await.unwrap_or_default();
        return Err(AppError::Llm(format!("LLM API error {status}: {body}")));
    }

    let body = resp
        .text()
        .await
        .map_err(|e| AppError::Llm(format!("Failed to read models response: {e}")))?;
    let json: serde_json::Value = serde_json::from_str(&body)
        .map_err(|e| AppError::Llm(format!("Failed to parse models response: {e}")))?;
    Ok(parse_model_ids(&json))
}

pub async fn probe_capabilities(llm: &LlmConfig) -> Result<LlmCapabilities, AppError> {
    let base = llm.base_url.trim_end_matches('/');
    let client = http_client();
    let mut available_models: Vec<String> = vec![];
    let mut supports_streaming = false;
    let mut supports_response_format = false;

    // Probe models endpoint
    let resp = client
        .get(format!("{}/models", base))
        .header("Authorization", format!("Bearer {}", llm.api_key))
        .send()
        .await
        .map_err(|e| AppError::Llm(format!("Failed to probe models endpoint: {}", e)))?;
    if resp.status().is_success() {
        let body = resp
            .text()
            .await
            .map_err(|e| AppError::Llm(format!("Failed to read models response: {}", e)))?;
        if let Ok(json) = serde_json::from_str::<serde_json::Value>(&body) {
            available_models = parse_model_ids(&json);
            available_models.truncate(20); // cap at 20 models
        }
    }

    // Probe streaming support
    let stream_body = serde_json::json!({
        "model": llm.model,
        "messages": [{"role": "user", "content": "say ok"}],
        "stream": true,
        "max_tokens": 5,
    });
    let resp = client
        .post(format!("{}/chat/completions", base))
        .header("Authorization", format!("Bearer {}", llm.api_key))
        .header("Content-Type", "application/json")
        .json(&stream_body)
        .send()
        .await
        .map_err(|e| AppError::Llm(format!("Failed to probe streaming: {}", e)))?;
    if resp.status().is_success() {
        let text = resp.text().await.unwrap_or_default();
        // Streaming responses contain "data:" lines
        supports_streaming = text.contains("data:");
    }

    // Probe response_format support
    let json_body = serde_json::json!({
        "model": llm.model,
        "messages": [{"role": "user", "content": "say ok"}],
        "response_format": {"type": "json_object"},
        "max_tokens": 10,
    });
    let resp = client
        .post(format!("{}/chat/completions", base))
        .header("Authorization", format!("Bearer {}", llm.api_key))
        .header("Content-Type", "application/json")
        .json(&json_body)
        .send()
        .await
        .map_err(|e| AppError::Llm(format!("Failed to probe response_format: {}", e)))?;
    if resp.status().is_success() {
        if let Ok(body) = resp.text().await {
            // Check if response is valid JSON (indicates response_format worked)
            supports_response_format = serde_json::from_str::<serde_json::Value>(&body).is_ok();
        }
    }

    Ok(LlmCapabilities {
        available_models,
        supports_streaming,
        supports_response_format,
        default_model: llm.model.clone(),
    })
}

pub async fn test_connection(llm: &LlmConfig) -> Result<ConnectionTestResult, AppError> {
    let request_body = serde_json::json!({
        "model": llm.model,
        "messages": [
            { "role": "user", "content": "hi" }
        ],
        "max_tokens": 5,
    });

    let response = http_client()
        .post(format!(
            "{}/chat/completions",
            llm.base_url.trim_end_matches('/')
        ))
        .header("Authorization", format!("Bearer {}", llm.api_key))
        .header("Content-Type", "application/json")
        .json(&request_body)
        .send()
        .await
        .map_err(|e| AppError::Llm(format!("Could not reach LLM endpoint: {}", e)))?;

    let status = response.status();
    let body = response.text().await.unwrap_or_default();
    Ok(classify_connection_status(status, &body))
}

pub fn classify_connection_status(status: StatusCode, body: &str) -> ConnectionTestResult {
    if status.is_success() {
        return ConnectionTestResult {
            ok: true,
            kind: "ok".to_string(),
            message: "Connection successful".to_string(),
            status: Some(status.as_u16()),
        };
    }

    let body_lower = body.to_ascii_lowercase();
    let (kind, message) = if status == StatusCode::UNAUTHORIZED || status == StatusCode::FORBIDDEN {
        ("auth", "LLM endpoint rejected the API key or permissions")
    } else if status == StatusCode::NOT_FOUND || body_lower.contains("model") {
        ("model", "LLM endpoint or configured model was not found")
    } else if status.is_server_error() {
        ("server", "LLM endpoint returned a server error")
    } else {
        ("unknown", "LLM endpoint returned an unexpected response")
    };

    let detail = body.trim();
    let message = if detail.is_empty() {
        message.to_string()
    } else {
        format!("{}: {}", message, detail)
    };

    ConnectionTestResult {
        ok: false,
        kind: kind.to_string(),
        message,
        status: Some(status.as_u16()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use reqwest::StatusCode;

    #[test]
    fn parse_model_ids_sorts_dedupes_and_ignores_malformed_entries() {
        let json = serde_json::json!({
            "data": [
                { "id": "gpt-4o-mini" },
                { "id": "qwen2.5:7b" },
                { "id": "gpt-4o-mini" },
                { "object": "model" },
                { "id": 42 }
            ]
        });

        assert_eq!(
            parse_model_ids(&json),
            vec!["gpt-4o-mini".to_string(), "qwen2.5:7b".to_string()]
        );
    }

    #[test]
    fn parse_model_ids_returns_empty_for_unexpected_shapes() {
        assert!(parse_model_ids(&serde_json::json!({})).is_empty());
        assert!(parse_model_ids(&serde_json::json!({ "data": "nope" })).is_empty());
    }

    #[test]
    fn classify_connection_status_identifies_auth_failures() {
        let result = classify_connection_status(StatusCode::UNAUTHORIZED, "bad api key");

        assert!(!result.ok);
        assert_eq!(result.kind, "auth");
        assert!(result.message.contains("API key"));
    }

    #[test]
    fn classify_connection_status_identifies_missing_models() {
        let result = classify_connection_status(StatusCode::NOT_FOUND, "model not found");

        assert!(!result.ok);
        assert_eq!(result.kind, "model");
        assert!(result.message.contains("model"));
    }

    #[test]
    fn classify_connection_status_reports_success() {
        let result = classify_connection_status(StatusCode::OK, "");

        assert!(result.ok);
        assert_eq!(result.kind, "ok");
    }

    #[test]
    fn classify_connection_status_identifies_server_errors() {
        let result =
            classify_connection_status(StatusCode::INTERNAL_SERVER_ERROR, "internal error");

        assert!(!result.ok);
        assert_eq!(result.kind, "server");
        assert!(result.message.contains("server error"));
    }

    #[test]
    fn classify_connection_status_identifies_forbidden() {
        let result = classify_connection_status(StatusCode::FORBIDDEN, "access denied");

        assert!(!result.ok);
        assert_eq!(result.kind, "auth");
        assert!(result.message.contains("API key"));
    }

    #[test]
    fn classify_connection_status_identifies_network_errors() {
        let result = classify_connection_status(StatusCode::BAD_GATEWAY, "bad gateway");

        assert!(!result.ok);
        assert_eq!(result.kind, "server");
    }

    #[test]
    fn classify_connection_status_handles_unknown_status() {
        let result = classify_connection_status(StatusCode::from_u16(418).unwrap(), "teapot");

        assert!(!result.ok);
        assert_eq!(result.kind, "unknown");
    }
}
