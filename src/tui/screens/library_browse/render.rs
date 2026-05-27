use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::{Color, Style};
use ratatui::widgets::{
    Block, Borders, Cell, Clear, List, ListItem, ListState, Paragraph, Row, Table,
};
use ratatui::Frame;

use crate::tui::text_search::LibrarySearchMode;

use super::types::{LibraryBrowseScreen, LibrarySubsection, LibraryViewMode};

impl LibraryBrowseScreen {
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
                let p =
                    ratatui::widgets::Paragraph::new(format!("Search: {}", self.list_search.query))
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

        if self.upload_prompt.is_some() {
            self.render_upload_popup(f, area);
        }
    }

    fn upload_popup_rect(area: Rect) -> Rect {
        let w = (area.width * 4 / 5).max(50).min(area.width);
        let h = (area.height * 7 / 10).max(16).min(area.height);
        let x = area.x + (area.width.saturating_sub(w)) / 2;
        let y = area.y + (area.height.saturating_sub(h)) / 2;
        Rect {
            x,
            y,
            width: w,
            height: h,
        }
    }

    fn upload_popup_platform_name(&self) -> String {
        match self.subsection {
            LibrarySubsection::ByConsole => self
                .selected_list_source_index()
                .and_then(|i| self.platforms.get(i))
                .map(|p| p.display_name.as_deref().unwrap_or(&p.name).to_string())
                .unwrap_or_else(|| "?".to_string()),
            LibrarySubsection::ByCollection => "(switch to Consoles — t)".to_string(),
        }
    }

    fn render_upload_popup(&mut self, f: &mut Frame, area: Rect) {
        let platform_name = self.upload_popup_platform_name();
        let Some(ref mut up) = self.upload_prompt else {
            return;
        };
        let popup = Self::upload_popup_rect(area);
        f.render_widget(Clear, popup);
        let scan_line = if up.scan_after {
            "Rescan after upload: yes — Ctrl+s to disable"
        } else {
            "Rescan after upload: no — Ctrl+s to enable"
        };
        let header = format!("{platform_name}\n{scan_line}");
        let inner = Block::default()
            .title("Upload ROM (Ctrl+u)")
            .borders(Borders::ALL)
            .inner(popup);
        let rows = Layout::default()
            .direction(ratatui::layout::Direction::Vertical)
            .constraints([Constraint::Length(2), Constraint::Min(8)])
            .split(inner);
        f.render_widget(
            Block::default()
                .title("Upload ROM (Ctrl+u)")
                .borders(Borders::ALL),
            popup,
        );
        f.render_widget(Paragraph::new(header), rows[0]);
        let footer = "Enter: open/select   Ctrl+Enter: confirm file   ↑ list top: path   Tab: path/list   Ctrl+s: rescan";
        up.picker.render(f, rows[1], "Choose ROM file", footer);
    }

    /// Cursor for the upload path field (when [`Self::upload_prompt`] is open).
    pub fn upload_prompt_cursor(&self, area: Rect) -> Option<(u16, u16)> {
        let up = self.upload_prompt.as_ref()?;
        let popup = Self::upload_popup_rect(area);
        let inner = Block::default()
            .title("Upload ROM (Ctrl+u)")
            .borders(Borders::ALL)
            .inner(popup);
        let rows = Layout::default()
            .direction(ratatui::layout::Direction::Vertical)
            .constraints([Constraint::Length(2), Constraint::Min(8)])
            .split(inner);
        up.picker.cursor_position(rows[1], "Choose ROM file")
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
                let prefix = if pos == self.list_index && self.view_mode == LibraryViewMode::List {
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
            let msg = self.empty_rom_state_message();
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
        let mut title = if self.rom_search.filter_browsing && !self.rom_search.query.is_empty() {
            format!(
                "Games (filtered: \"{}\") — {} — {} files",
                self.rom_search.query,
                groups.len(),
                total_files
            )
        } else if total_roms > 0 && (total_files as u64) < total_roms {
            format!(
                "Games ({} of {}) — {} files",
                total_files, total_roms, total_files
            )
        } else {
            format!("Games ({}) — {} files", groups.len(), total_files)
        };

        if self.rom_loading {
            title.push_str(" [Loading...]");
        }
        let widths = [Constraint::Percentage(100)];
        let table = Table::new(rows, widths)
            .header(header)
            .block(Block::default().title(title).borders(Borders::ALL));

        f.render_widget(table, area);
    }

    pub(crate) fn empty_rom_state_message(&self) -> String {
        if self.rom_search.mode.is_some() {
            "No games match your search".to_string()
        } else if self.rom_loading && self.expected_rom_count() > 0 {
            "Loading games...".to_string()
        } else {
            "Select a console or collection and press Enter to load ROMs".to_string()
        }
    }

    fn render_help(&self, f: &mut Frame, area: Rect) {
        let help = match self.view_mode {
            LibraryViewMode::List => {
                if self.list_search.mode.is_some() {
                    "Type filter | Enter: browse matches | Esc: clear"
                } else if self.list_search.filter_browsing {
                    "↑↓: Navigate | Enter: Load games | Esc: clear filter"
                } else {
                    "t: Switch | ↑↓: Select | Ctrl+u: Upload | / f: Filter | Enter: Games | Shift+/: Help | Esc: Menu"
                }
            }
            LibraryViewMode::Roms => {
                if self.rom_search.mode.is_some() {
                    "Type filter | Enter: browse matches | Esc: clear filter"
                } else if self.rom_search.filter_browsing {
                    "←: Back to list | ↑↓: Navigate | Enter: Game detail | Esc: clear filter"
                } else {
                    "←: Back | ↑↓: Navigate | Ctrl+u: Upload | / f: Filter | Enter: Detail | Shift+/: Help | Esc: Back"
                }
            }
        };
        let text = match &self.metadata_footer {
            Some(m) if !m.is_empty() => format!("{m}\n{help}"),
            _ => help.to_string(),
        };
        let p =
            ratatui::widgets::Paragraph::new(text).block(Block::default().borders(Borders::ALL));
        f.render_widget(p, area);
    }
}
