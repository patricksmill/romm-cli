//! HTTP client wrapper around the ROMM API.
//!
//! `RommClient` owns a configured `reqwest::Client` plus base URL and
//! authentication settings. Frontends (CLI, TUI, or a future GUI) depend
//! on this type instead of talking to `reqwest` directly.

use anyhow::{anyhow, Result};
use base64::{engine::general_purpose, Engine as _};
use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION};
use reqwest::multipart;
use reqwest::{Client as HttpClient, Method};
use serde_json::Value;
use std::path::Path;
use std::time::Instant;
use tokio::io::AsyncWriteExt as _;

use crate::config::{normalize_romm_origin, AuthConfig, Config};
use crate::core::interrupt::cancelled_error;
use crate::endpoints::Endpoint;

/// Default `User-Agent` for every request. The stock `reqwest` UA is sometimes blocked at the HTTP
/// layer (403, etc.) by reverse proxies; override with env `ROMM_USER_AGENT` if needed.
fn http_user_agent() -> String {
    match std::env::var("ROMM_USER_AGENT") {
        Ok(s) if !s.trim().is_empty() => s,
        _ => format!(
            "Mozilla/5.0 (compatible; romm-cli/{}; +https://github.com/patricksmill/romm-cli)",
            env!("CARGO_PKG_VERSION")
        ),
    }
}

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

fn version_from_heartbeat_json(v: &Value) -> Option<String> {
    v.get("SYSTEM")?.get("VERSION")?.as_str().map(String::from)
}

/// High-level HTTP client for the ROMM API.
///
/// This type hides the details of `reqwest` and authentication headers
/// behind a small interface that all frontends can share.
///
/// # Examples
///
/// ```no_run
/// # use romm_cli::config::Config;
/// # use romm_cli::client::RommClient;
/// # async fn example() -> anyhow::Result<()> {
/// let config = Config {
///     base_url: "https://romm.example.com".to_string(),
///     download_dir: "./downloads".to_string(),
///     use_https: true,
///     auth: None,
/// };
/// let client = RommClient::new(&config, false)?;
/// # Ok(())
/// # }
/// ```
#[derive(Clone)]
pub struct RommClient {
    /// The underlying HTTP client.
    http: HttpClient,
    /// The base URL of the RomM server.
    base_url: String,
    /// Current authentication configuration.
    auth: Option<AuthConfig>,
    /// Whether to log request details to stderr.
    verbose: bool,
}

/// Returns the browser-style origin for RomM (no `/api` suffix).
///
/// Same as [`crate::config::normalize_romm_origin`].
pub fn api_root_url(base_url: &str) -> String {
    normalize_romm_origin(base_url)
}

fn alternate_http_scheme_root(root: &str) -> Option<String> {
    root.strip_prefix("http://")
        .map(|rest| format!("https://{}", rest))
        .or_else(|| {
            root.strip_prefix("https://")
                .map(|rest| format!("http://{}", rest))
        })
}

/// Resolves the origin used to fetch `/openapi.json`.
///
/// Normally equals [`normalize_romm_origin`] applied to `api_base_url`,
/// but can be overridden by the `ROMM_OPENAPI_BASE_URL` environment variable.
pub fn resolve_openapi_root(api_base_url: &str) -> String {
    if let Ok(s) = std::env::var("ROMM_OPENAPI_BASE_URL") {
        let t = s.trim();
        if !t.is_empty() {
            return normalize_romm_origin(t);
        }
    }
    normalize_romm_origin(api_base_url)
}

/// Returns a list of candidate URLs to try for the OpenAPI JSON document.
///
/// This includes both HTTP/HTTPS schemes and common paths like `/openapi.json`
/// and `/api/openapi.json`.
pub fn openapi_spec_urls(api_root: &str) -> Vec<String> {
    let root = api_root.trim_end_matches('/').to_string();
    let mut roots = vec![root.clone()];
    if let Some(alt) = alternate_http_scheme_root(&root) {
        if alt != root {
            roots.push(alt);
        }
    }

    let mut urls = Vec::new();
    for r in roots {
        let b = r.trim_end_matches('/');
        urls.push(format!("{b}/openapi.json"));
        urls.push(format!("{b}/api/openapi.json"));
    }
    urls
}

impl RommClient {
    /// Construct a new client from the high-level [`Config`].
    ///
    /// `verbose` enables stderr request logging (method, path, query key names, status, timing).
    /// This is typically done once in `main` and the resulting `RommClient` is shared
    /// (by reference or cloning) with the chosen frontend.
    pub fn new(config: &Config, verbose: bool) -> Result<Self> {
        let http = HttpClient::builder()
            .user_agent(http_user_agent())
            .build()?;
        Ok(Self {
            http,
            base_url: config.base_url.clone(),
            auth: config.auth.clone(),
            verbose,
        })
    }

    /// Returns true if verbose logging is enabled.
    pub fn verbose(&self) -> bool {
        self.verbose
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

    /// Executes a typed [`Endpoint`] and returns its deserialized output.
    ///
    /// This is the preferred way to interact with the API when a typed
    /// endpoint definition exists.
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

    /// Low-level helper that issues an HTTP request and returns a raw JSON [`Value`].
    ///
    /// Higher-level code should generally prefer [`RommClient::call`].
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

    pub async fn request_json_unauthenticated(
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
        let headers = HeaderMap::new();

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

    /// RomM application version from `GET /api/heartbeat` (`SYSTEM.VERSION`), if the endpoint succeeds.
    pub async fn rom_server_version_from_heartbeat(&self) -> Option<String> {
        let v = self
            .request_json_unauthenticated("GET", "/api/heartbeat", &[], None)
            .await
            .ok()?;
        version_from_heartbeat_json(&v)
    }

    /// GET the OpenAPI spec from the server. Tries [`openapi_spec_urls`] in order (HTTP/HTTPS and
    /// `/openapi.json` vs `/api/openapi.json`). Uses [`resolve_openapi_root`] for the origin.
    pub async fn fetch_openapi_json(&self) -> Result<String> {
        let root = resolve_openapi_root(&self.base_url);
        let urls = openapi_spec_urls(&root);
        let mut failures = Vec::new();
        for url in &urls {
            match self.fetch_openapi_json_once(url).await {
                Ok(body) => return Ok(body),
                Err(e) => failures.push(format!("{url}: {e:#}")),
            }
        }
        Err(anyhow!(
            "could not download OpenAPI ({} attempt(s)): {}",
            failures.len(),
            failures.join(" | ")
        ))
    }

    async fn fetch_openapi_json_once(&self, url: &str) -> Result<String> {
        let headers = self.build_headers()?;

        let t0 = Instant::now();
        let resp = self
            .http
            .get(url)
            .headers(headers)
            .send()
            .await
            .map_err(|e| anyhow!("request failed: {e}"))?;

        let status = resp.status();
        if self.verbose {
            tracing::info!(
                "[romm-cli] GET {} -> {} ({}ms)",
                url,
                status.as_u16(),
                t0.elapsed().as_millis()
            );
        }
        if !status.is_success() {
            let body = resp.text().await.unwrap_or_default();
            return Err(anyhow!(
                "HTTP {} {} - {}",
                status.as_u16(),
                status.canonical_reason().unwrap_or(""),
                body.chars().take(500).collect::<String>()
            ));
        }

        resp.text()
            .await
            .map_err(|e| anyhow!("read OpenAPI body: {e}"))
    }

    /// Downloads a ROM (or multiple ROMs as a zip) to the specified path.
    ///
    /// This method supports resuming interrupted downloads by checking if the file
    /// already exists and sending an HTTP `Range` header.
    ///
    /// # Progress
    ///
    /// The `on_progress` callback is called with `(received_bytes, total_bytes)`.
    pub async fn download_rom<F>(
        &self,
        rom_id: u64,
        save_path: &Path,
        mut on_progress: F,
    ) -> Result<()>
    where
        F: FnMut(u64, u64) + Send,
    {
        self.download_rom_with_cancel(rom_id, save_path, |_, _| false, &mut on_progress)
            .await
    }

    pub async fn download_rom_with_cancel<F, C>(
        &self,
        rom_id: u64,
        save_path: &Path,
        mut is_cancelled: C,
        on_progress: &mut F,
    ) -> Result<()>
    where
        F: FnMut(u64, u64) + Send,
        C: FnMut(u64, u64) -> bool + Send,
    {
        let path = "/api/roms/download";
        let url = format!(
            "{}/{}",
            self.base_url.trim_end_matches('/'),
            path.trim_start_matches('/')
        );
        let mut headers = self.build_headers()?;

        let filename = save_path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("download.zip");

        // Check for an existing partial file to resume from.
        let existing_len = tokio::fs::metadata(save_path)
            .await
            .map(|m| m.len())
            .unwrap_or(0);

        if existing_len > 0 {
            let range = format!("bytes={existing_len}-");
            if let Ok(v) = reqwest::header::HeaderValue::from_str(&range) {
                headers.insert(reqwest::header::RANGE, v);
            }
        }

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

        // Determine whether the server honoured our Range header.
        let (mut received, total, mut file) = if status == reqwest::StatusCode::PARTIAL_CONTENT {
            // 206 — resume: content_length is the *remaining* bytes.
            let remaining = resp.content_length().unwrap_or(0);
            let total = existing_len + remaining;
            let file = tokio::fs::OpenOptions::new()
                .append(true)
                .open(save_path)
                .await
                .map_err(|e| anyhow!("open file for append {:?}: {e}", save_path))?;
            (existing_len, total, file)
        } else {
            // 200 — server doesn't support ranges; start from scratch.
            let total = resp.content_length().unwrap_or(0);
            let file = tokio::fs::File::create(save_path)
                .await
                .map_err(|e| anyhow!("create file {:?}: {e}", save_path))?;
            (0u64, total, file)
        };

        if is_cancelled(received, total) {
            return Err(cancelled_error());
        }

        while let Some(chunk) = resp.chunk().await.map_err(|e| anyhow!("read chunk: {e}"))? {
            if is_cancelled(received, total) {
                return Err(cancelled_error());
            }
            file.write_all(&chunk)
                .await
                .map_err(|e| anyhow!("write chunk {:?}: {e}", save_path))?;
            received += chunk.len() as u64;
            on_progress(received, total);
        }

        Ok(())
    }

    /// Uploads a ROM file to the server using the RomM chunked upload API.
    ///
    /// This method splits the file into chunks (default 2MB) and uploads them
    /// sequentially, providing progress updates via the `on_progress` callback.
    pub async fn upload_rom<F>(
        &self,
        platform_id: u64,
        file_path: &Path,
        mut on_progress: F,
    ) -> Result<()>
    where
        F: FnMut(u64, u64) + Send,
    {
        let filename = file_path
            .file_name()
            .and_then(|n| n.to_str())
            .ok_or_else(|| anyhow!("Invalid filename for upload"))?;

        let metadata = tokio::fs::metadata(file_path)
            .await
            .map_err(|e| anyhow!("Failed to read file metadata {:?}: {}", file_path, e))?;
        let total_size = metadata.len();

        // 2MB chunk size
        let chunk_size: u64 = 2 * 1024 * 1024;
        // Use integer division ceiling
        let total_chunks = if total_size == 0 {
            1
        } else {
            total_size.div_ceil(chunk_size)
        };

        let mut start_headers = self.build_headers()?;
        start_headers.insert(
            reqwest::header::HeaderName::from_static("x-upload-platform"),
            reqwest::header::HeaderValue::from_str(&platform_id.to_string())?,
        );
        start_headers.insert(
            reqwest::header::HeaderName::from_static("x-upload-filename"),
            reqwest::header::HeaderValue::from_str(filename)?,
        );
        start_headers.insert(
            reqwest::header::HeaderName::from_static("x-upload-total-size"),
            reqwest::header::HeaderValue::from_str(&total_size.to_string())?,
        );
        start_headers.insert(
            reqwest::header::HeaderName::from_static("x-upload-total-chunks"),
            reqwest::header::HeaderValue::from_str(&total_chunks.to_string())?,
        );

        let start_url = format!(
            "{}/api/roms/upload/start",
            self.base_url.trim_end_matches('/')
        );

        let t0 = Instant::now();
        let resp = self
            .http
            .post(&start_url)
            .headers(start_headers)
            .send()
            .await
            .map_err(|e| anyhow!("upload start request error: {}", e))?;

        let status = resp.status();
        if self.verbose {
            tracing::info!(
                "[romm-cli] POST /api/roms/upload/start -> {} ({}ms)",
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

        let start_resp: Value = resp
            .json()
            .await
            .map_err(|e| anyhow!("failed to parse start upload response: {}", e))?;
        let upload_id = start_resp
            .get("upload_id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow!("Missing upload_id in start response: {}", start_resp))?
            .to_string();

        use tokio::io::AsyncReadExt;
        let mut file = tokio::fs::File::open(file_path).await?;
        let mut uploaded_bytes = 0;
        let mut buffer = vec![0u8; chunk_size as usize];

        for chunk_index in 0..total_chunks {
            let mut chunk_bytes = 0;
            let mut chunk_data = Vec::new();

            while chunk_bytes < chunk_size as usize {
                let n = file.read(&mut buffer[..]).await?;
                if n == 0 {
                    break;
                }
                chunk_data.extend_from_slice(&buffer[..n]);
                chunk_bytes += n;
            }

            let mut chunk_headers = self.build_headers()?;
            chunk_headers.insert(
                reqwest::header::HeaderName::from_static("x-chunk-index"),
                reqwest::header::HeaderValue::from_str(&chunk_index.to_string())?,
            );

            let chunk_url = format!(
                "{}/api/roms/upload/{}",
                self.base_url.trim_end_matches('/'),
                upload_id
            );

            let _t_chunk = Instant::now();
            let chunk_resp = self
                .http
                .put(&chunk_url)
                .headers(chunk_headers)
                .body(chunk_data.clone())
                .send()
                .await
                .map_err(|e| anyhow!("chunk upload request error: {}", e))?;

            if !chunk_resp.status().is_success() {
                let body = chunk_resp.text().await.unwrap_or_default();
                // Attempt to cancel
                let cancel_url = format!(
                    "{}/api/roms/upload/{}/cancel",
                    self.base_url.trim_end_matches('/'),
                    upload_id
                );
                let _ = self
                    .http
                    .post(&cancel_url)
                    .headers(self.build_headers()?)
                    .send()
                    .await;

                return Err(anyhow!("Failed to upload chunk {}: {}", chunk_index, body));
            }

            uploaded_bytes += chunk_data.len() as u64;
            on_progress(uploaded_bytes, total_size);
        }

        let complete_url = format!(
            "{}/api/roms/upload/{}/complete",
            self.base_url.trim_end_matches('/'),
            upload_id
        );
        let complete_resp = self
            .http
            .post(&complete_url)
            .headers(self.build_headers()?)
            .send()
            .await
            .map_err(|e| anyhow!("upload complete request error: {}", e))?;

        if !complete_resp.status().is_success() {
            let body = complete_resp.text().await.unwrap_or_default();
            return Err(anyhow!("Failed to complete upload: {}", body));
        }

        Ok(())
    }

    /// Triggers a server-side task by name (e.g., `"scan_library"`).
    ///
    /// # Arguments
    ///
    /// * `task_name` - The internal name of the task to run.
    /// * `kwargs` - Optional JSON arguments to pass to the task.
    pub async fn run_task(&self, task_name: &str, kwargs: Option<Value>) -> Result<Value> {
        let path = format!("/api/tasks/run/{}", task_name);
        self.request_json("POST", &path, &[], kwargs).await
    }

    /// Polls the status of a running task by its ID.
    pub async fn get_task_status(&self, task_id: &str) -> Result<Value> {
        let path = format!("/api/tasks/{}", task_id);
        self.request_json("GET", &path, &[], None).await
    }

    /// Enqueues all runnable tasks on the server.
    pub async fn run_all_tasks(&self) -> Result<Value> {
        self.request_json("POST", "/api/tasks/run", &[], None).await
    }

    /// Lists all recent and active tasks.
    pub async fn list_tasks(&self) -> Result<Value> {
        self.request_json("GET", "/api/tasks", &[], None).await
    }

    /// Returns the current status of the task queue (active, queued, completed).
    pub async fn get_tasks_queue_status(&self) -> Result<Value> {
        self.request_json("GET", "/api/tasks/status", &[], None)
            .await
    }

    /// Uploads a game save file to the server.
    ///
    /// # Arguments
    ///
    /// * `rom_id` - ID of the ROM this save belongs to.
    /// * `emulator` - Optional name of the emulator that generated the save.
    /// * `file_path` - Local path to the save file.
    pub async fn upload_save_file(
        &self,
        rom_id: u64,
        emulator: Option<&str>,
        file_path: &Path,
    ) -> Result<Value> {
        let url = format!("{}/api/saves", self.base_url.trim_end_matches('/'));
        let bytes = tokio::fs::read(file_path)
            .await
            .map_err(|e| anyhow!("read {}: {e}", file_path.display()))?;
        let fname = file_path
            .file_name()
            .and_then(|n| n.to_str())
            .ok_or_else(|| anyhow!("upload path must have a unicode filename"))?;
        let part = multipart::Part::bytes(bytes).file_name(fname.to_string());
        let form = multipart::Form::new().part("saveFile", part);
        let mut query: Vec<(String, String)> = vec![("rom_id".into(), rom_id.to_string())];
        if let Some(em) = emulator {
            if !em.is_empty() {
                query.push(("emulator".into(), em.to_string()));
            }
        }
        let query_refs: Vec<(&str, &str)> = query
            .iter()
            .map(|(k, v)| (k.as_str(), v.as_str()))
            .collect();
        let headers = self.build_headers()?;
        let t0 = Instant::now();
        let resp = self
            .http
            .post(&url)
            .headers(headers)
            .query(&query_refs)
            .multipart(form)
            .send()
            .await
            .map_err(|e| anyhow!("save upload request: {e}"))?;
        let status = resp.status();
        if self.verbose {
            tracing::info!(
                "[romm-cli] POST /api/saves rom_id={rom_id} -> {} ({}ms)",
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
            .map_err(|e| anyhow!("read save upload body: {e}"))?;
        Ok(decode_json_response_body(&bytes))
    }

    /// `POST /api/states` with multipart field `stateFile`.
    pub async fn upload_state_file(
        &self,
        rom_id: u64,
        emulator: Option<&str>,
        file_path: &Path,
    ) -> Result<Value> {
        let url = format!("{}/api/states", self.base_url.trim_end_matches('/'));
        let bytes = tokio::fs::read(file_path)
            .await
            .map_err(|e| anyhow!("read {}: {e}", file_path.display()))?;
        let fname = file_path
            .file_name()
            .and_then(|n| n.to_str())
            .ok_or_else(|| anyhow!("upload path must have a unicode filename"))?;
        let part = multipart::Part::bytes(bytes).file_name(fname.to_string());
        let form = multipart::Form::new().part("stateFile", part);
        let mut query: Vec<(String, String)> = vec![("rom_id".into(), rom_id.to_string())];
        if let Some(em) = emulator {
            if !em.is_empty() {
                query.push(("emulator".into(), em.to_string()));
            }
        }
        let query_refs: Vec<(&str, &str)> = query
            .iter()
            .map(|(k, v)| (k.as_str(), v.as_str()))
            .collect();
        let headers = self.build_headers()?;
        let resp = self
            .http
            .post(&url)
            .headers(headers)
            .query(&query_refs)
            .multipart(form)
            .send()
            .await
            .map_err(|e| anyhow!("state upload request: {e}"))?;
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
        let bytes = resp
            .bytes()
            .await
            .map_err(|e| anyhow!("read state upload body: {e}"))?;
        Ok(decode_json_response_body(&bytes))
    }

    /// `POST /api/screenshots` with multipart field `screenshotFile`.
    pub async fn upload_screenshot_file(&self, rom_id: u64, file_path: &Path) -> Result<Value> {
        let url = format!("{}/api/screenshots", self.base_url.trim_end_matches('/'));
        let bytes = tokio::fs::read(file_path)
            .await
            .map_err(|e| anyhow!("read {}: {e}", file_path.display()))?;
        let fname = file_path
            .file_name()
            .and_then(|n| n.to_str())
            .ok_or_else(|| anyhow!("upload path must have a unicode filename"))?;
        let part = multipart::Part::bytes(bytes).file_name(fname.to_string());
        let form = multipart::Form::new().part("screenshotFile", part);
        let headers = self.build_headers()?;
        let resp = self
            .http
            .post(&url)
            .headers(headers)
            .query(&[("rom_id", rom_id.to_string().as_str())])
            .multipart(form)
            .send()
            .await
            .map_err(|e| anyhow!("screenshot upload: {e}"))?;
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
        let bytes = resp
            .bytes()
            .await
            .map_err(|e| anyhow!("read screenshot body: {e}"))?;
        Ok(decode_json_response_body(&bytes))
    }

    /// `POST /api/firmware?platform_id=` with multipart `files` (single file supported).
    pub async fn upload_firmware_file(&self, platform_id: u64, file_path: &Path) -> Result<Value> {
        let url = format!("{}/api/firmware", self.base_url.trim_end_matches('/'));
        let bytes = tokio::fs::read(file_path)
            .await
            .map_err(|e| anyhow!("read {}: {e}", file_path.display()))?;
        let fname = file_path
            .file_name()
            .and_then(|n| n.to_str())
            .ok_or_else(|| anyhow!("upload path must have a unicode filename"))?;
        let part = multipart::Part::bytes(bytes).file_name(fname.to_string());
        let form = multipart::Form::new().part("files", part);
        let headers = self.build_headers()?;
        let resp = self
            .http
            .post(&url)
            .headers(headers)
            .query(&[("platform_id", platform_id.to_string())])
            .multipart(form)
            .send()
            .await
            .map_err(|e| anyhow!("firmware upload: {e}"))?;
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
        let bytes = resp
            .bytes()
            .await
            .map_err(|e| anyhow!("read firmware body: {e}"))?;
        Ok(decode_json_response_body(&bytes))
    }

    /// Authenticated GET returning raw bytes (e.g. save/state/firmware file or gamelist export).
    pub async fn get_bytes(&self, path: &str, query: &[(String, String)]) -> Result<Vec<u8>> {
        let url = format!(
            "{}/{}",
            self.base_url.trim_end_matches('/'),
            path.trim_start_matches('/')
        );
        let headers = self.build_headers()?;
        let query_refs: Vec<(&str, &str)> = query
            .iter()
            .map(|(k, v)| (k.as_str(), v.as_str()))
            .collect();
        let resp = self
            .http
            .get(&url)
            .headers(headers)
            .query(&query_refs)
            .send()
            .await
            .map_err(|e| anyhow!("GET {path}: {e}"))?;
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
        Ok(resp.bytes().await?.to_vec())
    }

    /// POST returning raw bytes (e.g. gamelist XML).
    pub async fn post_bytes(
        &self,
        path: &str,
        query: &[(String, String)],
        json_body: Option<Value>,
    ) -> Result<Vec<u8>> {
        let url = format!(
            "{}/{}",
            self.base_url.trim_end_matches('/'),
            path.trim_start_matches('/')
        );
        let headers = self.build_headers()?;
        let query_refs: Vec<(&str, &str)> = query
            .iter()
            .map(|(k, v)| (k.as_str(), v.as_str()))
            .collect();
        let mut req = self.http.post(&url).headers(headers).query(&query_refs);
        if let Some(b) = json_body {
            req = req.json(&b);
        }
        let resp = req.send().await.map_err(|e| anyhow!("POST {path}: {e}"))?;
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
        Ok(resp.bytes().await?.to_vec())
    }

    /// `POST /api/roms/{id}/manuals` — raw file body with `x-upload-filename` header.
    pub async fn upload_rom_manual(&self, rom_id: u64, file_path: &Path) -> Result<Value> {
        let fname = file_path
            .file_name()
            .and_then(|n| n.to_str())
            .ok_or_else(|| anyhow!("manual path must have a unicode filename"))?
            .to_string();
        let url = format!(
            "{}/api/roms/{}/manuals",
            self.base_url.trim_end_matches('/'),
            rom_id
        );
        let bytes = tokio::fs::read(file_path)
            .await
            .map_err(|e| anyhow!("read {}: {e}", file_path.display()))?;
        let mut headers = self.build_headers()?;
        headers.insert(
            reqwest::header::HeaderName::from_static("x-upload-filename"),
            HeaderValue::from_str(&fname).map_err(|_| anyhow!("invalid x-upload-filename"))?,
        );
        let resp = self
            .http
            .post(&url)
            .headers(headers)
            .body(bytes)
            .send()
            .await
            .map_err(|e| anyhow!("manual upload: {e}"))?;
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
        let out = resp.bytes().await?;
        Ok(decode_json_response_body(&out))
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

    #[test]
    fn api_root_url_strips_trailing_api() {
        assert_eq!(
            super::api_root_url("http://localhost:8080/api"),
            "http://localhost:8080"
        );
        assert_eq!(
            super::api_root_url("http://localhost:8080/api/"),
            "http://localhost:8080"
        );
        assert_eq!(
            super::api_root_url("http://localhost:8080"),
            "http://localhost:8080"
        );
    }

    #[test]
    fn openapi_spec_urls_try_primary_scheme_then_alt() {
        let urls = super::openapi_spec_urls("http://example.test");
        assert_eq!(urls[0], "http://example.test/openapi.json");
        assert_eq!(urls[1], "http://example.test/api/openapi.json");
        assert!(
            urls.iter()
                .any(|u| u == "https://example.test/openapi.json"),
            "{urls:?}"
        );
    }
}
