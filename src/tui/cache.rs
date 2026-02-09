//! Persistent ROM cache — survives across program restarts.
//!
//! Stores `RomList` per platform/collection on disk as JSON. On load, entries
//! are validated against the live `rom_count` from the API; stale entries are
//! silently discarded so only changed platforms trigger a re-fetch.

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::types::RomList;

/// Default cache file path (next to the binary / CWD).
const DEFAULT_CACHE_FILE: &str = "romm-cache.json";

// ---------------------------------------------------------------------------
// Cache key
// ---------------------------------------------------------------------------

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub enum RomCacheKey {
    Platform(u64),
    Collection(u64),
}

// ---------------------------------------------------------------------------
// On-disk format
// ---------------------------------------------------------------------------

#[derive(Serialize, Deserialize)]
struct CacheFile {
    version: u32,
    entries: Vec<CacheEntry>,
}

#[derive(Serialize, Deserialize)]
struct CacheEntry {
    key: RomCacheKey,
    /// The `platform.rom_count` (or `collection.rom_count`) at the time we
    /// cached this data.  On lookup we compare this against the *current*
    /// platform count, NOT against `data.total` (which can legitimately differ).
    expected_count: u64,
    data: RomList,
}

// ---------------------------------------------------------------------------
// RomCache
// ---------------------------------------------------------------------------

pub struct RomCache {
    entries: HashMap<RomCacheKey, (u64, RomList)>, // (expected_count, data)
    path: PathBuf,
}

impl RomCache {
    /// Load cache from disk (or start empty if the file is missing / corrupt).
    pub fn load() -> Self {
        let path = PathBuf::from(
            std::env::var("ROMM_CACHE_PATH").unwrap_or_else(|_| DEFAULT_CACHE_FILE.to_string()),
        );
        Self::load_from(path)
    }

    fn load_from(path: PathBuf) -> Self {
        let entries = Self::read_file(&path).unwrap_or_default();
        Self { entries, path }
    }

    fn read_file(path: &Path) -> Option<HashMap<RomCacheKey, (u64, RomList)>> {
        let data = std::fs::read_to_string(path).ok()?;
        let file: CacheFile = serde_json::from_str(&data).ok()?;
        if file.version != 1 {
            return None;
        }
        let map = file
            .entries
            .into_iter()
            .map(|e| (e.key, (e.expected_count, e.data)))
            .collect();
        Some(map)
    }

    /// Persist current cache to disk (best-effort; errors are silently ignored).
    pub fn save(&self) {
        let file = CacheFile {
            version: 1,
            entries: self
                .entries
                .iter()
                .map(|(k, (ec, v))| CacheEntry {
                    key: *k,
                    expected_count: *ec,
                    data: v.clone(),
                })
                .collect(),
        };
        if let Ok(json) = serde_json::to_string(&file) {
            let _ = std::fs::write(&self.path, json);
        }
    }

    /// Return cached data **only** if the platform's `rom_count` hasn't changed
    /// since we cached it.  We compare the stored count (from the platforms
    /// endpoint at cache time) against the current count — NOT `RomList.total`,
    /// which can legitimately differ from `rom_count`.
    pub fn get_valid(&self, key: &RomCacheKey, expected_count: u64) -> Option<&RomList> {
        self.entries
            .get(key)
            .filter(|(stored_count, _)| *stored_count == expected_count)
            .map(|(_, list)| list)
    }

    /// Insert (or replace) an entry, then persist to disk.
    /// `expected_count` is the platform/collection `rom_count` at this moment.
    pub fn insert(&mut self, key: RomCacheKey, data: RomList, expected_count: u64) {
        self.entries.insert(key, (expected_count, data));
        self.save();
    }

}
