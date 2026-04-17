use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::{Color, Style};
use ratatui::widgets::{Block, Borders, Cell, Paragraph, Row, Table};
use ratatui::Frame;

use crate::core::utils::{self, RomGroup};
use crate::types::{Rom, RomList};

/// Full-text search screen over ROMs, with grouped results.
pub struct SearchScreen {
    pub query: String,
    pub cursor_pos: usize,
    pub results: Option<RomList>,
    /// One row per game name (base + updates/DLC grouped).
    pub result_groups: Option<Vec<RomGroup>>,
    /// Query string used for the API call that produced [`Self::result_groups`], if any.
    pub last_searched_query: Option<String>,
    pub selected: usize,
    pub scroll_offset: usize,
    visible_rows: usize,
}

impl Default for SearchScreen {
    fn default() -> Self {
        Self::new()
    }
}

impl SearchScreen {
    pub fn new() -> Self {
        Self {
            query: String::new(),
            cursor_pos: 0,
            results: None,
            result_groups: None,
            last_searched_query: None,
            selected: 0,
            scroll_offset: 0,
            visible_rows: 15,
        }
    }

    pub fn add_char(&mut self, c: char) {
        let pos = self.cursor_pos.min(self.query.len());
        self.query.insert(pos, c);
        self.cursor_pos = pos + 1;
    }

    pub fn delete_char(&mut self) {
        if self.cursor_pos > 0 && self.cursor_pos <= self.query.len() {
            self.query.remove(self.cursor_pos - 1);
            self.cursor_pos -= 1;
        }
    }

    pub fn cursor_left(&mut self) {
        if self.cursor_pos > 0 {
            self.cursor_pos -= 1;
        }
    }

    pub fn cursor_right(&mut self) {
        if self.cursor_pos < self.query.len() {
            self.cursor_pos += 1;
        }
    }

    pub fn set_results(&mut self, results: RomList) {
        self.results = Some(results.clone());
        self.result_groups = Some(utils::group_roms_by_name(&results.items));
        self.last_searched_query = Some(self.query.clone());
        self.selected = 0;
        self.scroll_offset = 0;
    }

    pub fn clear_results(&mut self) {
        self.results = None;
        self.result_groups = None;
        self.last_searched_query = None;
    }

    /// True when the on-screen results were fetched for the current `query` string.
    pub fn results_match_current_query(&self) -> bool {
        self.last_searched_query.as_deref() == Some(self.query.as_str())
    }

    pub fn next(&mut self) {
        if let Some(ref g) = self.result_groups {
            if !g.is_empty() {
                self.selected = (self.selected + 1) % g.len();
                self.update_scroll(self.visible_rows);
            }
        }
    }

    pub fn previous(&mut self) {
        if let Some(ref g) = self.result_groups {
            if !g.is_empty() {
                self.selected = if self.selected == 0 {
                    g.len() - 1
                } else {
                    self.selected - 1
                };
                self.update_scroll(self.visible_rows);
            }
        }
    }

    fn update_scroll(&mut self, visible: usize) {
        if let Some(ref g) = self.result_groups {
            let visible = visible.max(1);
            let max_scroll = g.len().saturating_sub(visible);
            if self.selected >= self.scroll_offset + visible {
                self.scroll_offset = (self.selected + 1).saturating_sub(visible);
            } else if self.selected < self.scroll_offset {
                self.scroll_offset = self.selected;
            }
            self.scroll_offset = self.scroll_offset.min(max_scroll);
        }
    }

    /// Primary ROM and other files (updates/DLC) for the selected game.
    pub fn get_selected_group(&self) -> Option<(Rom, Vec<Rom>)> {
        self.result_groups
            .as_ref()
            .and_then(|g| g.get(self.selected))
            .map(|g| (g.primary.clone(), g.others.clone()))
    }

    pub fn render(&mut self, f: &mut Frame, area: Rect) {
        let chunks = Layout::default()
            .constraints([
                Constraint::Length(3),
                Constraint::Min(5),
                Constraint::Length(3),
            ])
            .direction(ratatui::layout::Direction::Vertical)
            .split(area);

        let input_line = format!("Search: {}", self.query);
        let input = Paragraph::new(input_line)
            .block(Block::default().title("Search games").borders(Borders::ALL));
        f.render_widget(input, chunks[0]);

        if self.result_groups.is_some() {
            let visible = (chunks[1].height as usize).saturating_sub(3).max(1);
            // Keep selection and viewport aligned with the current terminal size.
            self.visible_rows = visible;
            self.update_scroll(visible);
            let Some(groups) = self.result_groups.as_ref() else {
                return;
            };
            let start = self.scroll_offset.min(groups.len().saturating_sub(visible));
            let end = (start + visible).min(groups.len());
            let visible_groups = &groups[start..end];

            let header = Row::new(vec![
                Cell::from("Name").style(Style::default().fg(Color::Cyan)),
                Cell::from("Platform").style(Style::default().fg(Color::Cyan)),
            ]);
            let rows: Vec<Row> = visible_groups
                .iter()
                .enumerate()
                .map(|(i, g)| {
                    let global_idx = start + i;
                    let platform = g
                        .primary
                        .platform_display_name
                        .as_deref()
                        .or(g.primary.platform_custom_name.as_deref())
                        .unwrap_or("—");
                    let style = if global_idx == self.selected {
                        Style::default().fg(Color::Yellow)
                    } else {
                        Style::default()
                    };
                    Row::new(vec![
                        Cell::from(g.name.as_str()).style(style),
                        Cell::from(platform).style(style),
                    ])
                    .height(1)
                })
                .collect();

            let total_files = self.results.as_ref().map(|r| r.items.len()).unwrap_or(0);
            let widths = [Constraint::Percentage(60), Constraint::Percentage(40)];
            let table = Table::new(rows, widths).header(header).block(
                Block::default()
                    .title(format!(
                        "Results ({}) — {} files",
                        groups.len(),
                        total_files
                    ))
                    .borders(Borders::ALL),
            );
            f.render_widget(table, chunks[1]);
        } else {
            let msg = "Type a search term and press Enter to search";
            let p =
                Paragraph::new(msg).block(Block::default().title("Results").borders(Borders::ALL));
            f.render_widget(p, chunks[1]);
        }

        let help = "Enter: Search (or open game if query unchanged) | ↑↓: Navigate | Esc: Back";
        let p = Paragraph::new(help).block(Block::default().borders(Borders::ALL));
        f.render_widget(p, chunks[2]);
    }

    pub fn cursor_position(&self, area: Rect) -> Option<(u16, u16)> {
        let chunks = Layout::default()
            .constraints([
                Constraint::Length(3),
                Constraint::Min(5),
                Constraint::Length(3),
            ])
            .direction(ratatui::layout::Direction::Vertical)
            .split(area);
        let offset = 9 + self.cursor_pos.min(self.query.len()) as u16;
        let x = chunks[0].x + offset.min(chunks[0].width.saturating_sub(1));
        let y = chunks[0].y + 1;
        Some((x, y))
    }
}

#[cfg(test)]
mod tests {
    use super::SearchScreen;
    use crate::types::RomList;

    fn empty_list() -> RomList {
        RomList {
            items: vec![],
            total: 0,
            limit: 50,
            offset: 0,
        }
    }

    #[test]
    fn set_results_records_last_searched_query() {
        let mut s = SearchScreen::new();
        s.query = "mario".to_string();
        s.set_results(empty_list());
        assert_eq!(s.last_searched_query.as_deref(), Some("mario"));
        assert!(s.results_match_current_query());
    }

    #[test]
    fn editing_query_after_search_marks_stale() {
        let mut s = SearchScreen::new();
        s.query = "mario".to_string();
        s.cursor_pos = s.query.len();
        s.set_results(empty_list());
        assert!(s.results_match_current_query());
        s.delete_char();
        assert_eq!(s.query, "mari");
        assert!(!s.results_match_current_query());
    }

    #[test]
    fn clear_results_clears_last_searched_query() {
        let mut s = SearchScreen::new();
        s.query = "a".to_string();
        s.set_results(empty_list());
        s.clear_results();
        assert!(s.last_searched_query.is_none());
    }
}
