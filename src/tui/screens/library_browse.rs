use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::{Color, Style};
use ratatui::widgets::{Block, Borders, Cell, List, ListItem, ListState, Row, Table};
use ratatui::Frame;

use crate::core::cache::RomCacheKey;
use crate::core::utils::{self, RomGroup};
use crate::endpoints::roms::GetRoms;
use crate::types::{Collection, Platform, Rom, RomList};

/// Which high-level grouping is currently shown in the left pane.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum LibrarySubsection {
    ByConsole,
    ByCollection,
}

/// Active search mode in the Games list.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum LibrarySearchMode {
    /// Filter results to only show matches.
    Filter,
    /// Jump to the next match in the full list.
    Jump,
}

/// Which side of the library view currently has focus.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum LibraryViewMode {
    /// Left panel: list of consoles or collections
    List,
    /// Right panel: list of ROMs for selected console/collection
    Roms,
}

/// Main library browser: consoles/collections on the left, games on the right.
pub struct LibraryBrowseScreen {
    pub platforms: Vec<Platform>,
    pub collections: Vec<Collection>,
    pub subsection: LibrarySubsection,
    pub list_index: usize,
    pub view_mode: LibraryViewMode,
    pub roms: Option<RomList>,
    /// One row per game name (base + updates/DLC grouped).
    pub rom_groups: Option<Vec<RomGroup>>,
    pub rom_selected: usize,
    pub scroll_offset: usize,
    /// Visible data rows in the ROM pane (updated at render time).
    visible_rows: usize,
    // Search state
    pub search_query: String,
    pub search_mode: Option<LibrarySearchMode>,
    /// After committing filter with Enter: keep query applied but hide the search bar for navigation.
    pub filter_browsing: bool,
    /// Normalized search query (de-accented, lowercase).
    normalized_query: String,
}

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
            visible_rows: 20, // reasonable default until first render
            search_query: String::new(),
            search_mode: None,
            filter_browsing: false,
            normalized_query: String::new(),
        }
    }

    pub fn list_len(&self) -> usize {
        match self.subsection {
            LibrarySubsection::ByConsole => self.platforms.len(),
            LibrarySubsection::ByCollection => self.collections.len(),
        }
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

    /// Keep `rom_selected` within the visible window.
    ///
    /// `visible` is the number of data rows that fit on screen (set at
    /// render time). This manual bookkeeping gives us fine-grained control
    /// over scrolling behavior without storing a separate viewport object.
    fn update_rom_scroll(&mut self, visible: usize) {
        if let Some(ref groups) = self.rom_groups {
            self.update_rom_scroll_with_len(groups.len(), visible);
        }
    }

    fn update_rom_scroll_with_len(&mut self, list_len: usize, visible: usize) {
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
        self.clear_search();
        self.view_mode = LibraryViewMode::List;
        self.clear_roms();
    }

    /// Clear the ROM list (and groups) so the right panel does not show
    /// another console/collection's games while loading the new selection.
    pub fn clear_roms(&mut self) {
        self.roms = None;
        self.rom_groups = None;
    }

    /// Select a console or collection and load its ROMs.
    pub fn set_roms(&mut self, roms: RomList) {
        self.roms = Some(roms.clone());
        self.rom_groups = Some(utils::group_roms_by_name(&roms.items));
        self.rom_selected = 0;
        self.scroll_offset = 0;
        self.clear_search(); // reset search when changing consoles
    }

    // -- Search logic -------------------------------------------------------

    pub fn enter_search(&mut self, mode: LibrarySearchMode) {
        self.search_mode = Some(mode);
        self.filter_browsing = false;
        self.search_query.clear();
        self.normalized_query.clear();
        self.rom_selected = 0;
        self.scroll_offset = 0;
    }

    pub fn clear_search(&mut self) {
        self.search_mode = None;
        self.filter_browsing = false;
        self.search_query.clear();
        self.normalized_query.clear();
    }

    pub fn add_search_char(&mut self, c: char) {
        self.search_query.push(c);
        self.normalized_query = self.normalize(&self.search_query);
        if self.search_mode == Some(LibrarySearchMode::Filter) {
            self.rom_selected = 0;
            self.scroll_offset = 0;
        } else if self.search_mode == Some(LibrarySearchMode::Jump) {
            self.jump_to_match(false);
        }
    }

    pub fn delete_search_char(&mut self) {
        self.search_query.pop();
        self.normalized_query = self.normalize(&self.search_query);
        if self.search_mode == Some(LibrarySearchMode::Filter) {
            self.rom_selected = 0;
            self.scroll_offset = 0;
        }
    }

    /// Helper to strip diacritics and convert to lowercase for searching.
    fn normalize(&self, s: &str) -> String {
        use unicode_normalization::UnicodeNormalization;
        s.nfd()
            .filter(|c| !unicode_normalization::char::is_combining_mark(*c))
            .collect::<String>()
            .to_lowercase()
    }

    pub fn jump_to_match(&mut self, next: bool) {
        if self.normalized_query.is_empty() {
            return;
        }
        let Some(ref groups) = self.rom_groups else {
            return;
        };
        let start_idx = if next {
            (self.rom_selected + 1) % groups.len()
        } else {
            self.rom_selected
        };

        for i in 0..groups.len() {
            let idx = (start_idx + i) % groups.len();
            if self
                .normalize(&groups[idx].name)
                .contains(&self.normalized_query)
            {
                self.rom_selected = idx;
                self.update_rom_scroll(self.visible_rows);
                return;
            }
        }
    }

    /// Primary ROM and other files (updates/DLC) for the selected game.
    pub fn get_selected_group(&self) -> Option<(Rom, Vec<Rom>)> {
        self.visible_rom_groups()
            .get(self.rom_selected)
            .map(|g| (g.primary.clone(), g.others.clone()))
    }

    /// Actual list shown in the right pane (optionally filtered).
    fn visible_rom_groups(&self) -> Vec<RomGroup> {
        let Some(ref groups) = self.rom_groups else {
            return Vec::new();
        };
        let filter_active = (self.search_mode == Some(LibrarySearchMode::Filter)
            || self.filter_browsing)
            && !self.normalized_query.is_empty();
        if filter_active {
            groups
                .iter()
                .filter(|g| self.normalize(&g.name).contains(&self.normalized_query))
                .cloned()
                .collect()
        } else {
            groups.clone()
        }
    }

    fn list_title(&self) -> &str {
        match self.subsection {
            LibrarySubsection::ByConsole => "Consoles",
            LibrarySubsection::ByCollection => "Collections",
        }
    }

    fn selected_platform_id(&self) -> Option<u64> {
        match self.subsection {
            LibrarySubsection::ByConsole => self.platforms.get(self.list_index).map(|p| p.id),
            LibrarySubsection::ByCollection => None,
        }
    }

    fn selected_collection_id(&self) -> Option<u64> {
        match self.subsection {
            LibrarySubsection::ByCollection => self.collections.get(self.list_index).map(|c| c.id),
            LibrarySubsection::ByConsole => None,
        }
    }

    /// Cache key for the currently selected console or collection.
    pub fn cache_key(&self) -> Option<RomCacheKey> {
        self.selected_platform_id()
            .map(RomCacheKey::Platform)
            .or_else(|| self.selected_collection_id().map(RomCacheKey::Collection))
    }

    /// Expected ROM count from the live platform/collection metadata.
    /// Used to validate whether the disk cache is still fresh.
    pub fn expected_rom_count(&self) -> u64 {
        match self.subsection {
            LibrarySubsection::ByConsole => self
                .platforms
                .get(self.list_index)
                .map(|p| p.rom_count)
                .unwrap_or(0),
            LibrarySubsection::ByCollection => self
                .collections
                .get(self.list_index)
                .and_then(|c| c.rom_count)
                .unwrap_or(0),
        }
    }

    pub fn get_roms_request_platform(&self) -> Option<GetRoms> {
        let count = self.expected_rom_count().min(20000);
        self.selected_platform_id().map(|id| GetRoms {
            platform_id: Some(id),
            limit: Some(count as u32),
            ..Default::default()
        })
    }

    pub fn get_roms_request_collection(&self) -> Option<GetRoms> {
        let count = self.expected_rom_count().min(20000);
        self.selected_collection_id().map(|id| GetRoms {
            collection_id: Some(id),
            limit: Some(count as u32),
            ..Default::default()
        })
    }

    pub fn render(&mut self, f: &mut Frame, area: Rect) {
        let chunks = Layout::default()
            .constraints([Constraint::Percentage(30), Constraint::Percentage(70)])
            .direction(ratatui::layout::Direction::Horizontal)
            .split(area);

        self.render_list(f, chunks[0]);

        let right_chunks = if self.search_mode.is_some() {
            Layout::default()
                .constraints([
                    Constraint::Length(3),
                    Constraint::Min(5),
                    Constraint::Length(3),
                ])
                .direction(ratatui::layout::Direction::Vertical)
                .split(chunks[1])
        } else {
            Layout::default()
                .constraints([Constraint::Min(5), Constraint::Length(3)])
                .direction(ratatui::layout::Direction::Vertical)
                .split(chunks[1])
        };

        if let Some(mode) = self.search_mode {
            let title = match mode {
                LibrarySearchMode::Filter => "Filter Search",
                LibrarySearchMode::Jump => "Jump Search (Tab to next)",
            };
            let p = ratatui::widgets::Paragraph::new(format!("Search: {}", self.search_query))
                .block(Block::default().title(title).borders(Borders::ALL));
            f.render_widget(p, right_chunks[0]);
            self.render_roms(f, right_chunks[1]);
            self.render_help(f, right_chunks[2]);
        } else {
            self.render_roms(f, right_chunks[0]);
            self.render_help(f, right_chunks[1]);
        }
    }

    fn render_list(&self, f: &mut Frame, area: Rect) {
        let items: Vec<ListItem> = match self.subsection {
            LibrarySubsection::ByConsole => self
                .platforms
                .iter()
                .enumerate()
                .map(|(idx, p)| {
                    let name = p.display_name.as_deref().unwrap_or(&p.name);
                    let count = p.rom_count;
                    let prefix =
                        if idx == self.list_index && self.view_mode == LibraryViewMode::List {
                            "▶ "
                        } else {
                            "  "
                        };
                    ListItem::new(format!("{}{} ({} roms)", prefix, name, count))
                })
                .collect(),
            LibrarySubsection::ByCollection => self
                .collections
                .iter()
                .enumerate()
                .map(|(idx, c)| {
                    let count = c.rom_count.unwrap_or(0);
                    let prefix =
                        if idx == self.list_index && self.view_mode == LibraryViewMode::List {
                            "▶ "
                        } else {
                            "  "
                        };
                    ListItem::new(format!("{}{} ({} roms)", prefix, c.name, count))
                })
                .collect(),
        };

        let list = List::new(items)
            .block(
                Block::default()
                    .title(self.list_title())
                    .borders(Borders::ALL),
            )
            .highlight_symbol(if self.view_mode == LibraryViewMode::List {
                ">> "
            } else {
                "   "
            });

        let mut state = ListState::default();
        if self.view_mode == LibraryViewMode::List {
            state.select(Some(self.list_index));
        }

        f.render_stateful_widget(list, area, &mut state);
    }

    fn render_roms(&mut self, f: &mut Frame, area: Rect) {
        let visible = (area.height as usize).saturating_sub(3).max(1);
        self.visible_rows = visible;

        let groups = self.visible_rom_groups();
        if groups.is_empty() {
            let msg = if self.search_mode.is_some() {
                "No games match your search".to_string()
            } else if self.roms.is_none() && self.expected_rom_count() > 0 {
                format!("Loading {} games... please wait", self.expected_rom_count())
            } else {
                "Select a console or collection and press Enter to load ROMs".to_string()
            };
            let p = ratatui::widgets::Paragraph::new(msg)
                .block(Block::default().title("Games").borders(Borders::ALL));
            f.render_widget(p, area);
            return;
        }

        // Keep selection in bounds if filtering just reduced the count.
        if self.rom_selected >= groups.len() {
            self.rom_selected = 0;
            self.scroll_offset = 0;
        }

        self.update_rom_scroll_with_len(groups.len(), visible);

        let start = self.scroll_offset.min(groups.len().saturating_sub(visible));
        let end = (start + visible).min(groups.len());
        let visible_groups = &groups[start..end];

        let header = Row::new(vec![
            Cell::from("Name").style(Style::default().fg(Color::Cyan))
        ]);
        let rows: Vec<Row> = visible_groups
            .iter()
            .enumerate()
            .map(|(i, g)| {
                let global_idx = start + i;
                let style = if global_idx == self.rom_selected {
                    Style::default().fg(Color::Yellow)
                } else {
                    Style::default()
                };
                Row::new(vec![Cell::from(g.name.as_str()).style(style)]).height(1)
            })
            .collect();

        let total_files = self.roms.as_ref().map(|r| r.items.len()).unwrap_or(0);
        let total_roms = self.roms.as_ref().map(|r| r.total).unwrap_or(0);
        let title = if self.filter_browsing && !self.search_query.is_empty() {
            format!(
                "Games (filtered: \"{}\") — {} — {} files",
                self.search_query,
                groups.len(),
                total_files
            )
        } else if total_roms > 0 && (groups.len() as u64) < total_roms {
            format!(
                "Games ({} of {}) — {} files",
                groups.len(),
                total_roms,
                total_files
            )
        } else {
            format!("Games ({}) — {} files", groups.len(), total_files)
        };
        let widths = [Constraint::Percentage(100)];
        let table = Table::new(rows, widths)
            .header(header)
            .block(Block::default().title(title).borders(Borders::ALL));

        f.render_widget(table, area);
    }

    fn render_help(&self, f: &mut Frame, area: Rect) {
        let help = match self.view_mode {
            LibraryViewMode::List => "t: Switch Console/Collection | ↑↓: Select (games load) | Enter: Focus games | Esc: Back",
            LibraryViewMode::Roms => {
                if self.search_mode.is_some() {
                    "Type filter | Enter: browse matches | Esc: clear filter"
                } else if self.filter_browsing {
                    "←: Back to list | ↑↓: Navigate | Enter: Game detail | Esc: clear filter"
                } else {
                    "←: Back to list | ↑↓: Navigate | Enter: Game detail | Esc: Back"
                }
            }
        };
        let p =
            ratatui::widgets::Paragraph::new(help).block(Block::default().borders(Borders::ALL));
        f.render_widget(p, area);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::utils;
    use crate::types::Rom;

    fn rom(id: u64, name: &str, fs_name: &str) -> Rom {
        Rom {
            id,
            platform_id: 1,
            platform_slug: None,
            platform_fs_slug: None,
            platform_custom_name: None,
            platform_display_name: None,
            fs_name: fs_name.to_string(),
            fs_name_no_tags: name.to_string(),
            fs_name_no_ext: name.to_string(),
            fs_extension: "zip".to_string(),
            fs_path: format!("/{id}.zip"),
            fs_size_bytes: 1,
            name: name.to_string(),
            slug: None,
            summary: None,
            path_cover_small: None,
            path_cover_large: None,
            url_cover: None,
            is_unidentified: false,
            is_identified: true,
        }
    }

    #[test]
    fn rom_next_wraps_within_filtered_list_when_filter_browsing() {
        let mut s = LibraryBrowseScreen::new(vec![], vec![]);
        let items = vec![
            rom(1, "alpha", "a.zip"),
            rom(2, "alphabet", "ab.zip"),
            rom(3, "beta", "b.zip"),
        ];
        s.rom_groups = Some(utils::group_roms_by_name(&items));
        s.view_mode = LibraryViewMode::Roms;
        s.enter_search(LibrarySearchMode::Filter);
        for c in "alp".chars() {
            s.add_search_char(c);
        }
        s.search_mode = None;
        s.filter_browsing = true;
        assert_eq!(s.rom_selected, 0);
        s.rom_next();
        assert_eq!(s.rom_selected, 1);
        s.rom_next();
        assert_eq!(s.rom_selected, 0);
    }
}
