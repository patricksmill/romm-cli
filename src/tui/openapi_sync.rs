//! Download and cache the RomM OpenAPI spec next to user config; refresh when `info.version` changes.

use anyhow::{anyhow, Result};
use serde_json::Value;
use std::path::Path;

use crate::client::RommClient;
use crate::tui::openapi::EndpointRegistry;

fn openapi_from_cwd() -> Option<String> {
    let dir = std::env::current_dir().ok()?;
    let p = dir.join("openapi.json");
    if p.is_file() {
        std::fs::read_to_string(p).ok()
    } else {
        None
    }
}

pub fn parse_openapi_info_version(json: &str) -> Option<String> {
    let v: Value = serde_json::from_str(json).ok()?;
    v.get("info")?.get("version")?.as_str().map(String::from)
}

fn heartbeat_rom_version(v: &Value) -> Option<String> {
    v.get("SYSTEM")?.get("VERSION")?.as_str().map(String::from)
}

/// Fetch OpenAPI from the server, update the on-disk cache when `info.version` differs from the
/// cached file, and build [`EndpointRegistry`]. If the fetch fails, uses an existing cache file.
///
/// Also calls `GET /api/heartbeat` for the RomM server version shown in Settings.
pub async fn sync_openapi_registry(
    client: &RommClient,
    cache_path: &Path,
) -> Result<(EndpointRegistry, Option<String>)> {
    let fetch_result = client.fetch_openapi_json().await;

    let openapi_body = match fetch_result {
        Ok(body) => {
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
                std::fs::write(cache_path, &body)
                    .map_err(|e| anyhow!("write OpenAPI cache {}: {e}", cache_path.display()))?;
                tracing::info!(
                    "OpenAPI cache {} (version {:?})",
                    cache_path.display(),
                    remote_ver
                );
            }
            body
        }
        Err(e) => {
            if let Some(body) = openapi_from_cwd() {
                tracing::warn!(
                    "Using ./openapi.json from the current directory (could not fetch from server: {:#})",
                    e
                );
                body
            } else {
                let cached = std::fs::read_to_string(cache_path).map_err(|_| {
                    anyhow!(
                        "Could not load OpenAPI: {:#}. \
                         Fix `API_BASE_URL` (try https if the UI uses TLS), ensure the server exposes /openapi.json, \
                         place an openapi.json in the current directory, or set ROMM_OPENAPI_PATH to a local file. \
                         No cache at {}.",
                        e,
                        cache_path.display()
                    )
                })?;
                tracing::warn!(
                    "Using cached OpenAPI at {} (server unreachable: {})",
                    cache_path.display(),
                    e
                );
                cached
            }
        }
    };

    let registry = EndpointRegistry::from_openapi_json(&openapi_body)
        .map_err(|e| anyhow!("invalid OpenAPI document: {e}"))?;

    let server_version = client
        .request_json("GET", "/api/heartbeat", &[], None)
        .await
        .ok()
        .as_ref()
        .and_then(heartbeat_rom_version);

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
}
