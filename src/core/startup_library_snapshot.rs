//! Compact on-disk snapshot of library **metadata** (platforms + merged collections).
//!
//! Used by the TUI to paint the library screen immediately on entry while a
//! background refresh reconciles with the API. Full ROM lists remain on-demand
//! and continue to use [`crate::core::cache::RomCache`].

use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::client::RommClient;
use crate::endpoints::collections::{
    merge_all_collection_sources, ListCollections, ListSmartCollections, ListVirtualCollections,
};
use crate::endpoints::platforms::ListPlatforms;
use crate::types::{Collection, Platform};

const SNAPSHOT_VERSION: u32 = 1;
const DEFAULT_FILE: &str = "library-metadata-snapshot.json";

/// On-disk JSON envelope.
#[derive(Debug, Serialize, Deserialize)]
struct SnapshotFile {
    version: u32,
    /// Unix timestamp (seconds) when saved.
    saved_at_secs: u64,
    platforms: Vec<Platform>,
    collections: Vec<Collection>,
}

/// Result of a live metadata fetch (background or cold path).
#[derive(Debug, Clone)]
pub struct LibraryMetadataFetch {
    pub platforms: Vec<Platform>,
    pub collections: Vec<Collection>,
    pub warnings: Vec<String>,
}

/// Load a snapshot from disk if present and valid.
pub fn load_snapshot() -> Option<LibraryMetadataFetch> {
    let path = snapshot_path();
    let data = std::fs::read_to_string(&path).ok()?;
    let file: SnapshotFile = serde_json::from_str(&data).ok()?;
    if file.version != SNAPSHOT_VERSION {
        return None;
    }
    Some(LibraryMetadataFetch {
        platforms: file.platforms,
        collections: file.collections,
        warnings: Vec::new(),
    })
}

/// Persist merged metadata for next startup.
pub fn save_snapshot(platforms: &[Platform], collections: &[Collection]) {
    let path = snapshot_path();
    let file = SnapshotFile {
        version: SNAPSHOT_VERSION,
        saved_at_secs: unix_now_secs(),
        platforms: platforms.to_vec(),
        collections: collections.to_vec(),
    };
    if let Some(parent) = path.parent() {
        if let Err(err) = std::fs::create_dir_all(parent) {
            tracing::warn!(
                "Failed to create library metadata snapshot directory {:?}: {}",
                parent,
                err
            );
            return;
        }
    }
    match serde_json::to_string(&file) {
        Ok(json) => {
            if let Err(err) = std::fs::write(&path, json) {
                tracing::warn!(
                    "Failed to write library metadata snapshot {:?}: {}",
                    path.display(),
                    err
                );
            }
        }
        Err(err) => tracing::warn!("Failed to serialize library metadata snapshot: {}", err),
    }
}

/// Effective path to the snapshot file.
pub fn snapshot_effective_path() -> PathBuf {
    snapshot_path()
}

fn snapshot_path() -> PathBuf {
    if let Ok(p) = std::env::var("ROMM_LIBRARY_METADATA_SNAPSHOT_PATH") {
        return PathBuf::from(p);
    }
    if let Ok(dir) = std::env::var("ROMM_TEST_LIBRARY_SNAPSHOT_DIR") {
        return PathBuf::from(dir).join(DEFAULT_FILE);
    }
    default_snapshot_path()
}

fn default_snapshot_path() -> PathBuf {
    if let Some(dir) = dirs::cache_dir() {
        return dir.join("romm-cli").join(DEFAULT_FILE);
    }
    PathBuf::from(DEFAULT_FILE)
}

fn unix_now_secs() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

/// Fetch platforms and merged collections from the API (same behavior as the
/// former synchronous TUI main-menu path, including virtual collection timeout).
pub async fn fetch_merged_library_metadata(client: &RommClient) -> LibraryMetadataFetch {
    use std::time::Duration;

    let mut warnings = Vec::new();

    let platforms = match client.call(&ListPlatforms).await {
        Ok(p) => p,
        Err(e) => {
            warnings.push(format!("GET /api/platforms: {e:#}"));
            Vec::new()
        }
    };

    let manual = match client.call(&ListCollections).await {
        Ok(c) => c.into_vec(),
        Err(e) => {
            warnings.push(format!("GET /api/collections: {e:#}"));
            Vec::new()
        }
    };
    let smart = match client.call(&ListSmartCollections).await {
        Ok(c) => c.into_vec(),
        Err(e) => {
            warnings.push(format!("GET /api/collections/smart: {e:#}"));
            Vec::new()
        }
    };
    let virtual_rows =
        match tokio::time::timeout(Duration::from_secs(3), client.call(&ListVirtualCollections))
            .await
        {
            Ok(Ok(v)) => v,
            Ok(Err(e)) => {
                warnings.push(format!("GET /api/collections/virtual?type=all: {e:#}"));
                Vec::new()
            }
            Err(_) => {
                warnings
                    .push("GET /api/collections/virtual?type=all: timed out after 3s".to_string());
                Vec::new()
            }
        };

    let collections = merge_all_collection_sources(manual, smart, virtual_rows);

    LibraryMetadataFetch {
        platforms,
        collections,
        warnings,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::Collection;
    use std::sync::{Mutex, MutexGuard, OnceLock};

    fn env_lock() -> &'static Mutex<()> {
        static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
        LOCK.get_or_init(|| Mutex::new(()))
    }

    struct TestEnv {
        _guard: MutexGuard<'static, ()>,
    }

    impl TestEnv {
        fn new() -> Self {
            let guard = env_lock().lock().expect("env lock");
            std::env::remove_var("ROMM_LIBRARY_METADATA_SNAPSHOT_PATH");
            std::env::remove_var("ROMM_TEST_LIBRARY_SNAPSHOT_DIR");
            Self { _guard: guard }
        }
    }

    impl Drop for TestEnv {
        fn drop(&mut self) {
            std::env::remove_var("ROMM_LIBRARY_METADATA_SNAPSHOT_PATH");
            std::env::remove_var("ROMM_TEST_LIBRARY_SNAPSHOT_DIR");
        }
    }

    fn sample_fetch() -> LibraryMetadataFetch {
        LibraryMetadataFetch {
            platforms: vec![Platform {
                id: 1,
                slug: "nes".into(),
                fs_slug: "nes".into(),
                rom_count: 2,
                name: "NES".into(),
                igdb_slug: None,
                moby_slug: None,
                hltb_slug: None,
                custom_name: None,
                igdb_id: None,
                sgdb_id: None,
                moby_id: None,
                launchbox_id: None,
                ss_id: None,
                ra_id: None,
                hasheous_id: None,
                tgdb_id: None,
                flashpoint_id: None,
                category: None,
                generation: None,
                family_name: None,
                family_slug: None,
                url: None,
                url_logo: None,
                firmware: vec![],
                aspect_ratio: None,
                created_at: "".into(),
                updated_at: "".into(),
                fs_size_bytes: 0,
                is_unidentified: false,
                is_identified: true,
                missing_from_fs: false,
                display_name: Some("Nintendo Entertainment System".into()),
            }],
            collections: vec![Collection {
                id: 10,
                name: "Favorites".into(),
                collection_type: None,
                rom_count: Some(1),
                is_smart: false,
                is_virtual: false,
                virtual_id: None,
            }],
            warnings: vec![],
        }
    }

    #[test]
    fn save_and_load_round_trip() {
        let _env = TestEnv::new();
        let dir = std::env::temp_dir().join(format!(
            "romm-lib-snap-{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        std::fs::create_dir_all(&dir).unwrap();
        std::env::set_var("ROMM_TEST_LIBRARY_SNAPSHOT_DIR", &dir);

        let fetch = sample_fetch();
        save_snapshot(&fetch.platforms, &fetch.collections);

        let loaded = load_snapshot().expect("snapshot should load");
        assert_eq!(loaded.platforms.len(), 1);
        assert_eq!(loaded.collections.len(), 1);
        assert_eq!(loaded.platforms[0].id, 1);
        assert_eq!(loaded.collections[0].id, 10);

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn corrupt_file_returns_none() {
        let _env = TestEnv::new();
        let dir = std::env::temp_dir().join(format!(
            "romm-lib-snap-bad-{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        std::fs::create_dir_all(&dir).unwrap();
        std::env::set_var("ROMM_TEST_LIBRARY_SNAPSHOT_DIR", &dir);
        let path = dir.join(DEFAULT_FILE);
        std::fs::write(&path, b"not json {{{").unwrap();
        assert!(load_snapshot().is_none());
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn wrong_version_returns_none() {
        let _env = TestEnv::new();
        let dir = std::env::temp_dir().join(format!(
            "romm-lib-snap-ver-{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        std::fs::create_dir_all(&dir).unwrap();
        std::env::set_var("ROMM_TEST_LIBRARY_SNAPSHOT_DIR", &dir);
        let path = dir.join(DEFAULT_FILE);
        let bad = serde_json::json!({
            "version": 999,
            "saved_at_secs": 0,
            "platforms": [],
            "collections": []
        });
        std::fs::write(&path, bad.to_string()).unwrap();
        assert!(load_snapshot().is_none());
        let _ = std::fs::remove_dir_all(&dir);
    }
}
