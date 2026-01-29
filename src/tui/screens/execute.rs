use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::widgets::{Block, Borders, Paragraph};
use ratatui::Frame;

use crate::tui::openapi::ApiEndpoint;

pub struct ExecuteScreen {
    pub endpoint: ApiEndpoint,
    pub query_params: Vec<(String, String)>,
    pub body_text: String,
    pub focused_field: FocusedField,
    pub param_input_idx: usize,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum FocusedField {
    QueryParams,
    Body,
}

impl ExecuteScreen {
    pub fn new(endpoint: ApiEndpoint) -> Self {
        let query_params: Vec<(String, String)> = endpoint
            .query_params
            .iter()
            .map(|p| {
                (
                    p.name.clone(),
                    p.default.clone().unwrap_or_default(),
                )
            })
            .collect();

        Self {
            endpoint,
            query_params,
            body_text: String::new(),
            focused_field: FocusedField::QueryParams,
            param_input_idx: 0,
        }
    }

    pub fn update_query_param(&mut self, value: String) {
        if let Some((_, ref mut v)) = self.query_params.get_mut(self.param_input_idx) {
            *v = value;
        }
    }

    pub fn get_query_params(&self) -> Vec<(String, String)> {
        self.query_params
            .iter()
            .filter(|(_, v)| !v.is_empty())
            .cloned()
            .collect()
    }

    pub fn next_field(&mut self) {
        match self.focused_field {
            FocusedField::QueryParams => {
                if self.param_input_idx + 1 < self.query_params.len() {
                    self.param_input_idx += 1;
                } else if self.endpoint.has_body {
                    self.focused_field = FocusedField::Body;
                }
            }
            FocusedField::Body => {
                self.focused_field = FocusedField::QueryParams;
                self.param_input_idx = 0;
            }
        }
    }

    pub fn previous_field(&mut self) {
        match self.focused_field {
            FocusedField::QueryParams => {
                if self.param_input_idx > 0 {
                    self.param_input_idx -= 1;
                } else if self.endpoint.has_body {
                    self.focused_field = FocusedField::Body;
                }
            }
            FocusedField::Body => {
                self.focused_field = FocusedField::QueryParams;
                self.param_input_idx = self.query_params.len().saturating_sub(1);
            }
        }
    }

    pub fn add_char_to_focused(&mut self, c: char) {
        match self.focused_field {
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

        // content areas (inside borders) start at +1,+1
        if self.endpoint.has_body {
            let query_chunks = Layout::default()
                .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
                .direction(ratatui::layout::Direction::Horizontal)
                .split(chunks[1]);

            let query_area = query_chunks[0];
            let body_area = query_chunks[1];

            match self.focused_field {
                FocusedField::QueryParams => self.cursor_in_query_params(query_area),
                FocusedField::Body => self.cursor_in_body(body_area),
            }
        } else {
            let query_area = chunks[1];
            self.cursor_in_query_params(query_area)
        }
    }

    fn cursor_in_query_params(&self, area: Rect) -> Option<(u16, u16)> {
        if area.width < 3 || area.height < 3 {
            return None;
        }

        let idx = self.param_input_idx.min(self.query_params.len().saturating_sub(1));
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
        let max_x = area.x + area.width - 2; // inside right border
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

        // Clamp to visible area (ignores wrapping; good enough for a visible caret)
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

        let info_text = format!(
            "{} {}",
            self.endpoint.method,
            self.endpoint.path
        );
        let info = Paragraph::new(info_text)
            .block(Block::default().title("Endpoint").borders(Borders::ALL));
        f.render_widget(info, chunks[0]);

        let _query_area = if self.endpoint.has_body {
            let query_chunks = Layout::default()
                .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
                .direction(ratatui::layout::Direction::Horizontal)
                .split(chunks[1]);

            self.render_query_params(f, query_chunks[0]);
            self.render_body(f, query_chunks[1]);
            chunks[1]
        } else {
            self.render_query_params(f, chunks[1]);
            chunks[1]
        };

        let help_text = "Tab: Next field | Backspace: Delete | Enter: Execute | Esc: Back";
        let help = Paragraph::new(help_text)
            .block(Block::default().borders(Borders::ALL));
        f.render_widget(help, chunks[2]);
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
                    .and_then(|p| Some(p.param_type.clone()))
                    .unwrap_or_else(|| "string".to_string());
                let marker = if required { "*" } else { " " };
                format!("{} {} ({}) = {}", marker, name, param_type, value)
            })
            .collect();

        let text = items.join("\n");
        let mut paragraph = Paragraph::new(text)
            .block(Block::default().title("Query Parameters").borders(Borders::ALL));

        if self.focused_field == FocusedField::QueryParams {
            paragraph = paragraph.style(ratatui::style::Style::default().fg(ratatui::style::Color::Yellow));
        }

        f.render_widget(paragraph, area);
    }

    fn render_body(&self, f: &mut Frame, area: Rect) {
        let mut paragraph = Paragraph::new(self.body_text.as_str())
            .block(Block::default().title("Request Body (JSON)").borders(Borders::ALL))
            .wrap(ratatui::widgets::Wrap { trim: true });

        if self.focused_field == FocusedField::Body {
            paragraph = paragraph.style(ratatui::style::Style::default().fg(ratatui::style::Color::Yellow));
        }

        f.render_widget(paragraph, area);
    }
}
