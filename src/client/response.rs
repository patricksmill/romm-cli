use anyhow::anyhow;
use reqwest::Response;
use reqwest::StatusCode;
use serde_json::Value;

/// Map a successful HTTP response body to JSON [`Value`].
///
/// Empty or whitespace-only bodies become [`Value::Null`] (e.g. HTTP 204).
/// Non-JSON UTF-8 bodies are wrapped as `{"_non_json_body": "..."}`.
pub(crate) fn decode_json_response_body(bytes: &[u8]) -> Value {
    if bytes.is_empty() || bytes.iter().all(|b| b.is_ascii_whitespace()) {
        return Value::Null;
    }
    serde_json::from_slice(bytes).unwrap_or_else(|_| {
        serde_json::json!({
            "_non_json_body": String::from_utf8_lossy(bytes).to_string()
        })
    })
}

/// Read an error response body without panicking when the stream fails.
pub(crate) async fn read_error_response_text(resp: Response) -> String {
    match resp.text().await {
        Ok(body) => body,
        Err(err) => format!("<failed to read error body: {err}>"),
    }
}

pub(crate) fn romm_api_error(status: StatusCode, body: &str) -> anyhow::Error {
    anyhow!(
        "ROMM API error: {} {} - {}",
        status.as_u16(),
        status.canonical_reason().unwrap_or(""),
        body
    )
}

pub(crate) fn romm_api_error_truncated(
    status: StatusCode,
    body: &str,
    max_chars: usize,
) -> anyhow::Error {
    anyhow!(
        "ROMM API error: {} {} - {}",
        status.as_u16(),
        status.canonical_reason().unwrap_or(""),
        body.chars().take(max_chars).collect::<String>()
    )
}

pub(crate) fn version_from_heartbeat_json(v: &Value) -> Option<String> {
    v.get("SYSTEM")?.get("VERSION")?.as_str().map(String::from)
}
