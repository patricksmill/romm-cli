use crate::core::utils::{self, RomGroup};
use crate::tui::text_search::{
    filter_source_indices, jump_next_index, normalize_label, LibrarySearchMode, SearchState,
};
use crate::types::{Collection, Platform, Rom, RomList};

use super::types::{
    LibraryBrowseScreen, LibrarySubsection, LibraryViewMode, UploadPrompt,
};

impl LibraryBrowseScreen {
    pub fn new(platforms: Vec<Platform>, collections: Vec<Collection>) -> Self {
        Self {
            platforms,
            collections,
            subsection: LibrarySubsection::ByConsole,
            list_index: 0,
            view_mode: LibraryViewMode::List,
            roms: None,
            rom_groups: None,
            rom_selected: 0,
            scroll_offset: 0,
            visible_rows: 20,
            list_search: SearchState::new(),
            rom_search: SearchState::new(),
            metadata_footer: None,
            metadata_footer_clear_at: None,
            rom_loading: false,
            upload_prompt: None,
        }
    }

    pub fn set_metadata_footer(&mut self, msg: Option<String>) {
        self.metadata_footer = msg;
        self.metadata_footer_clear_at = None;
    }

    pub fn set_temporary_metadata_footer(&mut self, msg: String, duration: std::time::Duration) {
        self.metadata_footer = Some(msg);
        self.metadata_footer_clear_at = Some(std::time::Instant::now() + duration);
    }

    pub fn poll_footer_clear(&mut self) {
        if let Some(clear_at) = self.metadata_footer_clear_at {
            if std::time::Instant::now() >= clear_at {
                self.metadata_footer = None;
                self.metadata_footer_clear_at = None;
            }
        }
    }

    /// `Ctrl+u` upload path modal is open.
    pub fn any_upload_prompt_open(&self) -> bool {
        self.upload_prompt.is_some()
    }

    pub fn open_upload_prompt(&mut self) {
        self.upload_prompt = Some(UploadPrompt::default());
    }

    pub fn close_upload_prompt(&mut self) {
        self.upload_prompt = None;
    }

    /// True while either pane has the search typing bar open (blocks global shortcuts).
    pub fn any_search_bar_open(&self) -> bool {
        self.list_search.mode.is_some() || self.rom_search.mode.is_some()
    }

    /// Display strings for each row (same text users see, without selection prefix).
    pub(crate) fn list_row_labels(&self) -> Vec<String> {
        match self.subsection {
            LibrarySubsection::ByConsole => self
                .platforms
                .iter()
                .map(|p| {
                    let name = p.display_name.as_deref().unwrap_or(&p.name);
                    format!("{} ({} roms)", name, p.rom_count)
                })
                .collect(),
            LibrarySubsection::ByCollection => self
                .collections
                .iter()
                .map(|c| {
                    let title = if c.is_virtual {
                        format!("{} [auto]", c.name)
                    } else if c.is_smart {
                        format!("{} [smart]", c.name)
                    } else {
                        c.name.clone()
                    };
                    format!("{} ({} roms)", title, c.rom_count.unwrap_or(0))
                })
                .collect(),
        }
    }

    pub(crate) fn visible_list_source_indices(&self) -> Vec<usize> {
        let labels = self.list_row_labels();
        if self.list_search.filter_active() {
            filter_source_indices(&labels, &self.list_search.normalized_query)
        } else {
            (0..labels.len()).collect()
        }
    }

    pub(crate) fn clamp_list_index(&mut self) {
        let v = self.visible_list_source_indices();
        if v.is_empty() || self.list_index >= v.len() {
            self.list_index = 0;
        }
    }

    /// Source index into `platforms` / `collections` for the current list selection.
    pub(crate) fn selected_list_source_index(&self) -> Option<usize> {
        let v = self.visible_list_source_indices();
        v.get(self.list_index).copied()
    }

    pub fn list_len(&self) -> usize {
        self.visible_list_source_indices().len()
    }

    pub fn list_next(&mut self) {
        let len = self.list_len();
        if len > 0 {
            self.list_index = (self.list_index + 1) % len;
        }
    }

    pub fn list_previous(&mut self) {
        let len = self.list_len();
        if len > 0 {
            self.list_index = if self.list_index == 0 {
                len - 1
            } else {
                self.list_index - 1
            };
        }
    }

    pub fn rom_next(&mut self) {
        let groups = self.visible_rom_groups();
        let len = groups.len();
        if len > 0 {
            self.rom_selected = (self.rom_selected + 1) % len;
            self.update_rom_scroll(self.visible_rows);
        }
    }

    pub fn rom_previous(&mut self) {
        let groups = self.visible_rom_groups();
        let len = groups.len();
        if len > 0 {
            self.rom_selected = if self.rom_selected == 0 {
                len - 1
            } else {
                self.rom_selected - 1
            };
            self.update_rom_scroll(self.visible_rows);
        }
    }

    fn update_rom_scroll(&mut self, visible: usize) {
        if self.rom_groups.is_none() {
            return;
        }
        let list_len = self.visible_rom_groups().len();
        self.update_rom_scroll_with_len(list_len, visible);
    }

    pub(crate) fn update_rom_scroll_with_len(&mut self, list_len: usize, visible: usize) {
        let visible = visible.max(1);
        let max_scroll = list_len.saturating_sub(visible);
        if self.rom_selected >= self.scroll_offset + visible {
            self.scroll_offset = (self.rom_selected + 1).saturating_sub(visible);
        } else if self.rom_selected < self.scroll_offset {
            self.scroll_offset = self.rom_selected;
        }
        self.scroll_offset = self.scroll_offset.min(max_scroll);
    }

    pub fn switch_subsection(&mut self) {
        self.subsection = match self.subsection {
            LibrarySubsection::ByConsole => LibrarySubsection::ByCollection,
            LibrarySubsection::ByCollection => LibrarySubsection::ByConsole,
        };
        self.list_index = 0;
        self.roms = None;
        self.rom_loading = false;
        self.view_mode = LibraryViewMode::List;
        self.list_search.clear();
    }

    pub fn switch_view(&mut self) {
        match self.view_mode {
            LibraryViewMode::List => {
                self.list_search.clear();
                self.view_mode = LibraryViewMode::Roms;
            }
            LibraryViewMode::Roms => {
                self.rom_search.clear();
                self.view_mode = LibraryViewMode::List;
            }
        }
        self.rom_selected = 0;
        self.scroll_offset = 0;
    }

    pub fn back_to_list(&mut self) {
        self.rom_search.clear();
        self.view_mode = LibraryViewMode::List;
    }

    pub fn clear_roms(&mut self) {
        self.roms = None;
        self.rom_groups = None;
        self.rom_selected = 0;
        self.scroll_offset = 0;
        self.rom_search.clear();
    }

    pub fn set_rom_loading(&mut self, loading: bool) {
        self.rom_loading = loading;
    }

    pub fn set_roms(&mut self, roms: RomList) {
        self.roms = Some(roms.clone());
        self.rom_groups = Some(utils::group_roms_by_name(&roms.items));
        self.rom_loading = false;
    }

    // -- List search --------------------------------------------------------

    pub fn enter_list_search(&mut self, mode: LibrarySearchMode) {
        self.list_search.enter(mode);
        self.list_index = 0;
    }

    pub fn clear_list_search(&mut self) {
        self.list_search.clear();
        self.clamp_list_index();
    }

    pub fn add_list_search_char(&mut self, c: char) {
        self.list_search.add_char(c);
        if self.list_search.mode == Some(LibrarySearchMode::Filter) {
            self.list_index = 0;
        } else if self.list_search.mode == Some(LibrarySearchMode::Jump) {
            self.list_jump_match(false);
        }
        self.clamp_list_index();
    }

    pub fn delete_list_search_char(&mut self) {
        self.list_search.delete_char();
        if self.list_search.mode == Some(LibrarySearchMode::Filter) {
            self.list_index = 0;
        }
        self.clamp_list_index();
    }

    pub fn commit_list_filter_bar(&mut self) {
        self.list_search.commit_filter_bar();
        self.clamp_list_index();
    }

    pub fn commit_rom_filter_bar(&mut self) {
        self.rom_search.commit_filter_bar();
    }

    pub fn list_jump_match(&mut self, next: bool) {
        if self.list_search.normalized_query.is_empty() {
            return;
        }
        let labels = self.list_row_labels();
        if labels.is_empty() {
            return;
        }
        let source = self
            .selected_list_source_index()
            .unwrap_or(0)
            .min(labels.len().saturating_sub(1));
        if let Some(new_src) =
            jump_next_index(&labels, source, &self.list_search.normalized_query, next)
        {
            let visible = self.visible_list_source_indices();
            if let Some(pos) = visible.iter().position(|&i| i == new_src) {
                self.list_index = pos;
            }
        }
    }

    // -- ROM search ---------------------------------------------------------

    pub fn enter_rom_search(&mut self, mode: LibrarySearchMode) {
        self.rom_search.enter(mode);
        self.rom_selected = 0;
        self.scroll_offset = 0;
    }

    pub fn clear_rom_search(&mut self) {
        self.rom_search.clear();
    }

    pub fn add_rom_search_char(&mut self, c: char) {
        self.rom_search.add_char(c);
        if self.rom_search.mode == Some(LibrarySearchMode::Filter) {
            self.rom_selected = 0;
            self.scroll_offset = 0;
        } else if self.rom_search.mode == Some(LibrarySearchMode::Jump) {
            self.jump_rom_match(false);
        }
    }

    pub fn delete_rom_search_char(&mut self) {
        self.rom_search.delete_char();
        if self.rom_search.mode == Some(LibrarySearchMode::Filter) {
            self.rom_selected = 0;
            self.scroll_offset = 0;
        }
    }

    pub fn jump_rom_match(&mut self, next: bool) {
        if self.rom_search.normalized_query.is_empty() {
            return;
        }
        let Some(ref groups) = self.rom_groups else {
            return;
        };
        let labels: Vec<String> = groups.iter().map(|g| g.name.clone()).collect();
        if labels.is_empty() {
            return;
        }
        let source = self.rom_selected.min(labels.len().saturating_sub(1));
        if let Some(idx) = jump_next_index(&labels, source, &self.rom_search.normalized_query, next)
        {
            self.rom_selected = idx;
            self.update_rom_scroll(self.visible_rows);
        }
    }

    pub fn get_selected_group(&self) -> Option<(Rom, Vec<Rom>)> {
        let visible = self.visible_rom_groups();
        if visible.is_empty() {
            return None;
        }
        let idx = if self.rom_selected >= visible.len() {
            0
        } else {
            self.rom_selected
        };
        visible
            .get(idx)
            .map(|g| (g.primary.clone(), g.others.clone()))
    }

    pub(crate) fn visible_rom_groups(&self) -> Vec<RomGroup> {
        let Some(ref groups) = self.rom_groups else {
            return Vec::new();
        };
        if self.rom_search.filter_active() {
            groups
                .iter()
                .filter(|g| normalize_label(&g.name).contains(&self.rom_search.normalized_query))
                .cloned()
                .collect()
        } else {
            groups.clone()
        }
    }

    pub(crate) fn list_title(&self) -> &str {
        match self.subsection {
            LibrarySubsection::ByConsole => "Consoles",
            LibrarySubsection::ByCollection => "Collections",
        }
    }

    pub fn selected_platform_id(&self) -> Option<u64> {
        match self.subsection {
            LibrarySubsection::ByConsole => self
                .selected_list_source_index()
                .and_then(|i| self.platforms.get(i).map(|p| p.id)),
            LibrarySubsection::ByCollection => None,
        }
    }

}
