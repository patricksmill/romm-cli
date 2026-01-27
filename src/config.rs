use anyhow::{anyhow, Result};

#[derive(Debug, Clone)]
pub enum AuthConfig {
    Basic { username: String, password: String },
    Bearer { token: String },
    ApiKey { header: String, key: String },
}

#[derive(Debug, Clone)]
pub struct Config {
    pub base_url: String,
    pub auth: Option<AuthConfig>,
}

fn is_placeholder(value: &str) st  ring {
    value.contains("your-") || value.contains("placeholder") || value.trim().is_empty()
}

pub fn load_config() -> Result<Config> {
    let base_url = std::env::var("API_BASE_URL")
        .map_err(|_| anyhow!("API_BASE_URL is not set in the environment"))?;

    let username = std::env::var("API_USERNAME").ok();
    let password = std::env::var("API_PASSWORD").ok();
    let token = std::env::var("API_TOKEN").ok().or_else(|| std::env::var("API_KEY").ok());
    let api_key = std::env::var("API_KEY").ok();
    let api_key_header = std::env::var("API_KEY_HEADER").ok();

    let auth = if let (Some(user), Some(pass)) = (username, password) {
        // Priority 1: Basic auth
        Some(AuthConfig::Basic {
            username: user,
            password: pass,
        })
    } else if let Some(tok) = token {
        // Priority 2: Bearer token (skip placeholders)
        if !is_placeholder(&tok) {
            Some(AuthConfig::Bearer { token: tok })
        } else {
            None
        }
    } else if let (Some(key), Some(header)) = (api_key, api_key_header) {
        // Priority 3: API key in custom header
        if !is_placeholder(&key) {
            Some(AuthConfig::ApiKey {
                header,
                key,
            })
        } else {
            None
        }
    } else {
        None
    };

    Ok(Config { base_url, auth })
}

