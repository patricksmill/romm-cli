//! HTTP client wrapper around the ROMM API.
//!
//! `RommClient` owns a configured `reqwest::Client` plus base URL and
//! authentication settings. Frontends (CLI, TUI, or a future GUI) depend
//! on this type instead of talking to `reqwest` directly.

mod download;
mod openapi;
mod request;
mod response;
mod rom_update;
mod tasks;
mod upload;

pub use openapi::{api_root_url, openapi_spec_urls, resolve_openapi_root};

use base64::{engine::general_purpose, Engine as _};
use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION};
use reqwest::Client as HttpClient;

use crate::config::{AuthConfig, Config};
use crate::error::ApiError;

/// Optional query fields for save uploads used in sync flows.
#[derive(Debug, Clone, Default)]
pub struct SaveUploadOptions<'a> {
    pub emulator: Option<&'a str>,
    pub slot: Option<&'a str>,
    pub device_id: Option<&'a str>,
    pub session_id: Option<u64>,
    pub overwrite: bool,
}

/// High-level HTTP client for the ROMM API.
///
/// This type hides the details of `reqwest` and authentication headers
/// behind a small interface that all frontends can share.
#[derive(Clone)]
pub struct RommClient {
    pub(crate) http: HttpClient,
    pub(crate) base_url: String,
    pub(crate) auth: Option<AuthConfig>,
    pub(crate) verbose: bool,
}

/// Default `User-Agent` for every request. The stock `reqwest` UA is sometimes blocked at the HTTP
/// layer (403, etc.) by reverse proxies; override with env `ROMM_USER_AGENT` if needed.
pub(crate) fn http_user_agent() -> String {
    match std::env::var("ROMM_USER_AGENT") {
        Ok(s) if !s.trim().is_empty() => s,
        _ => format!(
            "Mozilla/5.0 (compatible; romm-cli/{}; +https://github.com/patricksmill/romm-cli)",
            env!("CARGO_PKG_VERSION")
        ),
    }
}

impl RommClient {
    /// Construct a new client from the high-level [`Config`].
    pub fn new(config: &Config, verbose: bool) -> Result<Self, ApiError> {
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

    pub fn base_url(&self) -> &str {
        &self.base_url
    }

    /// Build the HTTP headers for the current authentication mode.
    pub(crate) fn build_headers(&self) -> Result<HeaderMap, ApiError> {
        let mut headers = HeaderMap::new();

        if let Some(auth) = &self.auth {
            match auth {
                AuthConfig::Basic { username, password } => {
                    let creds = format!("{username}:{password}");
                    let encoded = general_purpose::STANDARD.encode(creds.as_bytes());
                    let value = format!("Basic {encoded}");
                    headers.insert(
                        AUTHORIZATION,
                        HeaderValue::from_str(&value).map_err(|_| {
                            ApiError::InvalidHeader("invalid basic auth header value".into())
                        })?,
                    );
                }
                AuthConfig::Bearer { token } => {
                    let value = format!("Bearer {token}");
                    headers.insert(
                        AUTHORIZATION,
                        HeaderValue::from_str(&value).map_err(|_| {
                            ApiError::InvalidHeader("invalid bearer auth header value".into())
                        })?,
                    );
                }
                AuthConfig::ApiKey { header, key } => {
                    let name = reqwest::header::HeaderName::from_bytes(header.as_bytes()).map_err(
                        |_| {
                            ApiError::InvalidHeader(
                                "invalid API_KEY_HEADER, must be a valid HTTP header name".into(),
                            )
                        },
                    )?;
                    headers.insert(
                        name,
                        HeaderValue::from_str(key).map_err(|_| {
                            ApiError::InvalidHeader("invalid API_KEY header value".into())
                        })?,
                    );
                }
            }
        }

        Ok(headers)
    }
}

#[cfg(test)]
mod tests {
    use super::response::decode_json_response_body;
    use serde_json::Value;

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
