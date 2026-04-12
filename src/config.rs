//! Configuration and authentication for the ROMM client.
//!
//! This module is deliberately independent of any particular frontend:
//! both the TUI and the command-line subcommands share the same `Config`
//! and `AuthConfig` types.
//!
//! ## Configuration precedence
//!
//! Call [`load_config`] to read config:
//!
//! 1. Variables already set in the process environment (highest priority).
//! 2. User `config.json` (see [`user_config_json_path`]) at `{config_dir}/romm-cli/config.json` — fills any of
//!    `API_BASE_URL`, `ROMM_DOWNLOAD_DIR`, `API_USE_HTTPS`, and auth fields **not** set by the environment.
//!
//! There is **no** automatic loading of a `.env` file; set variables in your shell or process manager,
//! or rely on `config.json` written by `romm-cli init` / the TUI setup wizard.
//!
//! After env + JSON merge, secrets that are still placeholders (including [`KEYRING_SECRET_PLACEHOLDER`])
//! are resolved via the OS keyring (`keyring` crate, service name `romm-cli`). On Windows the stored
//! credential target is typically `API_TOKEN.romm-cli`, `API_PASSWORD.romm-cli`, or `API_KEY.romm-cli`.
//!
//! ## `load_config` vs `config.json`
//!
//! [`load_config`] merges sources **per field**: process environment wins over values from
//! `config.json` for `API_BASE_URL`, `ROMM_DOWNLOAD_DIR`, `API_USE_HTTPS`, and auth-related
//! fields. The keyring is used only to replace placeholder or sentinel secret strings after that merge.

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

/// Written to `config.json` when the real secret is stored in the OS keyring (`persist_user_config`).
pub const KEYRING_SECRET_PLACEHOLDER: &str = "<stored-in-keyring>";

/// True if `s` is the sentinel written to disk when the secret lives in the keyring.
pub fn is_keyring_placeholder(s: &str) -> bool {
    s == KEYRING_SECRET_PLACEHOLDER
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
pub(crate) fn keyring_get(key: &str) -> Option<String> {
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

/// Reads `config.json` from disk only (no env merge, no keyring resolution).
/// Used by the TUI setup wizard to detect `<stored-in-keyring>` placeholders.
pub fn read_user_config_json_from_disk() -> Option<Config> {
    let path = user_config_json_path()?;
    let content = std::fs::read_to_string(path).ok()?;
    serde_json::from_str(&content).ok()
}

/// Auth to pass to [`persist_user_config`] when saving non-auth fields (e.g. TUI Settings).
///
/// Prefer the in-memory [`Config::auth`]. If it is `None` (e.g. [`load_config`] could not read the
/// token from the keyring), reuse `auth` from [`read_user_config_json_from_disk`] so we do not
/// overwrite `config.json` with `"auth": null` while the file still held a bearer sentinel.
pub fn auth_for_persist_merge(in_memory: Option<AuthConfig>) -> Option<AuthConfig> {
    in_memory.or_else(|| read_user_config_json_from_disk().and_then(|c| c.auth))
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

fn env_nonempty(key: &str) -> Option<String> {
    std::env::var(key).ok().filter(|s| !s.trim().is_empty())
}

pub fn load_config() -> Result<Config> {
    // 1. Load from JSON first (if it exists)
    let mut json_config = None;
    if let Some(path) = user_config_json_path() {
        if path.is_file() {
            if let Ok(content) = std::fs::read_to_string(&path) {
                if let Ok(config) = serde_json::from_str::<Config>(&content) {
                    json_config = Some(config);
                }
            }
        }
    }

    // 2. Resolve base_url
    let base_raw = env_nonempty("API_BASE_URL")
        .or_else(|| json_config.as_ref().map(|c| c.base_url.clone()))
        .ok_or_else(|| {
            anyhow!(
                "API_BASE_URL is not set. Set it in the environment, a config.json file, or run: romm-cli init"
            )
        })?;
    let mut base_url = normalize_romm_origin(&base_raw);

    // 3. Resolve download_dir
    let download_dir = env_nonempty("ROMM_DOWNLOAD_DIR")
        .or_else(|| json_config.as_ref().map(|c| c.download_dir.clone()))
        .unwrap_or_else(|| {
            dirs::download_dir()
                .unwrap_or_else(|| dirs::home_dir().unwrap_or_default().join("Downloads"))
                .join("romm-cli")
                .display()
                .to_string()
        });

    // 4. Resolve use_https
    let use_https = if let Ok(s) = std::env::var("API_USE_HTTPS") {
        s.to_lowercase() == "true"
    } else if let Some(c) = &json_config {
        c.use_https
    } else {
        true
    };

    if use_https && base_url.starts_with("http://") {
        base_url = base_url.replace("http://", "https://");
    }

    // 5. Resolve Auth
    let mut username = env_nonempty("API_USERNAME");
    let mut password = env_nonempty("API_PASSWORD");
    let mut token = env_nonempty("API_TOKEN");
    let mut api_key = env_nonempty("API_KEY");
    let mut api_key_header = env_nonempty("API_KEY_HEADER");

    if let Some(c) = &json_config {
        if let Some(auth) = &c.auth {
            match auth {
                AuthConfig::Basic {
                    username: u,
                    password: p,
                } => {
                    if username.is_none() {
                        username = Some(u.clone());
                    }
                    if password.is_none() {
                        password = Some(p.clone());
                    }
                }
                AuthConfig::Bearer { token: t } => {
                    if token.is_none() {
                        token = Some(t.clone());
                    }
                }
                AuthConfig::ApiKey { header: h, key: k } => {
                    if api_key_header.is_none() {
                        api_key_header = Some(h.clone());
                    }
                    if api_key.is_none() {
                        api_key = Some(k.clone());
                    }
                }
            }
        }
    }

    // Resolve placeholders from keyring (including disk sentinel `<stored-in-keyring>`).
    if let Some(p) = &password {
        if is_placeholder(p) || is_keyring_placeholder(p) {
            if let Some(k) = keyring_get("API_PASSWORD") {
                password = Some(k);
            }
        }
    } else {
        password = keyring_get("API_PASSWORD");
    }

    if let Some(t) = &token {
        if is_placeholder(t) || is_keyring_placeholder(t) {
            if let Some(k) = keyring_get("API_TOKEN") {
                token = Some(k);
            }
        }
    } else {
        token = keyring_get("API_TOKEN");
    }

    if let Some(k) = &api_key {
        if is_placeholder(k) || is_keyring_placeholder(k) {
            if let Some(kr) = keyring_get("API_KEY") {
                api_key = Some(kr);
            }
        }
    } else {
        api_key = keyring_get("API_KEY");
    }

    if let Some(ref p) = password {
        if is_keyring_placeholder(p) {
            tracing::warn!(
                "Could not read API_PASSWORD from the OS keyring; value is still <stored-in-keyring>. \
                 On Windows, look for a Generic credential with target API_PASSWORD.romm-cli."
            );
        }
    }
    if let Some(ref t) = token {
        if is_keyring_placeholder(t) {
            tracing::warn!(
                "Could not read API_TOKEN from the OS keyring; value is still <stored-in-keyring>. \
                 On Windows, look for a Generic credential with target API_TOKEN.romm-cli."
            );
        }
    }
    if let Some(ref k) = api_key {
        if is_keyring_placeholder(k) {
            tracing::warn!(
                "Could not read API_KEY from the OS keyring; value is still <stored-in-keyring>. \
                 On Windows, look for a Generic credential with target API_KEY.romm-cli."
            );
        }
    }

    let auth = if let (Some(user), Some(pass)) = (username, password) {
        if !is_placeholder(&pass) && !is_keyring_placeholder(&pass) {
            Some(AuthConfig::Basic {
                username: user,
                password: pass,
            })
        } else {
            None
        }
    } else if let (Some(key), Some(header)) = (api_key, api_key_header) {
        if !is_placeholder(&key) && !is_keyring_placeholder(&key) {
            Some(AuthConfig::ApiKey { header, key })
        } else {
            None
        }
    } else if let Some(tok) = token {
        if !is_placeholder(&tok) && !is_keyring_placeholder(&tok) {
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
                *password = KEYRING_SECRET_PLACEHOLDER.to_string();
            }
        }
        Some(AuthConfig::Bearer { token }) => {
            if let Err(e) = keyring_store("API_TOKEN", token) {
                tracing::warn!("keyring store API_TOKEN: {e}; writing plaintext to config.json");
            } else {
                *token = KEYRING_SECRET_PLACEHOLDER.to_string();
            }
        }
        Some(AuthConfig::ApiKey { key, .. }) => {
            if let Err(e) = keyring_store("API_KEY", key) {
                tracing::warn!("keyring store API_KEY: {e}; writing plaintext to config.json");
            } else {
                *key = KEYRING_SECRET_PLACEHOLDER.to_string();
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
    use std::sync::{Mutex, MutexGuard, OnceLock};

    fn env_lock() -> &'static Mutex<()> {
        static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
        LOCK.get_or_init(|| Mutex::new(()))
    }

    struct TestEnv {
        _guard: MutexGuard<'static, ()>,
        config_dir: PathBuf,
    }

    impl TestEnv {
        fn new() -> Self {
            let guard = env_lock().lock().expect("env lock");
            clear_auth_env();

            let ts = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos();
            let config_dir = std::env::temp_dir().join(format!("romm-config-test-{ts}"));
            std::fs::create_dir_all(&config_dir).unwrap();
            std::env::set_var("ROMM_TEST_CONFIG_DIR", &config_dir);

            Self {
                _guard: guard,
                config_dir,
            }
        }
    }

    impl Drop for TestEnv {
        fn drop(&mut self) {
            clear_auth_env();
            std::env::remove_var("ROMM_TEST_CONFIG_DIR");
            let _ = std::fs::remove_dir_all(&self.config_dir);
        }
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
            "ROMM_TEST_CONFIG_DIR",
        ] {
            std::env::remove_var(key);
        }
    }

    #[test]
    fn prefers_basic_auth_over_other_modes() {
        let _env = TestEnv::new();
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
        let _env = TestEnv::new();
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
        let _env = TestEnv::new();
        std::env::set_var("API_BASE_URL", "http://romm.example/api/");
        let cfg = load_config().expect("config");
        // Upgraded to https by default
        assert_eq!(cfg.base_url, "https://romm.example");
    }

    #[test]
    fn does_not_enforce_https_if_toggle_is_false() {
        let _env = TestEnv::new();
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
        let _env = TestEnv::new();
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
        let _env = TestEnv::new();
        std::env::set_var("API_BASE_URL", "http://example.test");
        std::env::set_var("API_TOKEN", "your-bearer-token-here");

        let cfg = load_config().expect("config should load");
        assert!(cfg.auth.is_none(), "placeholder token should be ignored");
    }

    #[test]
    fn loads_from_user_json_file() {
        let env = TestEnv::new();
        let config_json = r#"{
            "base_url": "http://from-json-file.test",
            "download_dir": "/tmp/downloads",
            "use_https": false,
            "auth": null
        }"#;

        std::fs::write(env.config_dir.join("config.json"), config_json).unwrap();

        let cfg = load_config().expect("load from user config.json");
        assert_eq!(cfg.base_url, "http://from-json-file.test");
        assert_eq!(cfg.download_dir, "/tmp/downloads");
        assert!(!cfg.use_https);
    }

    #[test]
    fn auth_for_persist_merge_prefers_in_memory() {
        let env = TestEnv::new();
        let on_disk = r#"{
            "base_url": "http://disk.test",
            "download_dir": "/tmp",
            "use_https": false,
            "auth": { "Bearer": { "token": "from-disk" } }
        }"#;
        std::fs::write(env.config_dir.join("config.json"), on_disk).unwrap();

        let mem = Some(AuthConfig::Bearer {
            token: "from-memory".into(),
        });
        let merged = auth_for_persist_merge(mem.clone());
        assert_eq!(format!("{:?}", merged), format!("{:?}", mem));
    }

    #[test]
    fn auth_for_persist_merge_falls_back_to_disk_when_memory_empty() {
        let env = TestEnv::new();
        let on_disk = r#"{
            "base_url": "http://disk.test",
            "download_dir": "/tmp",
            "use_https": false,
            "auth": { "Bearer": { "token": "<stored-in-keyring>" } }
        }"#;
        std::fs::write(env.config_dir.join("config.json"), on_disk).unwrap();

        let merged = auth_for_persist_merge(None);
        match merged {
            Some(AuthConfig::Bearer { token }) => {
                assert_eq!(token, KEYRING_SECRET_PLACEHOLDER);
            }
            _ => panic!("expected bearer auth from disk"),
        }
    }

    #[test]
    fn bearer_keyring_sentinel_without_keyring_entry_yields_no_auth() {
        let env = TestEnv::new();
        std::env::set_var("API_BASE_URL", "http://example.test");
        let config_json = r#"{
            "base_url": "http://example.test",
            "download_dir": "/tmp",
            "use_https": false,
            "auth": { "Bearer": { "token": "<stored-in-keyring>" } }
        }"#;
        std::fs::write(env.config_dir.join("config.json"), config_json).unwrap();

        let cfg = load_config().expect("load");
        assert!(
            cfg.auth.is_none(),
            "unresolved keyring sentinel must not become Bearer auth in Config"
        );
    }
}
