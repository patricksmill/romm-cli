//! Pick a metadata provider match for the current ROM.

use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::text::Line;
use ratatui::widgets::{List, ListItem, ListState, Paragraph};
use ratatui::Frame;

use crate::tui::footer_hint::{render_footer_panel, FooterHintEntry};
use crate::tui::theme::RommStyles;
use romm_api::types::metadata::SearchRom;

use super::game_detail::GameDetailScreen;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MetadataMatchPhase {
    Loading,
    Ready,
    Failed(String),
    Applying,
}

pub struct MetadataMatchScreen {
    pub previous: Box<GameDetailScreen>,
    pub phase: MetadataMatchPhase,
    pub rows: Vec<SearchRom>,
    pub selected: usize,
    pub message: Option<String>,
}

impl MetadataMatchScreen {
    pub fn new_loading(previous: Box<GameDetailScreen>) -> Self {
        Self {
            previous,
            phase: MetadataMatchPhase::Loading,
            rows: Vec::new(),
            selected: 0,
            message: Some("Searching metadata providers…".into()),
        }
    }

    pub fn apply_search_result(&mut self, rows: Vec<SearchRom>) {
        if rows.is_empty() {
            self.phase = MetadataMatchPhase::Failed("No matches found".into());
            self.message = Some("No metadata matches".into());
        } else {
            self.phase = MetadataMatchPhase::Ready;
            self.rows = rows;
            self.selected = 0;
            self.message = None;
        }
    }

    pub fn apply_search_error(&mut self, err: String) {
        self.phase = MetadataMatchPhase::Failed(err.clone());
        self.message = Some(err);
    }

    pub fn set_applying(&mut self) {
        self.phase = MetadataMatchPhase::Applying;
        self.message = Some("Applying match…".into());
    }

    pub fn move_up(&mut self) {
        if self.selected > 0 {
            self.selected -= 1;
        }
    }

    pub fn move_down(&mut self) {
        if self.selected + 1 < self.rows.len() {
            self.selected += 1;
        }
    }

    pub fn selected_match_fields(&self) -> Option<romm_api::types::metadata::RomMatchFields> {
        self.rows
            .get(self.selected)
            .map(|r| r.primary_match_fields())
    }

    pub fn render(&mut self, f: &mut Frame, area: Rect, styles: &RommStyles) {
        let chunks = Layout::default()
            .constraints([Constraint::Min(3), Constraint::Length(3)])
            .direction(Direction::Vertical)
            .split(area);

        let title = self.previous.rom.name.clone();
        let block = styles.panel_block(format!("Match metadata — {title}"));

        match &self.phase {
            MetadataMatchPhase::Loading | MetadataMatchPhase::Applying => {
                let msg = self.message.as_deref().unwrap_or("…");
                let p = Paragraph::new(msg).block(block);
                f.render_widget(p, chunks[0]);
            }
            MetadataMatchPhase::Failed(ref e) => {
                let p = Paragraph::new(e.as_str()).block(block);
                f.render_widget(p, chunks[0]);
            }
            MetadataMatchPhase::Ready => {
                let items: Vec<ListItem> = self
                    .rows
                    .iter()
                    .enumerate()
                    .map(|(i, r)| {
                        let line = format!(
                            "{}. {}  igdb:{:?} ss:{:?} moby:{:?}",
                            i + 1,
                            r.name,
                            r.igdb_id,
                            r.ss_id,
                            r.moby_id
                        );
                        ListItem::new(Line::from(line))
                    })
                    .collect();
                let mut state = ListState::default();
                state.select(Some(self.selected));
                let list = List::new(items)
                    .block(block)
                    .highlight_style(styles.selection());
                f.render_stateful_widget(list, chunks[0], &mut state);
            }
        }

        render_footer_panel(
            f,
            chunks[1],
            styles,
            &[
                FooterHintEntry {
                    key: "Enter",
                    label: "Apply match",
                },
                FooterHintEntry {
                    key: "Esc",
                    label: "Cancel",
                },
            ],
            self.message.as_deref(),
        );
    }
}

#[cfg(test)]
mod tests {
    use romm_api::types::metadata::SearchRom;
    use serde_json::json;

    fn fixture_row() -> SearchRom {
        serde_json::from_value(json!({
            "name": "Zelda",
            "platform_id": 1,
            "igdb_id": 5
        }))
        .expect("fixture")
    }

    #[test]
    fn selected_row_builds_match_fields() {
        let row = fixture_row();
        let fields = row.primary_match_fields();
        assert_eq!(fields.igdb_id, Some(5));
    }
}
