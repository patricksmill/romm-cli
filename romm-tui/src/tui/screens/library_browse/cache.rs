use romm_api::core::cache::RomCacheKey;
use romm_api::endpoints::roms::GetRoms;
use romm_api::types::{Collection, Platform};

use super::types::{LibraryBrowseScreen, LibrarySubsection, LibraryViewMode};

impl LibraryBrowseScreen {
    fn collection_key(c: &Collection) -> RomCacheKey {
        if c.is_virtual {
            RomCacheKey::VirtualCollection(c.virtual_id.clone().unwrap_or_default())
        } else if c.is_smart {
            RomCacheKey::SmartCollection(c.id)
        } else {
            RomCacheKey::Collection(c.id)
        }
    }

    fn cache_key_for_position(
        &self,
        subsection: LibrarySubsection,
        source_idx: usize,
    ) -> Option<RomCacheKey> {
        match subsection {
            LibrarySubsection::ByConsole => self
                .platforms
                .get(source_idx)
                .map(|p| RomCacheKey::Platform(p.id)),
            LibrarySubsection::ByCollection => {
                self.collections.get(source_idx).map(Self::collection_key)
            }
        }
    }

    fn expected_rom_count_for_position(
        &self,
        subsection: LibrarySubsection,
        source_idx: usize,
    ) -> u64 {
        match subsection {
            LibrarySubsection::ByConsole => self
                .platforms
                .get(source_idx)
                .map(|p| p.rom_count)
                .unwrap_or(0),
            LibrarySubsection::ByCollection => self
                .collections
                .get(source_idx)
                .and_then(|c| c.rom_count)
                .unwrap_or(0),
        }
    }

    fn get_roms_request_for_position(
        &self,
        subsection: LibrarySubsection,
        source_idx: usize,
    ) -> Option<GetRoms> {
        let count = self
            .expected_rom_count_for_position(subsection, source_idx)
            .min(20000);
        if count == 0 {
            return None;
        }
        match subsection {
            LibrarySubsection::ByConsole => self.platforms.get(source_idx).map(|p| GetRoms {
                platform_id: Some(p.id),
                limit: Some(50),
                ..Default::default()
            }),
            LibrarySubsection::ByCollection => self.collections.get(source_idx).map(|c| {
                if c.is_virtual {
                    GetRoms {
                        virtual_collection_id: c.virtual_id.clone(),
                        limit: Some(50),
                        ..Default::default()
                    }
                } else if c.is_smart {
                    GetRoms {
                        smart_collection_id: Some(c.id),
                        limit: Some(50),
                        ..Default::default()
                    }
                } else {
                    GetRoms {
                        collection_id: Some(c.id),
                        limit: Some(50),
                        ..Default::default()
                    }
                }
            }),
        }
    }

    /// Replace metadata while preserving subsection and selection when possible.
    ///
    /// Returns `true` when the selected row identity or expected count changed,
    /// which means ROM pane data should be reloaded.
    pub fn replace_metadata_preserving_selection(
        &mut self,
        platforms: Vec<Platform>,
        collections: Vec<Collection>,
        update_platforms: bool,
        update_collections: bool,
    ) -> bool {
        let subsection = self.subsection;
        let old_source = self.selected_list_source_index();
        let old_key = old_source.and_then(|i| self.cache_key_for_position(subsection, i));
        let old_expected = old_source
            .map(|i| self.expected_rom_count_for_position(subsection, i))
            .unwrap_or(0);

        if update_platforms {
            self.platforms = platforms;
        }
        if update_collections {
            self.collections = collections;
        }

        let new_source = old_key.as_ref().and_then(|k| match subsection {
            LibrarySubsection::ByConsole => self
                .platforms
                .iter()
                .position(|p| matches!(k, RomCacheKey::Platform(id) if *id == p.id)),
            LibrarySubsection::ByCollection => self.collections.iter().position(|c| {
                let ck = Self::collection_key(c);
                &ck == k
            }),
        });

        self.list_index = new_source.unwrap_or(0);
        if self.list_index >= self.list_len() {
            self.list_index = 0;
        }

        let new_source = self.selected_list_source_index();
        let new_key = new_source.and_then(|i| self.cache_key_for_position(subsection, i));
        let new_expected = new_source
            .map(|i| self.expected_rom_count_for_position(subsection, i))
            .unwrap_or(0);

        let changed = old_key != new_key || old_expected != new_expected;
        if changed {
            self.clear_roms();
            self.view_mode = LibraryViewMode::List;
            self.rom_selected = 0;
            self.scroll_offset = 0;
        }
        changed
    }

    /// Build near-neighbor collection prefetch candidates around current selection.
    pub fn collection_prefetch_candidates(
        &self,
        radius: usize,
    ) -> Vec<(RomCacheKey, GetRoms, u64)> {
        if self.subsection != LibrarySubsection::ByCollection {
            return Vec::new();
        }
        let len = self.collections.len();
        if len == 0 {
            return Vec::new();
        }
        let center = self.selected_list_source_index().unwrap_or(0).min(len - 1);
        let start = center.saturating_sub(radius);
        let end = (center + radius + 1).min(len);
        let mut out = Vec::new();
        for source_idx in start..end {
            if source_idx == center {
                continue;
            }
            if let (Some(key), Some(req)) = (
                self.cache_key_for_position(LibrarySubsection::ByCollection, source_idx),
                self.get_roms_request_for_position(LibrarySubsection::ByCollection, source_idx),
            ) {
                let expected = self
                    .expected_rom_count_for_position(LibrarySubsection::ByCollection, source_idx);
                out.push((key, req, expected));
            }
        }
        out
    }
    pub fn cache_key(&self) -> Option<RomCacheKey> {
        match self.subsection {
            LibrarySubsection::ByConsole => self.selected_platform_id().map(RomCacheKey::Platform),
            LibrarySubsection::ByCollection => self
                .selected_list_source_index()
                .and_then(|i| self.collections.get(i))
                .map(|c| {
                    if c.is_virtual {
                        RomCacheKey::VirtualCollection(c.virtual_id.clone().unwrap_or_default())
                    } else if c.is_smart {
                        RomCacheKey::SmartCollection(c.id)
                    } else {
                        RomCacheKey::Collection(c.id)
                    }
                }),
        }
    }

    pub fn expected_rom_count(&self) -> u64 {
        match self.subsection {
            LibrarySubsection::ByConsole => self
                .selected_list_source_index()
                .and_then(|i| self.platforms.get(i).map(|p| p.rom_count))
                .unwrap_or(0),
            LibrarySubsection::ByCollection => self
                .selected_list_source_index()
                .and_then(|i| self.collections.get(i))
                .and_then(|c| c.rom_count)
                .unwrap_or(0),
        }
    }

    pub fn get_roms_request_platform(&self) -> Option<GetRoms> {
        self.selected_list_source_index()
            .and_then(|i| self.get_roms_request_for_position(LibrarySubsection::ByConsole, i))
    }

    pub fn get_roms_request_collection(&self) -> Option<GetRoms> {
        if self.subsection != LibrarySubsection::ByCollection {
            return None;
        }
        self.selected_list_source_index()
            .and_then(|i| self.get_roms_request_for_position(LibrarySubsection::ByCollection, i))
    }
}
