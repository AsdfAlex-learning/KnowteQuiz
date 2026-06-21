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
