//! ROM list fetch and collection prefetch scheduling.

use anyhow::Result;

use crate::client::RommClient;
use crate::endpoints::roms::GetRoms;
use crate::types::RomList;

use super::background::types::CollectionPrefetchDone;
use super::AppScreen;

#[inline]
pub(crate) fn primary_rom_load_result_is_current(done_gen: u64, current_gen: u64) -> bool {
    done_gen == current_gen
}

impl super::App {
    pub(in crate::tui::app) fn selected_rom_request_for_library(
        lib: &crate::tui::screens::library_browse::LibraryBrowseScreen,
    ) -> Option<GetRoms> {
        match lib.subsection {
            crate::tui::screens::library_browse::LibrarySubsection::ByConsole => {
                lib.get_roms_request_platform()
            }
            crate::tui::screens::library_browse::LibrarySubsection::ByCollection => {
                lib.get_roms_request_collection()
            }
        }
    }
    pub(in crate::tui::app) fn queue_collection_prefetches_from_screen(
        &mut self,
        radius: usize,
        _reason: &'static str,
    ) {
        let AppScreen::LibraryBrowse(ref lib) = self.screen else {
            return;
        };
        for (key, req, expected) in lib.collection_prefetch_candidates(radius) {
            if self.rom_cache.get_valid(&key, expected).is_some() {
                continue;
            }
            if self.collection_prefetch_queued_keys.contains(&key)
                || self.collection_prefetch_inflight_keys.contains(&key)
            {
                continue;
            }
            self.collection_prefetch_queued_keys.insert(key.clone());
            self.collection_prefetch_queue
                .push_back((key, req, expected));
        }
    }

    pub(in crate::tui::app) fn drive_collection_prefetch_scheduler(&mut self) {
        const PREFETCH_MAX_INFLIGHT: usize = 2;
        while self.collection_prefetch_inflight_keys.len() < PREFETCH_MAX_INFLIGHT {
            let Some((key, req, expected)) = self.collection_prefetch_queue.pop_back() else {
                break;
            };
            self.collection_prefetch_queued_keys.remove(&key);
            self.collection_prefetch_inflight_keys.insert(key.clone());
            let tx = self.collection_prefetch_tx.clone();
            let client = self.client.clone();
            tokio::spawn(async move {
                let result = Self::fetch_roms_full(client, req).await;
                let (roms, warning) = match result {
                    Ok(list) => (Some(list), None),
                    Err(e) => (None, Some(format!("Collection prefetch failed: {e:#}"))),
                };
                let _ = tx.send(CollectionPrefetchDone {
                    key,
                    expected,
                    roms,
                    warning,
                });
            });
        }
    }
    pub(in crate::tui::app) async fn fetch_roms_full(
        client: RommClient,
        req: GetRoms,
    ) -> Result<RomList> {
        let mut roms = client.call(&req).await?;
        let total = roms.total;
        let ceiling = 20000;
        while (roms.items.len() as u64) < total && (roms.items.len() as u64) < ceiling {
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
}
