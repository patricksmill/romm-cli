//! Configuration and authentication for the ROMM client.
//!
//! This module is deliberately independent of any particular frontend:
//! both the TUI and the command-line subcommands share the same `Config`
//! and `AuthConfig` types.

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

fn is_placeholder(value: &str) -> bool {
    value.contains("your-") || value.contains("placeholder") || value.trim().is_empty()
}

pub fn load_config() -> Result<Config> {
    let base_url = std::env::var("API_BASE_URL")
        .map_err(|_| anyhow!("API_BASE_URL is not set in the environment"))?;

    let username = std::env::var("API_USERNAME").ok();
    let password = std::env::var("API_PASSWORD").ok();
    let token = std::env::var("API_TOKEN")
        .ok()
        .or_else(|| std::env::var("API_KEY").ok());
    let api_key = std::env::var("API_KEY").ok();
    let api_key_header = std::env::var("API_KEY_HEADER").ok();

    let auth = if let (Some(user), Some(pass)) = (username, password) {
        // Priority 1: Basic auth
        Some(AuthConfig::Basic {
            username: user,
            password: pass,
        })
    } else if let (Some(key), Some(header)) = (api_key, api_key_header) {
        // Priority 2: API key in custom header (when both set, prefer over bearer)
        if !is_placeholder(&key) {
            Some(AuthConfig::ApiKey { header, key })
        } else {
            None
        }
    } else if let Some(tok) = token {
        // Priority 3: Bearer token (skip placeholders)
        if !is_placeholder(&tok) {
            Some(AuthConfig::Bearer { token: tok })
        } else {
            None
        }
    } else {
        None
    };

    Ok(Config { base_url, auth })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Mutex, OnceLock};

    fn env_lock() -> &'static Mutex<()> {
        static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
        LOCK.get_or_init(|| Mutex::new(()))
    }

    fn clear_auth_env() {
        for key in [
            "API_BASE_URL",
            "API_USERNAME",
            "API_PASSWORD",
            "API_TOKEN",
            "API_KEY",
            "API_KEY_HEADER",
        ] {
            std::env::remove_var(key);
        }
    }

    #[test]
    fn prefers_basic_auth_over_other_modes() {
        let _guard = env_lock().lock().expect("env lock");
        clear_auth_env();
        std::env::set_var("API_BASE_URL", "http://example.test");
        std::env::set_var("API_USERNAME", "user");
        std::env::set_var("API_PASSWORD", "pass");
        std::env::set_var("API_TOKEN", "token");
        std::env::set_var("API_KEY", "apikey");
        std::env::set_var("API_KEY_HEADER", "X-Api-Key");

        let cfg = load_config().expect("config should load");
        match cfg.auth {
            Some(AuthConfig::Basic { username, password }) => {
                assert_eq!(username, "user");
                assert_eq!(password, "pass");
            }
            _ => panic!("expected basic auth"),
        }
    }

    #[test]
    fn uses_api_key_header_when_token_missing() {
        let _guard = env_lock().lock().expect("env lock");
        clear_auth_env();
        std::env::set_var("API_BASE_URL", "http://example.test");
        std::env::set_var("API_KEY", "real-key");
        std::env::set_var("API_KEY_HEADER", "X-Api-Key");

        let cfg = load_config().expect("config should load");
        match cfg.auth {
            Some(AuthConfig::ApiKey { header, key }) => {
                assert_eq!(header, "X-Api-Key");
                assert_eq!(key, "real-key");
            }
            _ => panic!("expected api key auth"),
        }
    }

    #[test]
    fn ignores_placeholder_bearer_token() {
        let _guard = env_lock().lock().expect("env lock");
        clear_auth_env();
        std::env::set_var("API_BASE_URL", "http://example.test");
        std::env::set_var("API_TOKEN", "your-bearer-token-here");

        let cfg = load_config().expect("config should load");
        assert!(cfg.auth.is_none(), "placeholder token should be ignored");
    }
}
