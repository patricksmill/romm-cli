//! ROM list fetch and collection prefetch scheduling.

use std::time::Instant;

use crate::tui::screens::library_browse::LibraryBrowseScreen;
use romm_api::core::cache::RomCacheKey;
use romm_api::core::roms::{fetch_roms_paginated, rom_list_fetch_complete};
use romm_api::endpoints::roms::GetRoms;
use romm_api::log_redact::redact_anyhow_for_log;
use romm_api::types::RomList;

use super::background::types::CollectionPrefetchDone;
use super::AppScreen;

#[inline]
pub(crate) fn primary_rom_load_result_is_current(done_gen: u64, current_gen: u64) -> bool {
    done_gen == current_gen
}

/// True when a completed ROM load still matches the library pane selection.
pub(crate) fn primary_rom_load_result_matches_selection(
    lib: &LibraryBrowseScreen,
    key: &Option<RomCacheKey>,
) -> bool {
    lib.cache_key().as_ref() == key.as_ref()
}

impl super::App {
    pub(in crate::tui::app) fn matching_rom_partial(
        &mut self,
        key: &RomCacheKey,
        expected: u64,
    ) -> Option<RomList> {
        match self.rom_partials.get(key) {
            Some((stored, list)) if *stored == expected && !rom_list_fetch_complete(list) => {
                Some(list.clone())
            }
            Some(_) => {
                self.rom_partials.remove(key);
                None
            }
            None => None,
        }
    }

    /// Drop in-flight primary ROM fetches so stale batches cannot overwrite a new selection.
    pub(in crate::tui::app) fn invalidate_primary_rom_load(&mut self) {
        self.rom_load_gen = self.rom_load_gen.saturating_add(1);
        if let Some(task) = self.rom_load_task.take() {
            task.abort();
        }
    }

    pub(in crate::tui::app) fn queue_primary_rom_load(
        &mut self,
        key: Option<RomCacheKey>,
        req: Option<GetRoms>,
        expected: u64,
        context: &'static str,
    ) {
        self.invalidate_primary_rom_load();
        self.deferred_load_roms = Some((key, req, expected, context, Instant::now()));
    }

    pub(in crate::tui::app) fn cancel_primary_rom_load(&mut self) {
        self.invalidate_primary_rom_load();
        self.deferred_load_roms = None;
    }

    pub(in crate::tui::app) fn resume_library_rom_load_if_needed(&mut self, context: &'static str) {
        let pending = {
            let AppScreen::LibraryBrowse(ref mut lib) = self.screen else {
                return;
            };
            if !lib.rom_loading {
                return;
            }
            let expected = lib.expected_rom_count();
            if expected == 0 {
                lib.set_rom_loading(false);
                return;
            }
            (
                lib.cache_key(),
                Self::selected_rom_request_for_library(lib),
                expected,
            )
        };
        self.queue_primary_rom_load(pending.0, pending.1, pending.2, context);
    }

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
                let result = fetch_roms_paginated(&client, &req).await;
                let (roms, warning) = match result {
                    Ok(list) => (Some(list), None),
                    Err(e) => (
                        None,
                        Some(format!(
                            "Collection prefetch failed: {}",
                            redact_anyhow_for_log(&e)
                        )),
                    ),
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
}
