//! ROM metadata search, match, and cache helpers (CLI + TUI).

use crate::client::RommClient;
use crate::core::cache::RomCache;
use crate::endpoints::roms::{GetSearchCover, GetSearchRoms, RomUpdateFields};
use crate::error::ApiError;
use crate::types::metadata::{SearchCover, SearchRom};

/// Drop cached ROM list rows that may contain stale data after a metadata edit.
pub fn invalidate_platform_rom_cache(platform_id: u64) {
    let mut c = RomCache::load();
    c.remove_metadata_dependent_entries(platform_id);
}

/// Multipart fields for applying a search row (mirrors RomM web manual match).
pub fn search_row_apply_fields(row: &SearchRom) -> RomUpdateFields {
    RomUpdateFields {
        name: Some(row.name.clone()),
        summary: row.summary.clone().filter(|s| !s.is_empty()),
        url_cover: row.best_url_cover(),
        match_fields: row.primary_match_fields(),
    }
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
