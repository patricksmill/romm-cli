//! Load the RomM OpenAPI spec at TUI startup: local sources first for instant paint,
//! then refresh from the live server in the background. Used for feature compatibility
//! and server version.

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

fn first_usable_local_openapi_body(cache_path: &Path) -> String {
    if let Some(cwd) = openapi_from_cwd() {
        if EndpointRegistry::from_openapi_json(&cwd).is_ok() {
            return cwd;
        }
    }
    if let Ok(cached) = std::fs::read_to_string(cache_path) {
        if EndpointRegistry::from_openapi_json(&cached).is_ok() {
            return cached;
        }
    }
    EMBEDDED_OPENAPI_JSON.to_string()
}

pub fn parse_openapi_info_version(json: &str) -> Option<String> {
    let v: Value = serde_json::from_str(json).ok()?;
    v.get("info")?.get("version")?.as_str().map(String::from)
}

/// Sync-only OpenAPI load for first paint: `./openapi.json` → user cache → embedded bundle.
/// Does not touch the network or heartbeat.
pub fn load_openapi_registry_local(cache_path: &Path) -> Result<EndpointRegistry> {
    let body = first_usable_local_openapi_body(cache_path);
    EndpointRegistry::from_openapi_json(&body).map_err(|e| anyhow!("invalid OpenAPI document: {e}"))
}

fn write_openapi_cache(cache_path: &Path, body: &str, remote_ver: Option<&str>) -> Result<()> {
    if let Some(parent) = cache_path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| anyhow!("create OpenAPI cache dir: {e}"))?;
    }
    std::fs::write(cache_path, body)
        .map_err(|e| anyhow!("write OpenAPI cache {}: {e}", cache_path.display()))?;
    tracing::info!(
        "OpenAPI cache {} (version {:?})",
        cache_path.display(),
        remote_ver
    );
    Ok(())
}

/// Fetch OpenAPI from the server (updating the disk cache) and read heartbeat version.
///
/// On fetch failure, returns `registry: None` and still attempts heartbeat so Settings can
/// show a version. Callers keep their local registry when `registry` is `None`.
pub async fn refresh_openapi_from_server(
    client: &RommClient,
    cache_path: &Path,
) -> (Option<EndpointRegistry>, Option<String>) {
    let registry = match client.fetch_openapi_json().await {
        Ok(body) => {
            let remote_ver = parse_openapi_info_version(&body);
            let local_ver = std::fs::read_to_string(cache_path)
                .ok()
                .as_deref()
                .and_then(parse_openapi_info_version);

            let needs_write =
                !cache_path.is_file() || local_ver.as_deref() != remote_ver.as_deref();

            if needs_write {
                if let Err(e) = write_openapi_cache(cache_path, &body, remote_ver.as_deref()) {
                    tracing::warn!("failed to write OpenAPI cache: {e:#}");
                }
            }

            match EndpointRegistry::from_openapi_json(&body) {
                Ok(reg) => Some(reg),
                Err(e) => {
                    tracing::warn!("invalid OpenAPI from server: {e}");
                    None
                }
            }
        }
        Err(e) => {
            tracing::warn!(
                "OpenAPI refresh failed (keeping local registry): {}",
                e.redacted_for_log()
            );
            None
        }
    };

    let server_version = client.rom_server_version_from_heartbeat().await;
    (registry, server_version)
}

/// Resolve OpenAPI JSON: try the server first (updates disk cache when the spec changes), then
/// local sources. Also calls `GET /api/heartbeat` for the RomM server version.
///
/// Prefer [`load_openapi_registry_local`] + [`refresh_openapi_from_server`] for TUI startup so
/// first paint is not blocked on the network.
pub async fn sync_openapi_registry(
    client: &RommClient,
    cache_path: &Path,
) -> Result<(EndpointRegistry, Option<String>)> {
    let (remote, server_version) = refresh_openapi_from_server(client, cache_path).await;
    if let Some(registry) = remote {
        return Ok((registry, server_version));
    }
    let registry = load_openapi_registry_local(cache_path)?;
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

    #[test]
    fn load_local_uses_embedded_when_cache_unusable() {
        let cache_dir = std::env::temp_dir().join(format!(
            "romm-openapi-local-test-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        std::fs::create_dir_all(&cache_dir).unwrap();
        let cache_path = cache_dir.join("openapi.json");
        std::fs::write(&cache_path, "").unwrap();

        let registry = load_openapi_registry_local(&cache_path).expect("local load");
        assert!(
            !registry.endpoints.is_empty(),
            "bundled registry should have endpoints"
        );

        let _ = std::fs::remove_dir_all(&cache_dir);
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
}
