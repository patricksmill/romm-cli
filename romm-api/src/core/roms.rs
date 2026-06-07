//! Shared ROM list fetching helpers.

use anyhow::Result;

use crate::client::RommClient;
use crate::endpoints::roms::GetRoms;
use crate::types::RomList;

/// Maximum ROM rows fetched by paginated helpers (TUI safety cap).
pub const ROM_PAGE_CEILING: u64 = 20000;

/// Fetch all pages for a `GetRoms` request up to [`ROM_PAGE_CEILING`].
pub async fn fetch_roms_paginated(client: &RommClient, req: &GetRoms) -> Result<RomList> {
    let mut roms = client.call(req).await?;
    let total = roms.total;
    while (roms.items.len() as u64) < total && (roms.items.len() as u64) < ROM_PAGE_CEILING {
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
