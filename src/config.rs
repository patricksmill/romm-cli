//! Configuration and authentication for the ROMM client.
//!
//! This module is deliberately independent of any particular frontend:
//! both the TUI and the command-line subcommands share the same `Config`
//! and `AuthConfig` types.
//!
//! ## Environment file precedence
//!
//! Call [`load_config`] to read config:
//!
//! 1. Variables already set in the process environment (highest priority).
//! 2. Project `.env` in the current working directory (via `dotenvy`).
//! 3. User config: `{config_dir}/romm-cli/.env` — fills keys not already set (so a repo `.env` wins over user defaults).
//! 4. OS keyring — secrets stored by `romm-cli init` (lowest priority fallback).

use std::path::PathBuf;

use anyhow::{anyhow, Context, Result};

use serde::{Deserialize, Serialize};

// ---------------------------------------------------------------------------
// Types
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AuthConfig {
    Basic { username: String, password: String },
    Bearer { token: String },
    ApiKey { header: String, key: String },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub base_url: String,
    pub download_dir: String,
    pub use_https: bool,
    pub auth: Option<AuthConfig>,
}

fn is_placeholder(value: &str) -> bool {
    value.contains("your-") || value.contains("placeholder") || value.trim().is_empty()
}

/// RomM site URL: the same origin you use in the browser (scheme, host, optional port).
///
/// Trims whitespace and trailing `/`, and removes a trailing `/api` segment if present. HTTP
/// calls use paths such as `/api/platforms`; they must not double up with `.../api/api/...`.
pub fn normalize_romm_origin(url: &str) -> String {
    let mut s = url.trim().trim_end_matches('/').to_string();
    if s.ends_with("/api") {
        s.truncate(s.len() - 4);
    }
    s.trim_end_matches('/').to_string()
}

// ---------------------------------------------------------------------------
// Keyring helpers
// ---------------------------------------------------------------------------

const KEYRING_SERVICE: &str = "romm-cli";

/// Store a secret in the OS keyring under the `romm-cli` service name.
pub fn keyring_store(key: &str, value: &str) -> Result<()> {
    let entry = keyring::Entry::new(KEYRING_SERVICE, key)
        .map_err(|e| anyhow!("keyring entry error: {e}"))?;
    entry
        .set_password(value)
        .map_err(|e| anyhow!("keyring set error: {e}"))
}

/// Retrieve a secret from the OS keyring, returning `None` if not found.
fn keyring_get(key: &str) -> Option<String> {
    let entry = keyring::Entry::new(KEYRING_SERVICE, key).ok()?;
    entry.get_password().ok()
}

// ---------------------------------------------------------------------------
// Paths
// ---------------------------------------------------------------------------

/// Directory for user-level config (`romm-cli` under the OS config dir).
pub fn user_config_dir() -> Option<PathBuf> {
    if let Ok(dir) = std::env::var("ROMM_TEST_CONFIG_DIR") {
        return Some(PathBuf::from(dir));
    }
    dirs::config_dir().map(|d| d.join("romm-cli"))
}

/// Path to the user-level `config.json` file (`.../romm-cli/config.json`).
pub fn user_config_json_path() -> Option<PathBuf> {
    user_config_dir().map(|d| d.join("config.json"))
}

/// Where the OpenAPI spec is cached (`.../romm-cli/openapi.json`).
///
/// Override with `ROMM_OPENAPI_PATH` (absolute or relative path).
pub fn openapi_cache_path() -> Result<PathBuf> {
    if let Ok(p) = std::env::var("ROMM_OPENAPI_PATH") {
        return Ok(PathBuf::from(p));
    }
    let dir = user_config_dir().ok_or_else(|| {
        anyhow!("Could not resolve config directory. Set ROMM_OPENAPI_PATH to store openapi.json.")
    })?;
    Ok(dir.join("openapi.json"))
}

// ---------------------------------------------------------------------------
// Loading
// ---------------------------------------------------------------------------


/// Read an env var, falling back to the OS keyring if unset or empty.
///
/// A line like `API_PASSWORD=` in a project `.env` sets the variable to the empty string; we treat
/// that as "not set" so the keyring (e.g. after `romm-cli init` / TUI setup) is still used.
fn env_or_keyring(key: &str) -> Option<String> {
    match std::env::var(key) {
        Ok(s) if !s.trim().is_empty() => Some(s),
        Ok(_) => keyring_get(key),
        Err(_) => keyring_get(key),
    }
}

fn env_nonempty(key: &str) -> Option<String> {
    std::env::var(key).ok().filter(|s| !s.trim().is_empty())
}

pub fn load_config() -> Result<Config> {
    // Attempt to load from JSON first
    if let Some(path) = user_config_json_path() {
        if path.is_file() {
            if let Ok(content) = std::fs::read_to_string(&path) {
                if let Ok(mut config) = serde_json::from_str::<Config>(&content) {
                    // Populate secrets from keyring if they are missing or placeholders
                    if let Some(auth) = &mut config.auth {
                        match auth {
                            AuthConfig::Basic { password, .. } => {
                                if is_placeholder(password) {
                                    if let Some(p) = keyring_get("API_PASSWORD") {
                                        *password = p;
                                    }
                                }
                            }
                            AuthConfig::Bearer { token } => {
                                if is_placeholder(token) {
                                    if let Some(t) = keyring_get("API_TOKEN") {
                                        *token = t;
                                    }
                                }
                            }
                            AuthConfig::ApiKey { key, .. } => {
                                if is_placeholder(key) {
                                    if let Some(k) = keyring_get("API_KEY") {
                                        *key = k;
                                    }
                                }
                            }
                        }
                    }
                    return Ok(config);
                }
            }
        }
    }

    // Fallback to environment variables
    let base_raw = std::env::var("API_BASE_URL").map_err(|_| {
        anyhow!(
            "API_BASE_URL is not set. Set it in the environment, a config.json file, or run: romm-cli init"
        )
    })?;
    let mut base_url = normalize_romm_origin(&base_raw);

    let download_dir = env_nonempty("ROMM_DOWNLOAD_DIR").unwrap_or_else(|| {
        dirs::download_dir()
            .unwrap_or_else(|| dirs::home_dir().unwrap_or_default().join("Downloads"))
            .join("romm-cli")
            .display()
            .to_string()
    });

    let use_https = std::env::var("API_USE_HTTPS")
        .map(|s| s.to_lowercase() == "true")
        .unwrap_or(true);

    if use_https && base_url.starts_with("http://") {
        base_url = base_url.replace("http://", "https://");
    }

    let username = env_nonempty("API_USERNAME");
    let password = env_or_keyring("API_PASSWORD");
    let token = env_or_keyring("API_TOKEN").or_else(|| env_or_keyring("API_KEY"));
    let api_key = env_or_keyring("API_KEY");
    let api_key_header = env_nonempty("API_KEY_HEADER");

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

    Ok(Config {
        base_url,
        download_dir,
        use_https,
        auth,
    })
}


/// Write user-level `romm-cli/config.json` and store secrets in the OS keyring when possible
/// (same layout as interactive `romm-cli init`).
pub fn persist_user_config(
    base_url: &str,
    download_dir: &str,
    use_https: bool,
    auth: Option<AuthConfig>,
) -> Result<()> {
    let Some(path) = user_config_json_path() else {
        return Err(anyhow!(
            "Could not determine config directory (no HOME / APPDATA?)."
        ));
    };
    let dir = path
        .parent()
        .ok_or_else(|| anyhow!("invalid config path"))?;
    std::fs::create_dir_all(dir).with_context(|| format!("create {}", dir.display()))?;

    let mut config_to_save = Config {
        base_url: base_url.to_string(),
        download_dir: download_dir.to_string(),
        use_https,
        auth: auth.clone(),
    };

    match &mut config_to_save.auth {
        None => {}
        Some(AuthConfig::Basic { password, .. }) => {
            if let Err(e) = keyring_store("API_PASSWORD", password) {
                tracing::warn!("keyring store API_PASSWORD: {e}; writing plaintext to config.json");
            } else {
                *password = "<stored-in-keyring>".to_string();
            }
        }
        Some(AuthConfig::Bearer { token }) => {
            if let Err(e) = keyring_store("API_TOKEN", token) {
                tracing::warn!("keyring store API_TOKEN: {e}; writing plaintext to config.json");
            } else {
                *token = "<stored-in-keyring>".to_string();
            }
        }
        Some(AuthConfig::ApiKey { key, .. }) => {
            if let Err(e) = keyring_store("API_KEY", key) {
                tracing::warn!("keyring store API_KEY: {e}; writing plaintext to config.json");
            } else {
                *key = "<stored-in-keyring>".to_string();
            }
        }
    }

    let content = serde_json::to_string_pretty(&config_to_save)?;
    {
        use std::io::Write;
        let mut f =
            std::fs::File::create(&path).with_context(|| format!("write {}", path.display()))?;
        f.write_all(content.as_bytes())?;
    }

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = std::fs::metadata(&path)?.permissions();
        perms.set_mode(0o600);
        std::fs::set_permissions(&path, perms)?;
    }

    Ok(())
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
            "API_USE_HTTPS",
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
    fn normalizes_api_base_url_and_enforces_https_by_default() {
        let _guard = env_lock().lock().expect("env lock");
        clear_auth_env();
        std::env::set_var("API_BASE_URL", "http://romm.example/api/");
        let cfg = load_config().expect("config");
        // Upgraded to https by default
        assert_eq!(cfg.base_url, "https://romm.example");
    }

    #[test]
    fn does_not_enforce_https_if_toggle_is_false() {
        let _guard = env_lock().lock().expect("env lock");
        clear_auth_env();
        std::env::set_var("API_BASE_URL", "http://romm.example/api/");
        std::env::set_var("API_USE_HTTPS", "false");
        let cfg = load_config().expect("config");
        assert_eq!(cfg.base_url, "http://romm.example");
    }

    #[test]
    fn normalize_romm_origin_trims_and_strips_api_suffix() {
        assert_eq!(
            normalize_romm_origin("http://localhost:8080/api/"),
            "http://localhost:8080"
        );
        assert_eq!(
            normalize_romm_origin("https://x.example"),
            "https://x.example"
        );
    }

    #[test]
    fn empty_api_username_does_not_enable_basic() {
        let _guard = env_lock().lock().expect("env lock");
        clear_auth_env();
        std::env::set_var("API_BASE_URL", "http://example.test");
        std::env::set_var("API_USERNAME", "");
        std::env::set_var("API_PASSWORD", "secret");

        let cfg = load_config().expect("config should load");
        assert!(
            cfg.auth.is_none(),
            "empty API_USERNAME should not pair with password for Basic"
        );
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

    #[test]
    fn loads_from_user_json_file() {
        let _guard = env_lock().lock().expect("env lock");
        clear_auth_env();
        std::env::remove_var("API_BASE_URL");

        let ts = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let base = std::env::temp_dir().join(format!("romm-json-{ts}"));
        std::fs::create_dir_all(&base).unwrap();
        
        let config_json = r#"{
            "base_url": "http://from-json-file.test",
            "download_dir": "/tmp/downloads",
            "use_https": false,
            "auth": null
        }"#;
        
        std::fs::write(
            base.join("config.json"),
            config_json,
        )
        .unwrap();

        std::env::set_var("ROMM_TEST_CONFIG_DIR", base.as_os_str());

        let cfg = load_config().expect("load from user config.json");
        assert_eq!(cfg.base_url, "http://from-json-file.test");
        assert_eq!(cfg.download_dir, "/tmp/downloads");
        assert_eq!(cfg.use_https, false);

        std::env::remove_var("ROMM_TEST_CONFIG_DIR");
        std::env::remove_var("API_BASE_URL");
        let _ = std::fs::remove_dir_all(&base);
    }
}
