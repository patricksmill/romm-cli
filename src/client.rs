use anyhow::{anyhow, Result};
use base64::{engine::general_purpose, Engine as _};
use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION};
use reqwest::Client as HttpClient;

use crate::config::{AuthConfig, Config};
use crate::types::Platform;

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

    pub async fn get_platforms(&self) -> Result<Vec<Platform>> {
        let url = format!("{}/api/platforms", self.base_url.trim_end_matches('/'));
        let headers = self.build_headers()?;

        let resp = self
            .http
            .get(&url)
            .headers(headers)
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

        let platforms = resp.json::<Vec<Platform>>().await?;
        Ok(platforms)
    }
}

