//! HTTP client wrapper around the ROMM API.
//!
//! `RommClient` owns a configured `reqwest::Client` plus base URL and
//! authentication settings. Frontends (CLI, TUI, or a future GUI) depend
//! on this type instead of talking to `reqwest` directly.

use anyhow::{anyhow, Result};
use base64::{engine::general_purpose, Engine as _};
use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION};
use reqwest::{Client as HttpClient, Method};
use serde_json::Value;
use std::path::Path;
use std::time::Instant;
use tokio::io::AsyncWriteExt as _;

use crate::config::{AuthConfig, Config};
use crate::endpoints::Endpoint;

/// Map a successful HTTP response body to JSON [`Value`].
///
/// Empty or whitespace-only bodies become [`Value::Null`] (e.g. HTTP 204).
/// Non-JSON UTF-8 bodies are wrapped as `{"_non_json_body": "..."}`.
fn decode_json_response_body(bytes: &[u8]) -> Value {
    if bytes.is_empty() || bytes.iter().all(|b| b.is_ascii_whitespace()) {
        return Value::Null;
    }
    serde_json::from_slice(bytes).unwrap_or_else(|_| {
        serde_json::json!({
            "_non_json_body": String::from_utf8_lossy(bytes).to_string()
        })
    })
}

/// High-level HTTP client for the ROMM API.
///
/// This type hides the details of `reqwest` and authentication headers
/// behind a small, easy-to-mock interface that all frontends can share.
#[derive(Clone)]
pub struct RommClient {
    http: HttpClient,
    base_url: String,
    auth: Option<AuthConfig>,
    verbose: bool,
}

impl RommClient {
    /// Construct a new client from the high-level [`Config`].
    ///
    /// `verbose` enables stderr request logging (method, path, query key names, status, timing).
    /// This is typically done once in `main` and the resulting `RommClient` is shared
    /// (by reference or cloning) with the chosen frontend.
    pub fn new(config: &Config, verbose: bool) -> Result<Self> {
        let http = HttpClient::builder().build()?;
        Ok(Self {
            http,
            base_url: config.base_url.clone(),
            auth: config.auth.clone(),
            verbose,
        })
    }

    /// Build the HTTP headers for the current authentication mode.
    ///
    /// This helper centralises all auth logic so that the rest of the
    /// code never needs to worry about `Basic` vs `Bearer` vs API key.
    fn build_headers(&self) -> Result<HeaderMap> {
        let mut headers = HeaderMap::new();

        if let Some(auth) = &self.auth {
            match auth {
                AuthConfig::Basic { username, password } => {
                    let creds = format!("{username}:{password}");
                    let encoded = general_purpose::STANDARD.encode(creds.as_bytes());
                    let value = format!("Basic {encoded}");
                    headers.insert(
                        AUTHORIZATION,
                        HeaderValue::from_str(&value)
                            .map_err(|_| anyhow!("invalid basic auth header value"))?,
                    );
                }
                AuthConfig::Bearer { token } => {
                    let value = format!("Bearer {token}");
                    headers.insert(
                        AUTHORIZATION,
                        HeaderValue::from_str(&value)
                            .map_err(|_| anyhow!("invalid bearer auth header value"))?,
                    );
                }
                AuthConfig::ApiKey { header, key } => {
                    let name = reqwest::header::HeaderName::from_bytes(header.as_bytes()).map_err(
                        |_| anyhow!("invalid API_KEY_HEADER, must be a valid HTTP header name"),
                    )?;
                    headers.insert(
                        name,
                        HeaderValue::from_str(key)
                            .map_err(|_| anyhow!("invalid API_KEY header value"))?,
                    );
                }
            }
        }

        Ok(headers)
    }

    /// Call a typed endpoint using the low-level `request_json` primitive.
    pub async fn call<E>(&self, ep: &E) -> anyhow::Result<E::Output>
    where
        E: Endpoint,
        E::Output: serde::de::DeserializeOwned,
    {
        let method = ep.method();
        let path = ep.path();
        let query = ep.query();
        let body = ep.body();

        let value = self.request_json(method, &path, &query, body).await?;
        let output = serde_json::from_value(value)
            .map_err(|e| anyhow!("failed to decode response for {} {}: {}", method, path, e))?;

        Ok(output)
    }

    /// Low-level helper that issues an HTTP request and returns raw JSON.
    ///
    /// Higher-level helpers (such as typed `Endpoint` implementations)
    /// should prefer [`RommClient::call`] instead of using this directly.
    pub async fn request_json(
        &self,
        method: &str,
        path: &str,
        query: &[(String, String)],
        body: Option<Value>,
    ) -> Result<Value> {
        let url = format!(
            "{}/{}",
            self.base_url.trim_end_matches('/'),
            path.trim_start_matches('/')
        );
        let headers = self.build_headers()?;

        let http_method = Method::from_bytes(method.as_bytes())
            .map_err(|_| anyhow!("invalid HTTP method: {method}"))?;

        // Ensure query params serialize as key=value pairs (reqwest/serde_urlencoded
        // expect sequences of (key, value); using &[(&str, &str)] guarantees correct encoding).
        let query_refs: Vec<(&str, &str)> = query
            .iter()
            .map(|(k, v)| (k.as_str(), v.as_str()))
            .collect();

        let mut req = self
            .http
            .request(http_method, &url)
            .headers(headers)
            .query(&query_refs);

        if let Some(body) = body {
            req = req.json(&body);
        }

        let t0 = Instant::now();
        let resp = req
            .send()
            .await
            .map_err(|e| anyhow!("request error: {e}"))?;

        let status = resp.status();
        if self.verbose {
            let keys: Vec<&str> = query.iter().map(|(k, _)| k.as_str()).collect();
            tracing::info!(
                "[romm-cli] {} {} query_keys={:?} -> {} ({}ms)",
                method,
                path,
                keys,
                status.as_u16(),
                t0.elapsed().as_millis()
            );
        }
        if !status.is_success() {
            let body = resp.text().await.unwrap_or_default();
            return Err(anyhow!(
                "ROMM API error: {} {} - {}",
                status.as_u16(),
                status.canonical_reason().unwrap_or(""),
                body
            ));
        }

        let bytes = resp
            .bytes()
            .await
            .map_err(|e| anyhow!("read response body: {e}"))?;

        Ok(decode_json_response_body(&bytes))
    }

    /// Download ROM(s) as a zip file to `save_path`, calling `on_progress(received, total)`.
    /// Uses GET /api/roms/download?rom_ids={id}&filename=... per RomM OpenAPI.
    pub async fn download_rom<F>(
        &self,
        rom_id: u64,
        save_path: &Path,
        mut on_progress: F,
    ) -> Result<()>
    where
        F: FnMut(u64, u64) + Send,
    {
        let path = "/api/roms/download";
        let url = format!(
            "{}/{}",
            self.base_url.trim_end_matches('/'),
            path.trim_start_matches('/')
        );
        let headers = self.build_headers()?;

        let filename = save_path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("download.zip");
        let t0 = Instant::now();
        let mut resp = self
            .http
            .get(&url)
            .headers(headers)
            .query(&[
                ("rom_ids", rom_id.to_string()),
                ("filename", filename.to_string()),
            ])
            .send()
            .await
            .map_err(|e| anyhow!("download request error: {e}"))?;

        let status = resp.status();
        if self.verbose {
            tracing::info!(
                "[romm-cli] GET /api/roms/download rom_id={} filename={:?} -> {} ({}ms)",
                rom_id,
                filename,
                status.as_u16(),
                t0.elapsed().as_millis()
            );
        }
        if !status.is_success() {
            let body = resp.text().await.unwrap_or_default();
            return Err(anyhow!(
                "ROMM API error: {} {} - {}",
                status.as_u16(),
                status.canonical_reason().unwrap_or(""),
                body
            ));
        }

        let total = resp.content_length().unwrap_or(0);

        let mut file = tokio::fs::File::create(save_path)
            .await
            .map_err(|e| anyhow!("create file {:?}: {e}", save_path))?;
        let mut received: u64 = 0;

        while let Some(chunk) = resp.chunk().await.map_err(|e| anyhow!("read chunk: {e}"))? {
            file.write_all(&chunk)
                .await
                .map_err(|e| anyhow!("write chunk {:?}: {e}", save_path))?;
            received += chunk.len() as u64;
            on_progress(received, total);
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decode_json_empty_and_whitespace_to_null() {
        assert_eq!(decode_json_response_body(b""), Value::Null);
        assert_eq!(decode_json_response_body(b"  \n\t "), Value::Null);
    }

    #[test]
    fn decode_json_object_roundtrip() {
        let v = decode_json_response_body(br#"{"a":1}"#);
        assert_eq!(v["a"], 1);
    }

    #[test]
    fn decode_non_json_wrapped() {
        let v = decode_json_response_body(b"plain text");
        assert_eq!(v["_non_json_body"], "plain text");
    }
}
