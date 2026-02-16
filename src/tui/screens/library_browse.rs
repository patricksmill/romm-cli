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
        if let Some(ref groups) = self.rom_groups {
            if !groups.is_empty() {
                self.rom_selected = (self.rom_selected + 1) % groups.len();
                self.update_rom_scroll(self.visible_rows);
            }
        }
    }

    pub fn rom_previous(&mut self) {
        if let Some(ref groups) = self.rom_groups {
            if !groups.is_empty() {
                self.rom_selected = if self.rom_selected == 0 {
                    groups.len() - 1
                } else {
                    self.rom_selected - 1
                };
                self.update_rom_scroll(self.visible_rows);
            }
        }
    }

    /// Keep `rom_selected` within the visible window.
    ///
    /// `visible` is the number of data rows that fit on screen (set at
    /// render time). This manual bookkeeping gives us fine-grained control
    /// over scrolling behavior without storing a separate viewport object.
    fn update_rom_scroll(&mut self, visible: usize) {
        if let Some(ref groups) = self.rom_groups {
            let visible = visible.max(1);
            let max_scroll = groups.len().saturating_sub(visible);
            if self.rom_selected >= self.scroll_offset + visible {
                self.scroll_offset = (self.rom_selected + 1).saturating_sub(visible);
            } else if self.rom_selected < self.scroll_offset {
                self.scroll_offset = self.rom_selected;
            }
            self.scroll_offset = self.scroll_offset.min(max_scroll);
        }
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
        self.view_mode = LibraryViewMode::List;
        self.roms = None;
    }

    pub fn set_roms(&mut self, roms: RomList) {
        self.roms = Some(roms.clone());
        self.rom_groups = Some(utils::group_roms_by_name(&roms.items));
        self.rom_selected = 0;
        self.scroll_offset = 0;
    }

    /// Primary ROM and other files (updates/DLC) for the selected game.
    pub fn get_selected_group(&self) -> Option<(Rom, Vec<Rom>)> {
        self.rom_groups
            .as_ref()
            .and_then(|g| g.get(self.rom_selected))
            .map(|g| (g.primary.clone(), g.others.clone()))
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

    pub fn get_roms_request_platform_with_limit(&self, limit: u32) -> Option<GetRoms> {
        self.selected_platform_id().map(|id| GetRoms {
            platform_id: Some(id),
            limit: Some(limit),
            ..Default::default()
        })
    }

    pub fn get_roms_request_collection_with_limit(&self, limit: u32) -> Option<GetRoms> {
        self.selected_collection_id().map(|id| GetRoms {
            collection_id: Some(id),
            limit: Some(limit),
            ..Default::default()
        })
    }

    pub fn render(&mut self, f: &mut Frame, area: Rect) {
        let chunks = Layout::default()
            .constraints([Constraint::Percentage(30), Constraint::Percentage(70)])
            .direction(ratatui::layout::Direction::Horizontal)
            .split(area);

        self.render_list(f, chunks[0]);

        let right_chunks = Layout::default()
            .constraints([Constraint::Min(5), Constraint::Length(3)])
            .direction(ratatui::layout::Direction::Vertical)
            .split(chunks[1]);

        self.render_roms(f, right_chunks[0]);
        self.render_help(f, right_chunks[1]);
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
        if self.rom_groups.is_none() {
            let msg = "Select a console or collection and press Enter to load ROMs";
            let p = ratatui::widgets::Paragraph::new(msg)
                .block(Block::default().title("ROMs").borders(Borders::ALL));
            f.render_widget(p, area);
            return;
        }

        // Sync scroll offset with the real terminal height.
        let visible = (area.height as usize).saturating_sub(3).max(1);
        self.visible_rows = visible;
        self.update_rom_scroll(visible);

        let groups = self.rom_groups.as_ref().unwrap();
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
        let title = if total_roms > 0 && (groups.len() as u64) < total_roms {
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
            LibraryViewMode::Roms => "←: Back to list | ↑↓: Navigate | Enter: Game detail | Esc: Back",
        };
        let p =
            ratatui::widgets::Paragraph::new(help).block(Block::default().borders(Borders::ALL));
        f.render_widget(p, area);
    }
}
