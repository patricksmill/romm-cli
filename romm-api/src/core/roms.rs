//! Shared ROM list fetching helpers.

use anyhow::Result;

use crate::client::RommClient;
use crate::endpoints::roms::GetRoms;
use crate::types::RomList;

/// Maximum ROM rows fetched by paginated helpers (TUI safety cap).
pub const ROM_PAGE_CEILING: u64 = 20000;

/// True when a paginated ROM list has enough rows to stop fetching
/// (`items.len() >= total`, or the safety ceiling was hit).
pub fn rom_list_fetch_complete(list: &RomList) -> bool {
    let loaded = list.items.len() as u64;
    rom_list_cache_complete(list) || loaded >= ROM_PAGE_CEILING
}

/// True when a ROM list contains every row advertised by the API.
pub fn rom_list_cache_complete(list: &RomList) -> bool {
    list.items.len() as u64 >= list.total
}

/// Fetch all pages for a `GetRoms` request up to [`ROM_PAGE_CEILING`].
pub async fn fetch_roms_paginated(client: &RommClient, req: &GetRoms) -> Result<RomList> {
    let mut roms = client.call(req).await?;
    while !rom_list_fetch_complete(&roms) {
        let mut next_req = req.clone();
        next_req.offset = Some(roms.items.len() as u32);
        let next_batch = client.call(&next_req).await?;
        if next_batch.items.is_empty() {
            break;
        }
        roms.items.extend(next_batch.items);
    }
    Ok(roms)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::Rom;

    fn minimal_rom(id: u64) -> Rom {
        Rom {
            id,
            platform_id: 10,
            platform_slug: None,
            platform_fs_slug: None,
            platform_custom_name: None,
            platform_display_name: None,
            fs_name: format!("game{id}.zip"),
            fs_name_no_tags: format!("game{id}"),
            fs_name_no_ext: format!("game{id}"),
            fs_extension: "zip".to_string(),
            fs_path: format!("/roms/game{id}.zip"),
            fs_size_bytes: 1,
            name: format!("Game {id}"),
            slug: None,
            summary: None,
            path_cover_small: None,
            path_cover_large: None,
            url_cover: None,
            has_manual: false,
            path_manual: None,
            url_manual: None,
            is_unidentified: false,
            is_identified: true,
            files: Vec::new(),
            ra_id: None,
            merged_ra_metadata: None,
        }
    }

    fn list(items: usize, total: u64) -> RomList {
        RomList {
            items: (0..items).map(|i| minimal_rom(i as u64)).collect(),
            total,
            limit: 50,
            offset: 0,
        }
    }

    #[test]
    fn complete_when_items_cover_total() {
        assert!(rom_list_fetch_complete(&list(3, 3)));
    }

    #[test]
    fn incomplete_when_short_of_total() {
        assert!(!rom_list_fetch_complete(&list(1, 100)));
    }
}
