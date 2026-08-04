//! Load the RomM OpenAPI spec at TUI startup: prefer the live server, fall back to cache,
//! then a bundled copy shipped in the binary. Used for save-sync compatibility and server version.

use anyhow::{anyhow, Result};
use serde_json::Value;
use std::path::Path;

use romm_api::client::RommClient;
use romm_api::openapi::EndpointRegistry;

/// OpenAPI document baked into the binary (`romm-tui/openapi.json` at build time).
const EMBEDDED_OPENAPI_JSON: &str =
    include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/openapi.json"));

fn openapi_from_cwd() -> Option<String> {
    let dir = std::env::current_dir().ok()?;
    let p = dir.join("openapi.json");
    if p.is_file() {
        std::fs::read_to_string(p).ok()
    } else {
        None
    }
}

fn fallback_openapi_body(cache_path: &Path, reason: &str) -> String {
    let mut body = None;
    if let Some(cwd) = openapi_from_cwd() {
        if EndpointRegistry::from_openapi_json(&cwd).is_ok() {
            tracing::warn!("Using ./openapi.json ({reason})");
            body = Some(cwd);
        }
    }
    if body.is_none() {
        if let Ok(cached) = std::fs::read_to_string(cache_path) {
            if EndpointRegistry::from_openapi_json(&cached).is_ok() {
                tracing::warn!(
                    "Using cached OpenAPI at {} ({reason})",
                    cache_path.display()
                );
                body = Some(cached);
            }
        }
    }
    body.unwrap_or_else(|| {
        tracing::warn!(
            "Using bundled OpenAPI spec ({reason}). \
             OpenAPI paths match the build-time snapshot; connect to refresh from your server."
        );
        EMBEDDED_OPENAPI_JSON.to_string()
    })
}

pub fn parse_openapi_info_version(json: &str) -> Option<String> {
    let v: Value = serde_json::from_str(json).ok()?;
    v.get("info")?.get("version")?.as_str().map(String::from)
}

/// Resolve OpenAPI JSON: try the server first (updates disk cache when the spec changes), then
/// `./openapi.json`, then the user cache file, then the embedded bundle.
///
/// Also calls `GET /api/heartbeat` for the RomM server version shown in Settings.
pub async fn sync_openapi_registry(
    client: &RommClient,
    cache_path: &Path,
) -> Result<(EndpointRegistry, Option<String>)> {
    let fetch_result = client.fetch_openapi_json().await;

    let openapi_body = match fetch_result {
        Ok(body) => {
            if let Err(e) = EndpointRegistry::from_openapi_json(&body) {
                let reason = format!("live OpenAPI invalid: {e}");
                fallback_openapi_body(cache_path, &reason)
            } else {
                let remote_ver = parse_openapi_info_version(&body);
                let local_ver = std::fs::read_to_string(cache_path)
                    .ok()
                    .as_deref()
                    .and_then(parse_openapi_info_version);

                let needs_write =
                    !cache_path.is_file() || local_ver.as_deref() != remote_ver.as_deref();

                if needs_write {
                    if let Some(parent) = cache_path.parent() {
                        std::fs::create_dir_all(parent)
                            .map_err(|e| anyhow!("create OpenAPI cache dir: {e}"))?;
                    }
                    std::fs::write(cache_path, &body).map_err(|e| {
                        anyhow!("write OpenAPI cache {}: {e}", cache_path.display())
                    })?;
                    tracing::info!(
                        "OpenAPI cache {} (version {:?})",
                        cache_path.display(),
                        remote_ver
                    );
                }
                body
            }
        }
        Err(e) => {
            let reason = format!("server unreachable: {}", e.redacted_for_log());
            fallback_openapi_body(cache_path, &reason)
        }
    };

    let registry = EndpointRegistry::from_openapi_json(&openapi_body)
        .map_err(|e| anyhow!("invalid OpenAPI document: {e}"))?;

    let server_version = client.rom_server_version_from_heartbeat().await;

    Ok((registry, server_version))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_info_version() {
        let j = r#"{"openapi":"3.0.0","info":{"version":"1.2.3"},"paths":{}}"#;
        assert_eq!(parse_openapi_info_version(j), Some("1.2.3".to_string()));
    }

    #[test]
    fn embedded_openapi_json_parses() {
        super::EndpointRegistry::from_openapi_json(EMBEDDED_OPENAPI_JSON)
            .expect("bundled openapi.json");
    }

    #[tokio::test]
    async fn sync_falls_back_to_bundled_when_remote_and_cache_unusable() {
        use romm_api::client::RommClient;
        use romm_api::config::{AuthConfig, Config, ExtrasDefaults};
        use wiremock::matchers::{method, path};
        use wiremock::{Mock, MockServer, ResponseTemplate};

        let server = MockServer::start().await;
        for p in ["/openapi.json", "/api/openapi.json"] {
            Mock::given(method("GET"))
                .and(path(p))
                .respond_with(ResponseTemplate::new(200).set_body_string(""))
                .mount(&server)
                .await;
        }

        let cache_dir = std::env::temp_dir().join(format!(
            "romm-openapi-sync-test-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        std::fs::create_dir_all(&cache_dir).unwrap();
        let cache_path = cache_dir.join("openapi.json");
        std::fs::write(&cache_path, "").unwrap();

        let client = RommClient::new(
            &Config {
                base_url: server.uri(),
                download_dir: ".".to_string(),
                use_https: false,
                auth: Some(AuthConfig::Bearer {
                    token: "t".to_string(),
                }),
                extras_defaults: ExtrasDefaults::default(),
                save_sync: Default::default(),
                roms_layout: Default::default(),
                theme: romm_api::config::default_theme_id(),
                tui_layout: Default::default(),
            },
            false,
        )
        .unwrap();

        let (registry, _) = sync_openapi_registry(&client, &cache_path)
            .await
            .expect("should fall back to bundled OpenAPI");
        assert!(
            !registry.endpoints.is_empty(),
            "bundled registry should have endpoints"
        );

        let _ = std::fs::remove_dir_all(&cache_dir);
    }

    #[tokio::test]
    async fn sync_falls_back_to_bundled_when_remote_openapi_is_invalid() {
        use romm_api::client::RommClient;
        use romm_api::config::{AuthConfig, Config, ExtrasDefaults};
        use wiremock::matchers::{method, path};
        use wiremock::{Mock, MockServer, ResponseTemplate};

        let server = MockServer::start().await;
        let invalid_openapi = r#"{
            "openapi": "3.0.0",
            "info": { "version": "999.0.0" },
            "paths": { "/api/broken": "not-a-path-item" }
        }"#;
        Mock::given(method("GET"))
            .and(path("/openapi.json"))
            .respond_with(ResponseTemplate::new(200).set_body_string(invalid_openapi))
            .mount(&server)
            .await;

        let cache_dir = std::env::temp_dir().join(format!(
            "romm-openapi-sync-invalid-test-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        std::fs::create_dir_all(&cache_dir).unwrap();
        let cache_path = cache_dir.join("openapi.json");

        let client = RommClient::new(
            &Config {
                base_url: server.uri(),
                download_dir: ".".to_string(),
                use_https: false,
                auth: Some(AuthConfig::Bearer {
                    token: "t".to_string(),
                }),
                extras_defaults: ExtrasDefaults::default(),
                save_sync: Default::default(),
                roms_layout: Default::default(),
                theme: romm_api::config::default_theme_id(),
                tui_layout: Default::default(),
            },
            false,
        )
        .unwrap();

        let (registry, _) = sync_openapi_registry(&client, &cache_path)
            .await
            .expect("invalid live OpenAPI should fall back to bundled OpenAPI");
        assert!(
            !registry.endpoints.is_empty(),
            "bundled registry should have endpoints"
        );
        assert!(
            !cache_path.is_file(),
            "invalid live OpenAPI must not poison the cache"
        );

        let _ = std::fs::remove_dir_all(&cache_dir);
    }
}
