use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::widgets::{Block, Borders, Paragraph};
use ratatui::Frame;

use crate::tui::openapi::ApiEndpoint;

/// Screen for editing path/query/body parameters and executing a single endpoint.
pub struct ExecuteScreen {
    pub endpoint: ApiEndpoint,
    pub path_params: Vec<(String, String)>,
    pub path_param_input_idx: usize,
    pub query_params: Vec<(String, String)>,
    pub body_text: String,
    pub focused_field: FocusedField,
    pub param_input_idx: usize,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum FocusedField {
    PathParams,
    QueryParams,
    Body,
}

struct MiddleAreas {
    path: Option<Rect>,
    query: Rect,
    body: Option<Rect>,
}

impl ExecuteScreen {
    pub fn new(endpoint: ApiEndpoint) -> Self {
        let path_params: Vec<(String, String)> = endpoint
            .path_params
            .iter()
            .map(|p| (p.name.clone(), p.default.clone().unwrap_or_default()))
            .collect();

        let query_params: Vec<(String, String)> = endpoint
            .query_params
            .iter()
            .map(|p| (p.name.clone(), p.default.clone().unwrap_or_default()))
            .collect();

        let focused_field = if !path_params.is_empty() {
            FocusedField::PathParams
        } else {
            FocusedField::QueryParams
        };

        Self {
            endpoint,
            path_params,
            path_param_input_idx: 0,
            query_params,
            body_text: String::new(),
            focused_field,
            param_input_idx: 0,
        }
    }

    pub fn get_path_params(&self) -> Vec<(String, String)> {
        self.path_params
            .iter()
            .filter(|(_, v)| !v.is_empty())
            .cloned()
            .collect()
    }

    pub fn get_query_params(&self) -> Vec<(String, String)> {
        self.query_params
            .iter()
            .filter(|(_, v)| !v.is_empty())
            .cloned()
            .collect()
    }

    fn middle_areas(&self, mid: Rect) -> MiddleAreas {
        let has_path = !self.path_params.is_empty();

        if !has_path {
            if self.endpoint.has_body {
                let q = Layout::default()
                    .direction(Direction::Horizontal)
                    .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
                    .split(mid);
                return MiddleAreas {
                    path: None,
                    query: q[0],
                    body: Some(q[1]),
                };
            }
            return MiddleAreas {
                path: None,
                query: mid,
                body: None,
            };
        }

        let path_lines = self.path_params.len() as u16;
        let path_h = (path_lines + 2).clamp(3, mid.height.saturating_sub(4));
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(path_h), Constraint::Min(3)])
            .split(mid);
        let path_r = chunks[0];
        let bottom = chunks[1];

        if self.endpoint.has_body {
            let q = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
                .split(bottom);
            MiddleAreas {
                path: Some(path_r),
                query: q[0],
                body: Some(q[1]),
            }
        } else {
            MiddleAreas {
                path: Some(path_r),
                query: bottom,
                body: None,
            }
        }
    }

    pub fn next_field(&mut self) {
        match self.focused_field {
            FocusedField::PathParams => {
                if self.path_param_input_idx + 1 < self.path_params.len() {
                    self.path_param_input_idx += 1;
                } else if !self.query_params.is_empty() {
                    self.focused_field = FocusedField::QueryParams;
                    self.param_input_idx = 0;
                } else if self.endpoint.has_body {
                    self.focused_field = FocusedField::Body;
                } else {
                    self.path_param_input_idx = 0;
                }
            }
            FocusedField::QueryParams => {
                if self.param_input_idx + 1 < self.query_params.len() {
                    self.param_input_idx += 1;
                } else if self.endpoint.has_body {
                    self.focused_field = FocusedField::Body;
                } else if !self.path_params.is_empty() {
                    self.focused_field = FocusedField::PathParams;
                    self.path_param_input_idx = 0;
                } else {
                    self.param_input_idx = 0;
                }
            }
            FocusedField::Body => {
                if !self.path_params.is_empty() {
                    self.focused_field = FocusedField::PathParams;
                    self.path_param_input_idx = 0;
                } else {
                    self.focused_field = FocusedField::QueryParams;
                    self.param_input_idx = 0;
                }
            }
        }
    }

    pub fn previous_field(&mut self) {
        match self.focused_field {
            FocusedField::PathParams => {
                if self.path_param_input_idx > 0 {
                    self.path_param_input_idx -= 1;
                } else if self.endpoint.has_body {
                    self.focused_field = FocusedField::Body;
                } else if !self.query_params.is_empty() {
                    self.focused_field = FocusedField::QueryParams;
                    self.param_input_idx = self.query_params.len().saturating_sub(1);
                } else {
                    self.path_param_input_idx = self.path_params.len().saturating_sub(1);
                }
            }
            FocusedField::QueryParams => {
                if self.param_input_idx > 0 {
                    self.param_input_idx -= 1;
                } else if !self.path_params.is_empty() {
                    self.focused_field = FocusedField::PathParams;
                    self.path_param_input_idx = self.path_params.len().saturating_sub(1);
                } else if self.endpoint.has_body {
                    self.focused_field = FocusedField::Body;
                } else {
                    self.param_input_idx = self.query_params.len().saturating_sub(1);
                }
            }
            FocusedField::Body => {
                if !self.query_params.is_empty() {
                    self.focused_field = FocusedField::QueryParams;
                    self.param_input_idx = self.query_params.len().saturating_sub(1);
                } else if !self.path_params.is_empty() {
                    self.focused_field = FocusedField::PathParams;
                    self.path_param_input_idx = self.path_params.len().saturating_sub(1);
                }
            }
        }
    }

    pub fn add_char_to_focused(&mut self, c: char) {
        match self.focused_field {
            FocusedField::PathParams => {
                if let Some((_, ref mut v)) = self.path_params.get_mut(self.path_param_input_idx) {
                    v.push(c);
                }
            }
            FocusedField::QueryParams => {
                if let Some((_, ref mut v)) = self.query_params.get_mut(self.param_input_idx) {
                    v.push(c);
                }
            }
            FocusedField::Body => {
                self.body_text.push(c);
            }
        }
    }

    pub fn delete_char_from_focused(&mut self) {
        match self.focused_field {
            FocusedField::PathParams => {
                if let Some((_, ref mut v)) = self.path_params.get_mut(self.path_param_input_idx) {
                    v.pop();
                }
            }
            FocusedField::QueryParams => {
                if let Some((_, ref mut v)) = self.query_params.get_mut(self.param_input_idx) {
                    v.pop();
                }
            }
            FocusedField::Body => {
                self.body_text.pop();
            }
        }
    }

    pub fn cursor_position(&self, area: Rect) -> Option<(u16, u16)> {
        let chunks = Layout::default()
            .constraints([
                Constraint::Length(3),
                Constraint::Min(5),
                Constraint::Length(3),
            ])
            .split(area);

        let mid = self.middle_areas(chunks[1]);

        match self.focused_field {
            FocusedField::PathParams => mid.path.and_then(|pa| self.cursor_in_path_params(pa)),
            FocusedField::QueryParams => self.cursor_in_query_params(mid.query),
            FocusedField::Body => mid.body.and_then(|ba| self.cursor_in_body(ba)),
        }
    }

    fn cursor_in_path_params(&self, area: Rect) -> Option<(u16, u16)> {
        if area.width < 3 || area.height < 3 {
            return None;
        }

        let idx = self
            .path_param_input_idx
            .min(self.path_params.len().saturating_sub(1));
        let y = area.y + 1 + idx as u16;
        if y >= area.y + area.height - 1 {
            return None;
        }

        let (name, value) = self.path_params.get(idx)?;
        let param = self.endpoint.path_params.get(idx);
        let required = param.map(|p| p.required).unwrap_or(false);
        let param_type = param
            .map(|p| p.param_type.clone())
            .unwrap_or_else(|| "string".to_string());
        let marker = if required { "*" } else { " " };

        let line = format!("{marker} {name} ({param_type}) = {value}");
        let max_x = area.x + area.width - 2;
        let mut x = area.x + 1 + line.chars().count() as u16;
        if x > max_x {
            x = max_x;
        }

        Some((x, y))
    }

    fn cursor_in_query_params(&self, area: Rect) -> Option<(u16, u16)> {
        if area.width < 3 || area.height < 3 {
            return None;
        }

        let idx = self
            .param_input_idx
            .min(self.query_params.len().saturating_sub(1));
        let y = area.y + 1 + idx as u16;
        if y >= area.y + area.height - 1 {
            return None;
        }

        let (name, value) = self.query_params.get(idx)?;
        let param = self.endpoint.query_params.get(idx);
        let required = param.map(|p| p.required).unwrap_or(false);
        let param_type = param
            .map(|p| p.param_type.clone())
            .unwrap_or_else(|| "string".to_string());
        let marker = if required { "*" } else { " " };

        let line = format!("{marker} {name} ({param_type}) = {value}");
        let max_x = area.x + area.width - 2;
        let mut x = area.x + 1 + line.chars().count() as u16;
        if x > max_x {
            x = max_x;
        }

        Some((x, y))
    }

    fn cursor_in_body(&self, area: Rect) -> Option<(u16, u16)> {
        if area.width < 3 || area.height < 3 {
            return None;
        }

        let lines: Vec<&str> = self.body_text.split('\n').collect();
        let last_line = lines.last().copied().unwrap_or("");
        let line_idx = lines.len().saturating_sub(1);

        let max_visible_lines = (area.height - 2) as usize;
        let visible_line_idx = line_idx.min(max_visible_lines.saturating_sub(1));

        let y = area.y + 1 + visible_line_idx as u16;
        let max_x = area.x + area.width - 2;
        let mut x = area.x + 1 + last_line.chars().count() as u16;
        if x > max_x {
            x = max_x;
        }

        Some((x, y))
    }

    pub fn render(&self, f: &mut Frame, area: ratatui::layout::Rect) {
        let chunks = Layout::default()
            .constraints([
                Constraint::Length(3),
                Constraint::Min(5),
                Constraint::Length(3),
            ])
            .split(area);

        let info_text = format!("{} {}", self.endpoint.method, self.endpoint.path);
        let info = Paragraph::new(info_text)
            .block(Block::default().title("Endpoint").borders(Borders::ALL));
        f.render_widget(info, chunks[0]);

        let mid = self.middle_areas(chunks[1]);
        if let Some(pa) = mid.path {
            self.render_path_params(f, pa);
        }
        self.render_query_params(f, mid.query);
        if let Some(ba) = mid.body {
            self.render_body(f, ba);
        }

        let help_text = "Tab: Next field | Backspace: Delete | Enter: Execute | Esc: Back";
        let help = Paragraph::new(help_text).block(Block::default().borders(Borders::ALL));
        f.render_widget(help, chunks[2]);
    }

    fn render_path_params(&self, f: &mut Frame, area: Rect) {
        let items: Vec<String> = self
            .path_params
            .iter()
            .enumerate()
            .map(|(idx, (name, value))| {
                let param = self.endpoint.path_params.get(idx);
                let required = param.map(|p| p.required).unwrap_or(false);
                let param_type = param
                    .map(|p| p.param_type.clone())
                    .unwrap_or_else(|| "string".to_string());
                let marker = if required { "*" } else { " " };
                format!("{} {} ({}) = {}", marker, name, param_type, value)
            })
            .collect();

        let text = items.join("\n");
        let mut paragraph = Paragraph::new(text).block(
            Block::default()
                .title("Path parameters")
                .borders(Borders::ALL),
        );

        if self.focused_field == FocusedField::PathParams {
            paragraph =
                paragraph.style(ratatui::style::Style::default().fg(ratatui::style::Color::Yellow));
        }

        f.render_widget(paragraph, area);
    }

    fn render_query_params(&self, f: &mut Frame, area: Rect) {
        let items: Vec<String> = self
            .query_params
            .iter()
            .enumerate()
            .map(|(idx, (name, value))| {
                let param = self.endpoint.query_params.get(idx);
                let required = param.map(|p| p.required).unwrap_or(false);
                let param_type = param
                    .map(|p| p.param_type.clone())
                    .unwrap_or_else(|| "string".to_string());
                let marker = if required { "*" } else { " " };
                format!("{} {} ({}) = {}", marker, name, param_type, value)
            })
            .collect();

        let text = items.join("\n");
        let mut paragraph = Paragraph::new(text).block(
            Block::default()
                .title("Query Parameters")
                .borders(Borders::ALL),
        );

        if self.focused_field == FocusedField::QueryParams {
            paragraph =
                paragraph.style(ratatui::style::Style::default().fg(ratatui::style::Color::Yellow));
        }

        f.render_widget(paragraph, area);
    }

    fn render_body(&self, f: &mut Frame, area: Rect) {
        let mut paragraph = Paragraph::new(self.body_text.as_str())
            .block(
                Block::default()
                    .title("Request Body (JSON)")
                    .borders(Borders::ALL),
            )
            .wrap(ratatui::widgets::Wrap { trim: true });

        if self.focused_field == FocusedField::Body {
            paragraph =
                paragraph.style(ratatui::style::Style::default().fg(ratatui::style::Color::Yellow));
        }

        f.render_widget(paragraph, area);
    }
}
