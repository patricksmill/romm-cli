use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::widgets::{Block, Borders, Cell, List, ListItem, ListState, Paragraph, Row, Table};
use ratatui::Frame;

use crate::tui::text_search::LibrarySearchMode;
use crate::tui::theme::RommStyles;

use super::types::{LibraryBrowseScreen, LibrarySubsection, LibraryViewMode};

impl LibraryBrowseScreen {
    pub fn render(&mut self, f: &mut Frame, area: Rect, styles: &RommStyles) {
        let left = self.left_panel_percent;
        let chunks = Layout::default()
            .constraints([
                Constraint::Percentage(left),
                Constraint::Percentage(100 - left),
            ])
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
                    LibrarySearchMode::Filter => "Filter (list)",
                    LibrarySearchMode::Jump => "Jump (list)",
                };
                let p = Paragraph::new(format!("Search: {}", self.list_search.query))
                    .style(styles.text())
                    .block(styles.panel_block(title));
                f.render_widget(p, left_chunks[0]);
            }
            self.render_list(f, left_chunks[1], styles);
        } else {
            self.render_list(f, left_area, styles);
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
                LibrarySearchMode::Filter => "Filter (games)",
                LibrarySearchMode::Jump => "Jump (games)",
            };
            let p = Paragraph::new(format!("Search: {}", self.rom_search.query))
                .style(styles.text())
                .block(styles.panel_block(title));
            f.render_widget(p, right_chunks[0]);
            self.render_roms(f, right_chunks[1], styles);
            self.render_help(f, right_chunks[2], styles);
        } else {
            self.render_roms(f, right_chunks[0], styles);
            self.render_help(f, right_chunks[1], styles);
        }

        if self.upload_prompt.is_some() {
            self.render_upload_popup(f, area, styles);
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

    fn render_upload_popup(&mut self, f: &mut Frame, area: Rect, styles: &RommStyles) {
        let platform_name = self.upload_popup_platform_name();
        let Some(ref mut up) = self.upload_prompt else {
            return;
        };
        let popup = Self::upload_popup_rect(area);
        styles.fill_surface(f, popup);
        let scan_line = if up.scan_after {
            "Rescan after upload: yes — Ctrl+s to disable"
        } else {
            "Rescan after upload: no — Ctrl+s to enable"
        };
        let header = format!("{platform_name}\n{scan_line}");
        let block = styles.panel_block("Upload ROM (Ctrl+u)");
        let inner = block.inner(popup);
        let rows = Layout::default()
            .direction(ratatui::layout::Direction::Vertical)
            .constraints([Constraint::Length(2), Constraint::Min(8)])
            .split(inner);
        f.render_widget(block, popup);
        f.render_widget(Paragraph::new(header).style(styles.text()), rows[0]);
        let footer = "Enter: open/select   Ctrl+Enter: confirm file   ↑ list top: path   Tab: path/list   Ctrl+s: rescan";
        up.picker
            .render(f, rows[1], "Choose ROM file", footer, styles);
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

    fn render_list(&self, f: &mut Frame, area: Rect, styles: &RommStyles) {
        let visible = self.visible_list_source_indices();
        let labels = self.list_row_labels();

        let items: Vec<ListItem> = visible
            .iter()
            .map(|&source_idx| {
                let line = labels
                    .get(source_idx)
                    .cloned()
                    .unwrap_or_else(|| "?".to_string());
                ListItem::new(line).style(styles.text())
            })
            .collect();

        let list = List::new(items)
            .block(styles.panel_block(self.list_title()))
            .highlight_symbol(if self.view_mode == LibraryViewMode::List {
                ">> "
            } else {
                "   "
            })
            .highlight_style(styles.selection());

        let mut state = ListState::default();
        if self.view_mode == LibraryViewMode::List {
            state.select(Some(self.list_index));
        }

        f.render_stateful_widget(list, area, &mut state);
    }

    fn render_roms(&mut self, f: &mut Frame, area: Rect, styles: &RommStyles) {
        let visible = (area.height as usize).saturating_sub(3).max(1);
        self.visible_rows = visible;

        let groups = self.visible_rom_groups();
        if groups.is_empty() {
            let msg = self.empty_rom_state_message();
            let p = Paragraph::new(msg)
                .style(styles.text())
                .block(styles.panel_block("Games"));
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

        let header = Row::new(vec![Cell::from("Name").style(styles.label())]);
        let rows: Vec<Row> = visible_groups
            .iter()
            .enumerate()
            .map(|(i, g)| {
                let global_idx = start + i;
                let style = styles.row(i, global_idx == self.rom_selected);
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
            .block(styles.panel_block(title));

        f.render_widget(table, area);
    }

    pub(crate) fn empty_rom_state_message(&self) -> String {
        if self.rom_search.mode.is_some() || self.rom_search.filter_browsing {
            "No games match your search".to_string()
        } else if self.rom_loading && self.expected_rom_count() > 0 {
            "Loading games...".to_string()
        } else {
            "Select a console or collection and press Enter to load ROMs".to_string()
        }
    }

    fn render_help(&self, f: &mut Frame, area: Rect, styles: &RommStyles) {
        let help = match self.view_mode {
            LibraryViewMode::List => {
                if self.list_search.mode.is_some() {
                    "Type filter | Enter: browse matches | Esc: clear"
                } else if self.list_search.filter_browsing {
                    "↑↓: Navigate | Enter: Games | Esc: clear filter | /: Search | f: Filter"
                } else {
                    "t: Switch | ↑↓: Select | f: Filter | Ctrl+u: Upload | Enter: Games | /: Search | ,: Settings | d: Downloads | Ctrl+←/→: Resize | Esc: Quit"
                }
            }
            LibraryViewMode::Roms => {
                if self.rom_search.mode.is_some() {
                    "Type filter | Enter: browse matches | Esc: clear"
                } else if self.rom_search.filter_browsing {
                    "←: Back | ↑↓: Navigate | Enter: Detail | Esc: clear filter | /: Search | f: Filter"
                } else {
                    "←: Back | ↑↓: Navigate | f: Filter | Enter: Detail | /: Search | ,: Settings | d: Downloads | Ctrl+←/→: Resize | Esc: Back"
                }
            }
        };
        let text = match &self.metadata_footer {
            Some(m) if !m.is_empty() => format!("{m}\n{help}"),
            _ => help.to_string(),
        };
        let p = Paragraph::new(text)
            .style(styles.footer_hint())
            .block(styles.panel_block_untitled());
        f.render_widget(p, area);
    }
}
