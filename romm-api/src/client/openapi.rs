use std::time::Instant;

use crate::config::normalize_romm_origin;
use crate::error::ApiError;

use super::response::{
    api_error_from_response_truncated, read_error_response_text, version_from_heartbeat_json,
};
use super::RommClient;

/// Returns the browser-style origin for RomM (no `/api` suffix).
pub fn api_root_url(base_url: &str) -> String {
    normalize_romm_origin(base_url)
}

fn alternate_http_scheme_root(root: &str) -> Option<String> {
    root.strip_prefix("http://")
        .map(|rest| format!("https://{}", rest))
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
    pub async fn fetch_openapi_json(&self) -> Result<String, ApiError> {
        let root = resolve_openapi_root(&self.base_url);
        let urls = openapi_spec_urls(&root);
        let mut failures = Vec::new();
        for url in &urls {
            match self.fetch_openapi_json_once(url).await {
                Ok(body) => return Ok(body),
                Err(e) => failures.push(format!(
                    "{}: {}",
                    crate::log_redact::redact_url_for_log(url),
                    e.redacted_for_log()
                )),
            }
        }
        Err(ApiError::UnexpectedResponse(format!(
            "could not download OpenAPI ({} attempt(s)): {}",
            failures.len(),
            failures.join(" | ")
        )))
    }

    async fn fetch_openapi_json_once(&self, url: &str) -> Result<String, ApiError> {
        let headers = self.build_headers()?;

        let t0 = Instant::now();
        let resp = self.http.get(url).headers(headers).send().await?;

        let status = resp.status();
        if self.verbose {
            tracing::info!(
                "[romm-cli] GET {} -> {} ({}ms)",
                crate::log_redact::redact_url_for_log(url),
                status.as_u16(),
                t0.elapsed().as_millis()
            );
        }
        if !status.is_success() {
            let body = read_error_response_text(resp).await;
            return Err(api_error_from_response_truncated(status, &body, 500));
        }

        let body = resp.text().await.map_err(ApiError::from)?;
        validate_openapi_json_body(&body)?;
        Ok(body)
    }
}

/// Reject empty/HTML/non-OpenAPI 200 bodies so callers can try the next URL or fall back.
fn validate_openapi_json_body(body: &str) -> Result<(), ApiError> {
    let trimmed = body.trim();
    if trimmed.is_empty() {
        return Err(ApiError::UnexpectedResponse(
            "OpenAPI response body is empty".into(),
        ));
    }
    let value: serde_json::Value = serde_json::from_str(trimmed)
        .map_err(|e| ApiError::UnexpectedResponse(format!("OpenAPI response is not JSON: {e}")))?;
    if value.get("paths").and_then(|p| p.as_object()).is_none() {
        return Err(ApiError::UnexpectedResponse(
            "OpenAPI JSON missing 'paths' object".into(),
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::config::{AuthConfig, Config, ExtrasDefaults};
    use wiremock::matchers::{method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    use super::*;

    fn client_for(base_url: &str) -> RommClient {
        RommClient::new(
            &Config {
                base_url: base_url.to_string(),
                download_dir: ".".to_string(),
                use_https: false,
                auth: Some(AuthConfig::Bearer {
                    token: "secret".to_string(),
                }),
                extras_defaults: ExtrasDefaults::default(),
                save_sync: Default::default(),
                roms_layout: Default::default(),
                theme: crate::config::default_theme_id(),
                tui_layout: Default::default(),
            },
            false,
        )
        .expect("client")
    }

    #[tokio::test]
    async fn fetch_openapi_json_rejects_empty_200_body() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/openapi.json"))
            .respond_with(ResponseTemplate::new(200).set_body_string(""))
            .mount(&server)
            .await;
        Mock::given(method("GET"))
            .and(path("/api/openapi.json"))
            .respond_with(ResponseTemplate::new(200).set_body_string(""))
            .mount(&server)
            .await;

        let err = client_for(&server.uri())
            .fetch_openapi_json()
            .await
            .expect_err("empty OpenAPI body must not succeed");
        let msg = err.to_string();
        assert!(
            msg.contains("OpenAPI") || msg.contains("empty") || msg.contains("JSON"),
            "{msg}"
        );
    }

    #[tokio::test]
    async fn fetch_openapi_json_skips_empty_and_uses_next_url() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/openapi.json"))
            .respond_with(ResponseTemplate::new(200).set_body_string(""))
            .mount(&server)
            .await;
        let body = r#"{"openapi":"3.0.0","info":{"version":"1.0.0"},"paths":{}}"#;
        Mock::given(method("GET"))
            .and(path("/api/openapi.json"))
            .respond_with(ResponseTemplate::new(200).set_body_string(body))
            .mount(&server)
            .await;

        let got = client_for(&server.uri())
            .fetch_openapi_json()
            .await
            .expect("second URL should supply valid OpenAPI JSON");
        assert!(got.contains("\"paths\""));
    }

    #[tokio::test]
    async fn fetch_openapi_json_failure_does_not_echo_auth_body() {
        let server = MockServer::start().await;
        let echoed_secret = "proxy echoed Authorization: Bearer secret";
        Mock::given(method("GET"))
            .and(path("/openapi.json"))
            .respond_with(ResponseTemplate::new(401).set_body_string(echoed_secret))
            .mount(&server)
            .await;
        Mock::given(method("GET"))
            .and(path("/api/openapi.json"))
            .respond_with(ResponseTemplate::new(401).set_body_string(echoed_secret))
            .mount(&server)
            .await;

        let err = client_for(&server.uri())
            .fetch_openapi_json()
            .await
            .expect_err("failed OpenAPI downloads should report sanitized attempts");
        let msg = err.to_string();
        assert!(!msg.contains("Bearer secret"), "{msg}");
        assert!(!err.redacted_for_log().contains("Bearer secret"));
        assert!(msg.contains("401"));
    }

    #[tokio::test]
    async fn fetch_openapi_json_request_error_redacts_url_secrets() {
        let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        let port = listener.local_addr().unwrap().port();
        drop(listener);
        let base_url = format!(
            "http://user:url-secret@127.0.0.1:{port}?token=query-secret"
        );

        let err = client_for(&base_url)
            .fetch_openapi_json()
            .await
            .expect_err("connection failures should report sanitized URLs");
        let msg = err.to_string();
        assert!(!msg.contains("url-secret"), "{msg}");
        assert!(!msg.contains("query-secret"), "{msg}");
        let log_msg = err.redacted_for_log();
        assert!(!log_msg.contains("url-secret"), "{log_msg}");
        assert!(!log_msg.contains("query-secret"), "{log_msg}");
    }
}
