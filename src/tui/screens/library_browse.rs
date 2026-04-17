use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::{Color, Style};
use ratatui::widgets::{Block, Borders, Cell, List, ListItem, ListState, Row, Table};
use ratatui::Frame;

use crate::core::cache::RomCacheKey;
use crate::core::utils::{self, RomGroup};
use crate::endpoints::roms::GetRoms;
use crate::tui::text_search::{filter_source_indices, jump_next_index, normalize_label, SearchState};
use crate::types::{Collection, Platform, Rom, RomList};

pub use crate::tui::text_search::LibrarySearchMode;

/// Which high-level grouping is currently shown in the left pane.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum LibrarySubsection {
    ByConsole,
    ByCollection,
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
    /// Filter/jump for the consoles/collections list (left pane).
    pub list_search: SearchState,
    /// Filter/jump for the games table (right pane).
    pub rom_search: SearchState,
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
            visible_rows: 20,
            list_search: SearchState::new(),
            rom_search: SearchState::new(),
        }
    }

    /// True while either pane has the search typing bar open (blocks global shortcuts).
    pub fn any_search_bar_open(&self) -> bool {
        self.list_search.mode.is_some() || self.rom_search.mode.is_some()
    }

    /// Display strings for each row (same text users see, without selection prefix).
    fn list_row_labels(&self) -> Vec<String> {
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

    fn visible_list_source_indices(&self) -> Vec<usize> {
        let labels = self.list_row_labels();
        if self.list_search.filter_active() {
            filter_source_indices(&labels, &self.list_search.normalized_query)
        } else {
            (0..labels.len()).collect()
        }
    }

    fn clamp_list_index(&mut self) {
        let v = self.visible_list_source_indices();
        if v.is_empty() {
            self.list_index = 0;
        } else if self.list_index >= v.len() {
            self.list_index = 0;
        }
    }

    /// Source index into `platforms` / `collections` for the current list selection.
    fn selected_list_source_index(&self) -> Option<usize> {
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
        self.clear_roms();
    }

    pub fn clear_roms(&mut self) {
        self.roms = None;
        self.rom_groups = None;
    }

    pub fn set_roms(&mut self, roms: RomList) {
        self.roms = Some(roms.clone());
        self.rom_groups = Some(utils::group_roms_by_name(&roms.items));
        self.rom_selected = 0;
        self.scroll_offset = 0;
        self.rom_search.clear();
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
        if let Some(idx) =
            jump_next_index(&labels, source, &self.rom_search.normalized_query, next)
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

    fn visible_rom_groups(&self) -> Vec<RomGroup> {
        let Some(ref groups) = self.rom_groups else {
            return Vec::new();
        };
        if self.rom_search.filter_active() {
            groups
                .iter()
                .filter(|g| {
                    normalize_label(&g.name).contains(&self.rom_search.normalized_query)
                })
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
            LibrarySubsection::ByConsole => self
                .selected_list_source_index()
                .and_then(|i| self.platforms.get(i).map(|p| p.id)),
            LibrarySubsection::ByCollection => None,
        }
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
        let count = self.expected_rom_count().min(20000);
        self.selected_platform_id().map(|id| GetRoms {
            platform_id: Some(id),
            limit: Some(count as u32),
            ..Default::default()
        })
    }

    pub fn get_roms_request_collection(&self) -> Option<GetRoms> {
        if self.subsection != LibrarySubsection::ByCollection {
            return None;
        }
        let count = self.expected_rom_count().min(20000);
        self.selected_list_source_index()
            .and_then(|i| self.collections.get(i))
            .map(|c| {
                if c.is_virtual {
                    GetRoms {
                        virtual_collection_id: c.virtual_id.clone(),
                        limit: Some(count as u32),
                        ..Default::default()
                    }
                } else if c.is_smart {
                    GetRoms {
                        smart_collection_id: Some(c.id),
                        limit: Some(count as u32),
                        ..Default::default()
                    }
                } else {
                    GetRoms {
                        collection_id: Some(c.id),
                        limit: Some(count as u32),
                        ..Default::default()
                    }
                }
            })
    }

    pub fn render(&mut self, f: &mut Frame, area: Rect) {
        let chunks = Layout::default()
            .constraints([Constraint::Percentage(30), Constraint::Percentage(70)])
            .direction(ratatui::layout::Direction::Horizontal)
            .split(area);

        let left_area = chunks[0];
        if self.list_search.mode.is_some() {
            let left_chunks = Layout::default()
                .constraints([Constraint::Length(3), Constraint::Min(3)])
                .direction(ratatui::layout::Direction::Vertical)
                .split(left_area);
            if let Some(mode) = self.list_search.mode {
                let title = match mode {
                    LibrarySearchMode::Filter => "Filter Search (list)",
                    LibrarySearchMode::Jump => "Jump Search (list, Tab next)",
                };
                let p = ratatui::widgets::Paragraph::new(format!("Search: {}", self.list_search.query))
                    .block(Block::default().title(title).borders(Borders::ALL));
                f.render_widget(p, left_chunks[0]);
            }
            self.render_list(f, left_chunks[1]);
        } else {
            self.render_list(f, left_area);
        }

        let right_chunks = if self.rom_search.mode.is_some() {
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

        if let Some(mode) = self.rom_search.mode {
            let title = match mode {
                LibrarySearchMode::Filter => "Filter Search",
                LibrarySearchMode::Jump => "Jump Search (Tab to next)",
            };
            let p = ratatui::widgets::Paragraph::new(format!("Search: {}", self.rom_search.query))
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
        let visible = self.visible_list_source_indices();
        let labels = self.list_row_labels();

        let items: Vec<ListItem> = visible
            .iter()
            .enumerate()
            .map(|(pos, &source_idx)| {
                let line = labels
                    .get(source_idx)
                    .cloned()
                    .unwrap_or_else(|| "?".to_string());
                let prefix =
                    if pos == self.list_index && self.view_mode == LibraryViewMode::List {
                        "▶ "
                    } else {
                        "  "
                    };
                ListItem::new(format!("{}{}", prefix, line))
            })
            .collect();

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
            let msg = if self.rom_search.mode.is_some() {
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
        let title = if self.rom_search.filter_browsing && !self.rom_search.query.is_empty() {
            format!(
                "Games (filtered: \"{}\") — {} — {} files",
                self.rom_search.query,
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
            LibraryViewMode::List => {
                if self.list_search.mode.is_some() {
                    "Type filter | Enter: browse matches | Esc: clear"
                } else if self.list_search.filter_browsing {
                    "↑↓: Navigate | Enter: Load games | Esc: clear filter"
                } else {
                    "t: Switch | ↑↓: Select | / f: Filter/Jump list | Enter: Games | Esc: Menu"
                }
            }
            LibraryViewMode::Roms => {
                if self.rom_search.mode.is_some() {
                    "Type filter | Enter: browse matches | Esc: clear filter"
                } else if self.rom_search.filter_browsing {
                    "←: Back to list | ↑↓: Navigate | Enter: Game detail | Esc: clear filter"
                } else {
                    "←: Back to list | ↑↓: Navigate | / f: Filter/Jump games | Enter: Game detail | Esc: Back"
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
    fn get_selected_group_clamps_stale_index_after_filter() {
        let mut s = LibraryBrowseScreen::new(vec![], vec![]);
        let items = vec![
            rom(1, "alpha", "a.zip"),
            rom(2, "alphabet", "ab.zip"),
            rom(3, "beta", "b.zip"),
        ];
        s.rom_groups = Some(utils::group_roms_by_name(&items));
        s.view_mode = LibraryViewMode::Roms;
        s.enter_rom_search(LibrarySearchMode::Filter);
        for c in "alp".chars() {
            s.add_rom_search_char(c);
        }
        s.rom_search.mode = None;
        s.rom_search.filter_browsing = true;
        s.rom_selected = 99;
        let (primary, _) = s.get_selected_group().expect("clamped index should yield a group");
        assert_eq!(primary.name, "alpha");
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
        s.enter_rom_search(LibrarySearchMode::Filter);
        for c in "alp".chars() {
            s.add_rom_search_char(c);
        }
        s.rom_search.mode = None;
        s.rom_search.filter_browsing = true;
        assert_eq!(s.rom_selected, 0);
        s.rom_next();
        assert_eq!(s.rom_selected, 1);
        s.rom_next();
        assert_eq!(s.rom_selected, 0);
    }
}
