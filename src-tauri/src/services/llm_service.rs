use crate::models::settings::LlmConfig;
use reqwest::StatusCode;
use serde::{Deserialize, Serialize};

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

pub async fn probe_capabilities(llm: &LlmConfig) -> LlmCapabilities {
    let base = llm.base_url.trim_end_matches('/');
    let client = reqwest::Client::new();
    let mut available_models: Vec<String> = vec![];
    let mut supports_streaming = false;
    let mut supports_response_format = false;

    // Probe models endpoint
    if let Ok(resp) = client
        .get(format!("{}/models", base))
        .header("Authorization", format!("Bearer {}", llm.api_key))
        .send()
        .await
    {
        if resp.status().is_success() {
            if let Ok(body) = resp.text().await {
                if let Ok(json) = serde_json::from_str::<serde_json::Value>(&body) {
                    if let Some(models) = json["data"].as_array() {
                        available_models = models
                            .iter()
                            .filter_map(|m| m["id"].as_str().map(|s| s.to_string()))
                            .collect();
                        available_models.sort();
                        available_models.truncate(20); // cap at 20 models
                    }
                }
            }
        }
    }

    // Probe streaming support
    let stream_body = serde_json::json!({
        "model": llm.model,
        "messages": [{"role": "user", "content": "say ok"}],
        "stream": true,
        "max_tokens": 5,
    });
    if let Ok(resp) = client
        .post(format!("{}/chat/completions", base))
        .header("Authorization", format!("Bearer {}", llm.api_key))
        .header("Content-Type", "application/json")
        .json(&stream_body)
        .send()
        .await
    {
        if resp.status().is_success() {
            let text = resp.text().await.unwrap_or_default();
            // Streaming responses contain "data:" lines
            supports_streaming = text.contains("data:");
        }
    }

    // Probe response_format support
    let json_body = serde_json::json!({
        "model": llm.model,
        "messages": [{"role": "user", "content": "say ok"}],
        "response_format": {"type": "json_object"},
        "max_tokens": 10,
    });
    if let Ok(resp) = client
        .post(format!("{}/chat/completions", base))
        .header("Authorization", format!("Bearer {}", llm.api_key))
        .header("Content-Type", "application/json")
        .json(&json_body)
        .send()
        .await
    {
        if resp.status().is_success() {
            if let Ok(body) = resp.text().await {
                // Check if response is valid JSON (indicates response_format worked)
                supports_response_format = serde_json::from_str::<serde_json::Value>(&body).is_ok();
            }
        }
    }

    LlmCapabilities {
        available_models,
        supports_streaming,
        supports_response_format,
        default_model: llm.model.clone(),
    }
}

pub async fn test_connection(llm: &LlmConfig) -> ConnectionTestResult {
    let client = reqwest::Client::new();
    let request_body = serde_json::json!({
        "model": llm.model,
        "messages": [
            { "role": "user", "content": "hi" }
        ],
        "max_tokens": 5,
    });

    match client
        .post(format!(
            "{}/chat/completions",
            llm.base_url.trim_end_matches('/')
        ))
        .header("Authorization", format!("Bearer {}", llm.api_key))
        .header("Content-Type", "application/json")
        .json(&request_body)
        .send()
        .await
    {
        Ok(response) => {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            classify_connection_status(status, &body)
        }
        Err(error) => ConnectionTestResult {
            ok: false,
            kind: "network".to_string(),
            message: format!("Could not reach LLM endpoint: {}", error),
            status: None,
        },
    }
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
}
