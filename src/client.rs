use anyhow::{anyhow, Result};
use base64::{engine::general_purpose, Engine as _};
use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION};
use reqwest::{Client as HttpClient, Method};
use serde_json::Value;
use std::path::Path;

use crate::config::{AuthConfig, Config};
use crate::endpoints::Endpoint;

#[derive(Clone)]
pub struct RommClient {
    http: HttpClient,
    base_url: String,
    auth: Option<AuthConfig>,
}

impl RommClient {
    pub fn new(config: &Config) -> Result<Self> {
        let http = HttpClient::builder().build()?;
        Ok(Self {
            http,
            base_url: config.base_url.clone(),
            auth: config.auth.clone(),
        })
    }

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
                    let name =
                        reqwest::header::HeaderName::from_bytes(header.as_bytes()).map_err(|_| {
                            anyhow!("invalid API_KEY_HEADER, must be a valid HTTP header name")
                        })?;
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
        let output = serde_json::from_value(value).map_err(|e| {
            anyhow!(
                "failed to decode response for {} {}: {}",
                method,
                path,
                e
            )
        })?;

        Ok(output)
    }

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

        let method = Method::from_bytes(method.as_bytes())
            .map_err(|_| anyhow!("invalid HTTP method: {method}"))?;

        let mut req = self
            .http
            .request(method, &url)
            .headers(headers)
            .query(&query);

        if let Some(body) = body {
            req = req.json(&body);
        }

        let resp = req
            .send()
            .await
            .map_err(|e| anyhow!("request error: {e}"))?;

        let status = resp.status();
        if !status.is_success() {
            let body = resp.text().await.unwrap_or_default();
            return Err(anyhow!(
                "ROMM API error: {} {} - {}",
                status.as_u16(),
                status.canonical_reason().unwrap_or(""),
                body
            ));
        }

        let value = resp.json::<Value>().await?;
        Ok(value)
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
        let resp = self
            .http
            .get(&url)
            .headers(headers)
            .query(&[("rom_ids", rom_id.to_string()), ("filename", filename.to_string())])
            .send()
            .await
            .map_err(|e| anyhow!("download request error: {e}"))?;

        let status = resp.status();
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
        let bytes = resp
            .bytes()
            .await
            .map_err(|e| anyhow!("read body: {e}"))?;
        let received = bytes.len() as u64;
        on_progress(received, total);
        std::fs::write(save_path, &bytes).map_err(|e| anyhow!("write file {:?}: {e}", save_path))?;
        Ok(())
    }
}

