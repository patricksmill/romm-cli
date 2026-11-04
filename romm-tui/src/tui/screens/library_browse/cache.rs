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
        if self.expected_rom_count_for_position(subsection, source_idx) == 0 {
            return None;
        }
        let mut request = GetRoms {
            limit: Some(50),
            ..Default::default()
        };
        match self.cache_key_for_position(subsection, source_idx)? {
            RomCacheKey::Platform(id) => request.platform_id = Some(id),
            RomCacheKey::Collection(id) => request.collection_id = Some(id),
            RomCacheKey::SmartCollection(id) => request.smart_collection_id = Some(id),
            RomCacheKey::VirtualCollection(id) => {
                // Cache identity defaults a missing ID to ""; the request must retain None.
                request.virtual_collection_id =
                    self.collections[source_idx].virtual_id.as_ref().map(|_| id);
            }
        }
        Some(request)
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
        self.selected_list_source_index()
            .and_then(|i| self.cache_key_for_position(self.subsection, i))
    }

    pub fn expected_rom_count(&self) -> u64 {
        self.selected_list_source_index()
            .map(|i| self.expected_rom_count_for_position(self.subsection, i))
            .unwrap_or(0)
    }

    /// Prefer the first visible list row that has a complete ROM cache entry.
    /// Leaves selection at index 0 when none match.
    pub fn prefer_first_complete_cache_hit(&mut self, cache: &romm_api::core::cache::RomCache) {
        let visible = self.visible_list_source_indices();
        for (list_idx, &source_idx) in visible.iter().enumerate() {
            if let Some(key) = self.cache_key_for_position(self.subsection, source_idx) {
                if cache.get_complete(&key).is_some() {
                    self.list_index = list_idx;
                    return;
                }
            }
        }
    }

    /// Current list selection as a persistable config value.
    pub fn selection_state(&self) -> Option<romm_api::config::LibrarySelectionState> {
        use romm_api::config::{LibrarySelectionKey, LibrarySelectionState, LibrarySubsectionKind};
        let key = match self.cache_key()? {
            RomCacheKey::Platform(id) => LibrarySelectionKey::Platform { id },
            RomCacheKey::Collection(id) => LibrarySelectionKey::Collection { id },
            RomCacheKey::SmartCollection(id) => LibrarySelectionKey::SmartCollection { id },
            RomCacheKey::VirtualCollection(id) => LibrarySelectionKey::VirtualCollection { id },
        };
        let subsection = match self.subsection {
            LibrarySubsection::ByConsole => LibrarySubsectionKind::Console,
            LibrarySubsection::ByCollection => LibrarySubsectionKind::Collection,
        };
        Some(LibrarySelectionState { subsection, key })
    }

    /// Restore a previously persisted selection when that row still exists.
    ///
    /// Returns `true` when the selection was applied.
    pub fn restore_selection(
        &mut self,
        saved: &Option<romm_api::config::LibrarySelectionState>,
    ) -> bool {
        use romm_api::config::{LibrarySelectionKey, LibrarySubsectionKind};
        let Some(saved) = saved.as_ref() else {
            return false;
        };
        let subsection = match saved.subsection {
            LibrarySubsectionKind::Console => LibrarySubsection::ByConsole,
            LibrarySubsectionKind::Collection => LibrarySubsection::ByCollection,
        };
        self.subsection = subsection;
        self.list_search.clear();
        self.view_mode = LibraryViewMode::List;
        self.roms = None;
        self.rom_loading = false;

        let source_idx = match (&saved.key, subsection) {
            (LibrarySelectionKey::Platform { id }, LibrarySubsection::ByConsole) => {
                self.platforms.iter().position(|p| p.id == *id)
            }
            (LibrarySelectionKey::Collection { id }, LibrarySubsection::ByCollection) => self
                .collections
                .iter()
                .position(|c| !c.is_virtual && !c.is_smart && c.id == *id),
            (LibrarySelectionKey::SmartCollection { id }, LibrarySubsection::ByCollection) => self
                .collections
                .iter()
                .position(|c| c.is_smart && !c.is_virtual && c.id == *id),
            (LibrarySelectionKey::VirtualCollection { id }, LibrarySubsection::ByCollection) => {
                self.collections
                    .iter()
                    .position(|c| c.is_virtual && c.virtual_id.as_deref() == Some(id.as_str()))
            }
            _ => None,
        };
        let Some(source_idx) = source_idx else {
            self.subsection = LibrarySubsection::ByConsole;
            self.list_index = 0;
            return false;
        };
        let visible = self.visible_list_source_indices();
        if let Some(list_idx) = visible.iter().position(|&s| s == source_idx) {
            self.list_index = list_idx;
            true
        } else {
            self.list_index = 0;
            false
        }
    }

    pub fn get_roms_request(&self) -> Option<GetRoms> {
        self.selected_list_source_index()
            .and_then(|i| self.get_roms_request_for_position(self.subsection, i))
    }
}
