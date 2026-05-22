use anyhow::{anyhow, Result};
use std::time::Instant;

use crate::config::normalize_romm_origin;

use super::response::{
    read_error_response_text, romm_api_error_truncated, version_from_heartbeat_json,
};
use super::RommClient;

/// Returns the browser-style origin for RomM (no `/api` suffix).
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
    /// RomM application version from `GET /api/heartbeat` (`SYSTEM.VERSION`), if the endpoint succeeds.
    pub async fn rom_server_version_from_heartbeat(&self) -> Option<String> {
        let v = self
            .request_json_unauthenticated("GET", "/api/heartbeat", &[], None)
            .await
            .ok()?;
        version_from_heartbeat_json(&v)
    }

    /// GET the OpenAPI spec from the server.
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
            let body = read_error_response_text(resp).await;
            return Err(romm_api_error_truncated(status, &body, 500));
        }

        resp.text()
            .await
            .map_err(|e| anyhow!("read OpenAPI body: {e}"))
    }
}
