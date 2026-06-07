use romm_api::core::utils::{self, RomGroup};
use romm_api::types::{Collection, Platform, Rom, RomList};

use super::types::{
    LibraryBrowseScreen, LibrarySubsection, LibraryViewMode, UploadPrompt, LEFT_PANEL_PERCENT_MAX,
    LEFT_PANEL_PERCENT_MIN,
};

impl LibraryBrowseScreen {
    pub fn new(
        platforms: Vec<Platform>,
        collections: Vec<Collection>,
        left_panel_percent: u16,
    ) -> Self {
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
            metadata_footer: None,
            metadata_footer_clear_at: None,
            rom_loading: false,
            upload_prompt: None,
            left_panel_percent: left_panel_percent
                .clamp(LEFT_PANEL_PERCENT_MIN, LEFT_PANEL_PERCENT_MAX),
        }
    }

    pub fn adjust_left_panel_percent(&mut self, delta: i16) {
        let next = (self.left_panel_percent as i16 + delta)
            .clamp(LEFT_PANEL_PERCENT_MIN as i16, LEFT_PANEL_PERCENT_MAX as i16);
        self.left_panel_percent = next as u16;
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

    pub fn list_len(&self) -> usize {
        self.list_row_labels().len()
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
    }

    pub fn switch_view(&mut self) {
        self.view_mode = match self.view_mode {
            LibraryViewMode::List => LibraryViewMode::Roms,
            LibraryViewMode::Roms => LibraryViewMode::List,
        };
        self.rom_selected = 0;
        self.scroll_offset = 0;
    }

    pub fn back_to_list(&mut self) {
        self.view_mode = LibraryViewMode::List;
    }

    pub fn clear_roms(&mut self) {
        self.roms = None;
        self.rom_groups = None;
        self.rom_selected = 0;
        self.scroll_offset = 0;
    }

    pub fn set_rom_loading(&mut self, loading: bool) {
        self.rom_loading = loading;
    }

    pub fn set_roms(&mut self, roms: RomList) {
        self.roms = Some(roms.clone());
        self.rom_groups = Some(utils::group_roms_by_name(&roms.items));
        self.rom_loading = false;
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
        self.rom_groups.clone().unwrap_or_default()
    }

    pub(crate) fn list_title(&self) -> &str {
        match self.subsection {
            LibrarySubsection::ByConsole => "Consoles",
            LibrarySubsection::ByCollection => "Collections",
        }
    }

    pub fn selected_platform_id(&self) -> Option<u64> {
        match self.subsection {
            LibrarySubsection::ByConsole => self.platforms.get(self.list_index).map(|p| p.id),
            LibrarySubsection::ByCollection => None,
        }
    }

    pub(crate) fn selected_list_source_index(&self) -> Option<usize> {
        if self.list_len() == 0 {
            None
        } else {
            Some(self.list_index.min(self.list_len().saturating_sub(1)))
        }
    }
}
