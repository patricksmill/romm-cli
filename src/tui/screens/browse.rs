use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::widgets::{Block, Borders, List, ListItem, ListState};
use ratatui::Frame;

use crate::tui::openapi::{ApiEndpoint, EndpointRegistry};
use std::collections::HashMap;

/// Logical grouping of API endpoints (by tag or path prefix).
#[derive(Debug, Clone)]
pub struct Section {
    pub name: String,
    pub endpoint_indices: Vec<usize>,
}

/// API browser screen for exploring ROMM endpoints.
pub struct BrowseScreen {
    pub registry: EndpointRegistry,
    pub sections: Vec<Section>,
    pub selected_section: usize,
    pub selected_endpoint: usize,
    pub view_mode: ViewMode,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ViewMode {
    Sections,
    Endpoints,
}

impl BrowseScreen {
    pub fn new(registry: EndpointRegistry) -> Self {
        let mut sections_map: HashMap<String, Vec<usize>> = HashMap::new();

        for (idx, endpoint) in registry.endpoints.iter().enumerate() {
            let section_name = if !endpoint.tags.is_empty() {
                endpoint.tags[0].clone()
            } else {
                let path_parts: Vec<&str> =
                    endpoint.path.split('/').filter(|s| !s.is_empty()).collect();
                if path_parts.len() >= 2 {
                    path_parts[1].to_string()
                } else {
                    "Other".to_string()
                }
            };

            sections_map
                .entry(section_name)
                .or_insert_with(Vec::new)
                .push(idx);
        }

        let mut sections: Vec<Section> = sections_map
            .into_iter()
            .map(|(name, endpoint_indices)| Section {
                name,
                endpoint_indices,
            })
            .collect();

        sections.sort_by(|a, b| a.name.cmp(&b.name));

        Self {
            registry,
            sections,
            selected_section: 0,
            selected_endpoint: 0,
            view_mode: ViewMode::Sections,
        }
    }

    pub fn next(&mut self) {
        match self.view_mode {
            ViewMode::Sections => {
                if !self.sections.is_empty() {
                    self.selected_section = (self.selected_section + 1) % self.sections.len();
                    self.selected_endpoint = 0;
                }
            }
            ViewMode::Endpoints => {
                if let Some(section) = self.sections.get(self.selected_section) {
                    if !section.endpoint_indices.is_empty() {
                        self.selected_endpoint =
                            (self.selected_endpoint + 1) % section.endpoint_indices.len();
                    }
                }
            }
        }
    }

    pub fn previous(&mut self) {
        match self.view_mode {
            ViewMode::Sections => {
                if !self.sections.is_empty() {
                    self.selected_section = if self.selected_section == 0 {
                        self.sections.len() - 1
                    } else {
                        self.selected_section - 1
                    };
                    self.selected_endpoint = 0;
                }
            }
            ViewMode::Endpoints => {
                if let Some(section) = self.sections.get(self.selected_section) {
                    if !section.endpoint_indices.is_empty() {
                        self.selected_endpoint = if self.selected_endpoint == 0 {
                            section.endpoint_indices.len() - 1
                        } else {
                            self.selected_endpoint - 1
                        };
                    }
                }
            }
        }
    }

    pub fn switch_view(&mut self) {
        self.view_mode = match self.view_mode {
            ViewMode::Sections => ViewMode::Endpoints,
            ViewMode::Endpoints => ViewMode::Sections,
        };
        self.selected_endpoint = 0;
    }

    pub fn get_selected_endpoint(&self) -> Option<&ApiEndpoint> {
        self.sections
            .get(self.selected_section)
            .and_then(|section| section.endpoint_indices.get(self.selected_endpoint))
            .and_then(|&idx| self.registry.endpoints.get(idx))
    }

    pub fn render(&self, f: &mut Frame, area: ratatui::layout::Rect) {
        let chunks = Layout::default()
            .constraints([Constraint::Percentage(30), Constraint::Percentage(70)])
            .direction(ratatui::layout::Direction::Horizontal)
            .split(area);

        self.render_sections(f, chunks[0]);

        let endpoint_chunks = Layout::default()
            .constraints([Constraint::Min(5), Constraint::Length(3)])
            .direction(ratatui::layout::Direction::Vertical)
            .split(chunks[1]);

        self.render_endpoints(f, endpoint_chunks[0]);
        self.render_help(f, endpoint_chunks[1]);
    }

    fn render_help(&self, f: &mut Frame, area: Rect) {
        let help_text = match self.view_mode {
            ViewMode::Sections => "Tab/→: View endpoints | ↑↓: Navigate sections | Esc: Back",
            ViewMode::Endpoints => {
                "Tab/←: View sections | ↑↓: Navigate | Enter: Execute | Esc: Back"
            }
        };
        let help = ratatui::widgets::Paragraph::new(help_text)
            .block(ratatui::widgets::Block::default().borders(ratatui::widgets::Borders::ALL));
        f.render_widget(help, area);
    }

    fn render_sections(&self, f: &mut Frame, area: Rect) {
        let items: Vec<ListItem> = self
            .sections
            .iter()
            .enumerate()
            .map(|(idx, section)| {
                let count = section.endpoint_indices.len();
                let name = if idx == self.selected_section && self.view_mode == ViewMode::Sections {
                    format!("▶ {} ({})", section.name, count)
                } else {
                    format!("  {} ({})", section.name, count)
                };
                ListItem::new(name)
            })
            .collect();

        let list = List::new(items)
            .block(Block::default().title("Sections").borders(Borders::ALL))
            .highlight_symbol(if self.view_mode == ViewMode::Sections {
                ">> "
            } else {
                "   "
            });

        let mut state = ListState::default();
        if self.view_mode == ViewMode::Sections {
            state.select(Some(self.selected_section));
        }

        f.render_stateful_widget(list, area, &mut state);
    }

    fn render_endpoints(&self, f: &mut Frame, area: Rect) {
        let (items, count): (Vec<ListItem>, usize) =
            if let Some(section) = self.sections.get(self.selected_section) {
                let items: Vec<ListItem> = section
                    .endpoint_indices
                    .iter()
                    .enumerate()
                    .map(|(_idx, &endpoint_idx)| {
                        let ep = &self.registry.endpoints[endpoint_idx];
                        let method_color = match ep.method.as_str() {
                            "GET" => ratatui::style::Color::Green,
                            "POST" => ratatui::style::Color::Blue,
                            "PUT" => ratatui::style::Color::Yellow,
                            "DELETE" => ratatui::style::Color::Red,
                            _ => ratatui::style::Color::White,
                        };

                        let summary = ep
                            .summary
                            .as_ref()
                            .map(|s| format!(" - {}", s))
                            .unwrap_or_default();

                        ListItem::new(format!("{} {}{}", ep.method, ep.path, summary))
                            .style(ratatui::style::Style::default().fg(method_color))
                    })
                    .collect();
                let count = items.len();
                (items, count)
            } else {
                (vec![], 0)
            };

        let section_name = self
            .sections
            .get(self.selected_section)
            .map(|s| s.name.clone())
            .unwrap_or_else(|| "No Section".to_string());

        let list = List::new(items)
            .block(
                Block::default()
                    .title(format!("Endpoints: {} ({})", section_name, count))
                    .borders(Borders::ALL),
            )
            .highlight_symbol(if self.view_mode == ViewMode::Endpoints {
                ">> "
            } else {
                "   "
            });

        let mut state = ListState::default();
        if self.view_mode == ViewMode::Endpoints {
            state.select(Some(self.selected_endpoint));
        }

        f.render_stateful_widget(list, area, &mut state);
    }
}
