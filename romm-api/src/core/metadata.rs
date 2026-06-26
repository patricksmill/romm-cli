//! ROM metadata search, match, and cache helpers (CLI + TUI).

use crate::client::RommClient;
use crate::core::cache::{RomCache, RomCacheKey};
use crate::endpoints::roms::{GetSearchCover, GetSearchRoms};
use crate::error::ApiError;
use crate::types::metadata::{SearchCover, SearchRom};

/// Drop cached ROM list rows for one platform after metadata edit.
pub fn invalidate_platform_rom_cache(platform_id: u64) {
    let mut c = RomCache::load();
    c.remove(&RomCacheKey::Platform(platform_id));
}

/// `GET /api/search/roms` — provider search for manual match.
pub async fn search_metadata_matches(
    client: &RommClient,
    rom_id: u64,
    search_term: Option<String>,
    search_by: Option<String>,
) -> Result<Vec<SearchRom>, ApiError> {
    client
        .call(&GetSearchRoms {
            rom_id,
            search_term,
            search_by,
        })
        .await
}

/// `GET /api/search/cover` — SteamGridDB cover search (optional after match).
pub async fn search_covers(
    client: &RommClient,
    search_term: &str,
) -> Result<Vec<SearchCover>, ApiError> {
    client
        .call(&GetSearchCover {
            search_term: search_term.to_string(),
        })
        .await
}
