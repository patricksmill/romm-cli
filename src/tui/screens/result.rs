use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::{Color, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Cell, Paragraph, Row, Scrollbar, ScrollbarState, Table};
use ratatui::Frame;

use crate::tui::utils::open_in_browser;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ResultViewMode {
    Json,
    Table,
}

pub struct ResultScreen {
    pub raw: serde_json::Value,
    pub highlighted_lines: Vec<Line<'static>>,
    pub scroll: usize,
    pub scrollbar_state: ScrollbarState,
    pub view_mode: ResultViewMode,
    pub table_selected: usize,
    pub table_row_count: usize,
    pub message: Option<String>,
}

/// Detail view for a single ROM/item when pressing Enter on the table.
pub struct ResultDetailScreen {
    pub parent: ResultScreen,
    pub item: serde_json::Value,
    pub table_rows: Vec<(String, String)>, // (field_name, field_value)
    pub scroll: usize,
    pub scrollbar_state: ScrollbarState,
    pub message: Option<String>,
}

impl ResultScreen {
    /// Create a result screen. If `endpoint_method` is "GET" and `endpoint_path` is "/api/roms"
    /// and the response has list items, table view is used as default.
    pub fn new(
        result: serde_json::Value,
        endpoint_method: Option<&str>,
        endpoint_path: Option<&str>,
    ) -> Self {
        let result_text =
            serde_json::to_string_pretty(&result).unwrap_or_else(|_| format!("{:?}", result));
        let highlighted_lines = Self::highlight_json_lines(&result_text);
        let line_count = highlighted_lines.len().max(1);
        let scrollbar_state = ScrollbarState::new(line_count.saturating_sub(1));

        let (table_row_count, _) = Self::items_from_value(&result);

        let prefer_table = endpoint_method.map_or(false, |m| m.eq_ignore_ascii_case("GET"))
            && endpoint_path.map_or(false, |p| p.trim_end_matches('/') == "/api/roms")
            && table_row_count > 0;

        Self {
            raw: result,
            highlighted_lines,
            scroll: 0,
            scrollbar_state,
            view_mode: if prefer_table {
                ResultViewMode::Table
            } else {
                ResultViewMode::Json
            },
            table_selected: 0,
            table_row_count,
            message: None,
        }
    }

    fn highlight_json_lines(text: &str) -> Vec<Line<'static>> {
        let mut out = Vec::new();
        for line in text.lines() {
            out.push(Self::highlight_json_line(line));
        }
        if out.is_empty() {
            out.push(Line::from(Span::raw("")));
        }
        out
    }

    fn highlight_json_line(line: &str) -> Line<'static> {
        let key_style = Style::default().fg(Color::Cyan);
        let string_style = Style::default().fg(Color::Green);
        let number_style = Style::default().fg(Color::Yellow);
        let bool_null_style = Style::default().fg(Color::Magenta);
        let default_style = Style::default();

        let mut spans = Vec::new();
        let bytes = line.as_bytes();
        let mut i = 0;

        while i < bytes.len() {
            if bytes[i] == b'"' {
                let mut end = i + 1;
                while end < bytes.len() {
                    if bytes[end] == b'\\' && end + 1 < bytes.len() {
                        end += 2;
                        continue;
                    }
                    if bytes[end] == b'"' {
                        end += 1;
                        break;
                    }
                    end += 1;
                }
                let s = std::str::from_utf8(&bytes[i..end]).unwrap_or("");
                let rest_trimmed = bytes.get(end..).and_then(|s| {
                    let mut j = 0;
                    while j < s.len() && (s[j] == b' ' || s[j] == b'\t') {
                        j += 1;
                    }
                    s.get(j..)
                });
                let is_key = rest_trimmed
                    .map(|r| r.first() == Some(&b':'))
                    .unwrap_or(false);
                if is_key {
                    spans.push(Span::styled(s.to_string(), key_style));
                } else {
                    spans.push(Span::styled(s.to_string(), string_style));
                }
                i = end;
                continue;
            }
            if bytes[i].is_ascii_digit()
                || (bytes[i] == b'-' && i + 1 < bytes.len() && bytes[i + 1].is_ascii_digit())
            {
                let mut end = i;
                if bytes[end] == b'-' {
                    end += 1;
                }
                while end < bytes.len()
                    && (bytes[end].is_ascii_digit()
                        || bytes[end] == b'.'
                        || bytes[end] == b'e'
                        || bytes[end] == b'E'
                        || bytes[end] == b'+'
                        || bytes[end] == b'-')
                {
                    end += 1;
                }
                let s = std::str::from_utf8(&bytes[i..end]).unwrap_or("");
                spans.push(Span::styled(s.to_string(), number_style));
                i = end;
                continue;
            }
            if i + 4 <= bytes.len() && std::str::from_utf8(&bytes[i..i + 4]).unwrap_or("") == "true"
            {
                spans.push(Span::styled("true".to_string(), bool_null_style));
                i += 4;
                continue;
            }
            if i + 5 <= bytes.len()
                && std::str::from_utf8(&bytes[i..i + 5]).unwrap_or("") == "false"
            {
                spans.push(Span::styled("false".to_string(), bool_null_style));
                i += 5;
                continue;
            }
            if i + 4 <= bytes.len() && std::str::from_utf8(&bytes[i..i + 4]).unwrap_or("") == "null"
            {
                spans.push(Span::styled("null".to_string(), bool_null_style));
                i += 4;
                continue;
            }
            let ch = std::str::from_utf8(&bytes[i..(i + 1).min(bytes.len())]).unwrap_or("");
            spans.push(Span::styled(ch.to_string(), default_style));
            i += 1;
        }
        if spans.is_empty() {
            Line::from(Span::raw(line.to_string()))
        } else {
            Line::from(spans)
        }
    }

    fn items_from_value(v: &serde_json::Value) -> (usize, Option<&Vec<serde_json::Value>>) {
        let obj = match v.as_object() {
            Some(o) => o,
            None => return (0, None),
        };
        let items = match obj.get("items").and_then(|i| i.as_array()) {
            Some(arr) => arr,
            None => return (0, None),
        };
        let total = obj
            .get("total")
            .and_then(|t| t.as_u64())
            .unwrap_or(items.len() as u64) as usize;
        (total.min(items.len()), Some(items))
    }

    pub fn collect_image_urls(value: &serde_json::Value) -> Vec<String> {
        let mut urls = Vec::new();
        fn collect(v: &serde_json::Value, out: &mut Vec<String>) {
            match v {
                serde_json::Value::Object(m) => {
                    for (k, val) in m {
                        if (k == "url_cover")
                            || (k == "url_logo")
                            || (k.starts_with("url_") && k.contains("cover"))
                        {
                            if let Some(s) = val.as_str().filter(|s| !s.is_empty()) {
                                out.push(s.to_string());
                            }
                        }
                        collect(val, out);
                    }
                }
                serde_json::Value::Array(arr) => {
                    for item in arr {
                        collect(item, out);
                    }
                }
                _ => {}
            }
        }
        collect(value, &mut urls);
        urls
    }

    /// Returns the selected row value when in table view (for opening detail screen).
    pub fn get_selected_item_value(&self) -> Option<serde_json::Value> {
        if self.view_mode != ResultViewMode::Table {
            return None;
        }
        let (_, items_opt) = Self::items_from_value(&self.raw);
        let items = items_opt?;
        let row = items.get(self.table_selected.min(items.len().saturating_sub(1)))?;
        Some(row.clone())
    }

    pub fn scroll_down(&mut self, amount: usize) {
        let max_scroll = self.highlighted_lines.len().saturating_sub(1);
        self.scroll = (self.scroll + amount).min(max_scroll);
        self.scrollbar_state = self.scrollbar_state.position(self.scroll);
    }

    pub fn scroll_up(&mut self, amount: usize) {
        self.scroll = self.scroll.saturating_sub(amount);
        self.scrollbar_state = self.scrollbar_state.position(self.scroll);
    }

    pub fn table_next(&mut self) {
        if self.table_row_count > 0 {
            self.table_selected = (self.table_selected + 1) % self.table_row_count;
        }
    }

    pub fn table_previous(&mut self) {
        if self.table_row_count > 0 {
            self.table_selected = if self.table_selected == 0 {
                self.table_row_count - 1
            } else {
                self.table_selected - 1
            };
        }
    }

    pub fn table_page_up(&mut self) {
        const PAGE: usize = 10;
        self.table_selected = self.table_selected.saturating_sub(PAGE);
    }

    pub fn table_page_down(&mut self) {
        const PAGE: usize = 10;
        if self.table_row_count > 0 {
            self.table_selected = (self.table_selected + PAGE).min(self.table_row_count - 1);
        }
    }

    pub fn switch_view_mode(&mut self) {
        self.view_mode = match self.view_mode {
            ResultViewMode::Json => {
                if self.table_row_count > 0 {
                    ResultViewMode::Table
                } else {
                    ResultViewMode::Json
                }
            }
            ResultViewMode::Table => ResultViewMode::Json,
        };
        self.table_selected = 0;
    }

    pub fn clear_message(&mut self) {
        self.message = None;
    }

    pub fn render(&self, f: &mut Frame, area: Rect) {
        let chunks = Layout::default()
            .constraints([Constraint::Min(3), Constraint::Length(3)])
            .direction(ratatui::layout::Direction::Vertical)
            .split(area);

        let content_area = chunks[0];
        match self.view_mode {
            ResultViewMode::Json => self.render_json(f, content_area),
            ResultViewMode::Table => self.render_table(f, content_area),
        }

        let help = match self.view_mode {
            ResultViewMode::Json => "t: Toggle view | ↑↓: Scroll | Esc: Back",
            ResultViewMode::Table => "t: Toggle view | Enter: Detail | ↑↓: Row | Esc: Back",
        };
        let msg = self.message.as_deref().unwrap_or(help);
        let footer = Paragraph::new(msg).block(Block::default().borders(Borders::ALL));
        f.render_widget(footer, chunks[1]);
    }

    fn render_json(&self, f: &mut Frame, area: Rect) {
        let visible: Vec<Line> = self
            .highlighted_lines
            .iter()
            .skip(self.scroll)
            .take(area.height as usize - 2)
            .cloned()
            .collect();

        let paragraph = Paragraph::new(visible)
            .block(
                Block::default()
                    .title("Response (JSON)")
                    .borders(Borders::ALL),
            )
            .wrap(ratatui::widgets::Wrap { trim: true });

        f.render_widget(paragraph, area);

        let mut state = self.scrollbar_state.clone();
        let scrollbar = Scrollbar::default()
            .orientation(ratatui::widgets::ScrollbarOrientation::VerticalRight)
            .begin_symbol(Some("↑"))
            .end_symbol(Some("↓"));
        f.render_stateful_widget(scrollbar, area, &mut state);
    }

    fn render_table(&self, f: &mut Frame, area: Rect) {
        let (_, items_opt) = Self::items_from_value(&self.raw);
        let items = match items_opt {
            Some(arr) if !arr.is_empty() => arr,
            _ => {
                let p = Paragraph::new("No items array or empty").block(
                    Block::default()
                        .title("Response (Table)")
                        .borders(Borders::ALL),
                );
                f.render_widget(p, area);
                return;
            }
        };

        // Table block: 1 top border + 1 header + N data rows + 1 bottom border
        let visible_row_count = (area.height as usize).saturating_sub(3).max(1);
        let max_scroll_start = items.len().saturating_sub(visible_row_count);
        let scroll_start = (self.table_selected + 1)
            .saturating_sub(visible_row_count)
            .min(max_scroll_start);
        let scroll_end = (scroll_start + visible_row_count).min(items.len());
        let visible_items = &items[scroll_start..scroll_end];

        let header_cells = ["id", "name", "platform_id", "cover"]
            .iter()
            .map(|h| Cell::from(*h).style(Style::default().fg(Color::Cyan)));
        let header = Row::new(header_cells).height(1);

        let rows: Vec<Row> = visible_items
            .iter()
            .enumerate()
            .map(|(local_idx, row)| {
                let global_idx = scroll_start + local_idx;
                let empty = serde_json::Map::new();
                let obj = row.as_object().unwrap_or(&empty);
                let id = obj
                    .get("id")
                    .and_then(|v| v.as_u64())
                    .map(|n| n.to_string())
                    .unwrap_or_else(|| "-".to_string());
                let name = obj
                    .get("name")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();
                let pid_num = obj.get("platform_id").and_then(|v| v.as_u64());
                let platform_name = obj
                    .get("platform_display_name")
                    .or_else(|| obj.get("platform_custom_name"))
                    .or_else(|| obj.get("platform_name"))
                    .and_then(|v| v.as_str())
                    .filter(|s| !s.is_empty());
                let pid = match (platform_name, pid_num) {
                    (Some(name), Some(id)) => format!("{} ({})", name, id),
                    (None, Some(id)) => format!("({})", id),
                    _ => "-".to_string(),
                };
                let cover = if obj.get("url_cover").or(obj.get("url_logo")).is_some() {
                    "[IMG]"
                } else {
                    "-"
                };
                let style = if global_idx == self.table_selected {
                    Style::default().fg(Color::Yellow)
                } else {
                    Style::default()
                };
                Row::new(vec![
                    Cell::from(id).style(style),
                    Cell::from(name).style(style),
                    Cell::from(pid).style(style),
                    Cell::from(cover).style(style),
                ])
                .height(1)
            })
            .collect();

        let widths = [
            Constraint::Length(6),
            Constraint::Percentage(40),
            Constraint::Min(16),
            Constraint::Length(6),
        ];
        let title = format!(
            "Response (Table) - {} rows {}-{}",
            items.len(),
            scroll_start + 1,
            scroll_end
        );
        let table = Table::new(rows, widths)
            .header(header)
            .block(Block::default().title(title).borders(Borders::ALL));

        f.render_widget(table, area);
    }
}

impl ResultDetailScreen {
    pub fn new(parent: ResultScreen, item: serde_json::Value) -> Self {
        let table_rows = Self::value_to_table_rows(&item);
        let row_count = table_rows.len().max(1);
        let scrollbar_state = ScrollbarState::new(row_count.saturating_sub(1));

        Self {
            parent,
            item,
            table_rows,
            scroll: 0,
            scrollbar_state,
            message: None,
        }
    }

    fn value_to_table_rows(value: &serde_json::Value) -> Vec<(String, String)> {
        let mut rows = Vec::new();
        if let Some(obj) = value.as_object() {
            for (key, val) in obj {
                let value_str = match val {
                    serde_json::Value::Null => "null".to_string(),
                    serde_json::Value::Bool(b) => b.to_string(),
                    serde_json::Value::Number(n) => n.to_string(),
                    serde_json::Value::String(s) => s.clone(),
                    serde_json::Value::Array(_) => {
                        format!("[{} items]", val.as_array().map(|a| a.len()).unwrap_or(0))
                    }
                    serde_json::Value::Object(_) => format!(
                        "{{{} fields}}",
                        val.as_object().map(|o| o.len()).unwrap_or(0)
                    ),
                };
                rows.push((key.clone(), value_str));
            }
        }
        rows.sort_by(|a, b| a.0.cmp(&b.0)); // Sort by field name
        rows
    }

    pub fn scroll_down(&mut self, amount: usize) {
        let max_scroll = self.table_rows.len().saturating_sub(1);
        self.scroll = (self.scroll + amount).min(max_scroll);
        self.scrollbar_state = self.scrollbar_state.position(self.scroll);
    }

    pub fn scroll_up(&mut self, amount: usize) {
        self.scroll = self.scroll.saturating_sub(amount);
        self.scrollbar_state = self.scrollbar_state.position(self.scroll);
    }

    pub fn open_image_url(&mut self) {
        self.message = None;
        let urls = ResultScreen::collect_image_urls(&self.item);
        let url = match urls.first() {
            Some(u) => u.clone(),
            None => {
                self.message = Some("No image URL".to_string());
                return;
            }
        };
        match open_in_browser(&url) {
            Ok(_) => self.message = Some("Opened in browser".to_string()),
            Err(e) => self.message = Some(format!("Failed to open: {}", e)),
        }
    }

    pub fn clear_message(&mut self) {
        self.message = None;
    }

    pub fn render(&self, f: &mut Frame, area: Rect) {
        let chunks = Layout::default()
            .constraints([Constraint::Min(3), Constraint::Length(3)])
            .direction(ratatui::layout::Direction::Vertical)
            .split(area);

        let content_area = chunks[0];

        // Table block: 1 top border + 1 header + N data rows + 1 bottom border
        let visible_row_count = (content_area.height as usize).saturating_sub(3).max(1);
        let max_scroll = self.table_rows.len().saturating_sub(visible_row_count);
        let scroll_start = self.scroll.min(max_scroll);
        let scroll_end = (scroll_start + visible_row_count).min(self.table_rows.len());
        let visible_rows = &self.table_rows[scroll_start..scroll_end];

        let header_cells = ["Field", "Value"]
            .iter()
            .map(|h| Cell::from(*h).style(Style::default().fg(Color::Cyan)));
        let header = Row::new(header_cells).height(1);

        let rows: Vec<Row> = visible_rows
            .iter()
            .map(|(key, value)| {
                Row::new(vec![
                    Cell::from(key.clone()).style(Style::default().fg(Color::Yellow)),
                    Cell::from(value.clone()),
                ])
                .height(1)
            })
            .collect();

        let widths = [Constraint::Percentage(30), Constraint::Percentage(70)];
        let title = format!("ROM detail - {} fields", self.table_rows.len());
        let table = Table::new(rows, widths)
            .header(header)
            .block(Block::default().title(title).borders(Borders::ALL));

        f.render_widget(table, content_area);

        let mut state = self.scrollbar_state.clone();
        let scrollbar = Scrollbar::default()
            .orientation(ratatui::widgets::ScrollbarOrientation::VerticalRight)
            .begin_symbol(Some("↑"))
            .end_symbol(Some("↓"));
        f.render_stateful_widget(scrollbar, content_area, &mut state);

        let help = "o: Open image | ↑↓: Scroll | Esc: Back";
        let msg = self.message.as_deref().unwrap_or(help);
        let footer = Paragraph::new(msg).block(Block::default().borders(Borders::ALL));
        f.render_widget(footer, chunks[1]);
    }
}
