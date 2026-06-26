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
    QueryInput,
    Loading,
    Ready,
    Failed(String),
    Applying,
}

pub struct MetadataMatchScreen {
    pub previous: Box<GameDetailScreen>,
    pub phase: MetadataMatchPhase,
    pub search_query: String,
    pub cursor_pos: usize,
    pub rows: Vec<SearchRom>,
    pub selected: usize,
    pub message: Option<String>,
}

fn seed_search_query(rom: &romm_api::types::Rom) -> String {
    let no_tags = rom.fs_name_no_tags.trim();
    if !no_tags.is_empty() {
        no_tags.to_string()
    } else {
        rom.name.trim().to_string()
    }
}

impl MetadataMatchScreen {
    pub fn new_for_rom(previous: Box<GameDetailScreen>) -> Self {
        let search_query = seed_search_query(&previous.rom);
        let cursor_pos = search_query.len();
        Self {
            previous,
            phase: MetadataMatchPhase::QueryInput,
            search_query,
            cursor_pos,
            rows: Vec::new(),
            selected: 0,
            message: None,
        }
    }

    pub fn is_query_input(&self) -> bool {
        matches!(self.phase, MetadataMatchPhase::QueryInput)
    }

    pub fn set_loading(&mut self) {
        self.phase = MetadataMatchPhase::Loading;
        self.message = Some("Searching metadata providers…".into());
    }

    pub fn return_to_query_input(&mut self) {
        self.phase = MetadataMatchPhase::QueryInput;
        self.rows.clear();
        self.selected = 0;
        self.message = None;
    }

    pub fn add_char(&mut self, c: char) {
        let pos = self.cursor_pos.min(self.search_query.len());
        self.search_query.insert(pos, c);
        self.cursor_pos = pos + 1;
    }

    pub fn delete_char(&mut self) {
        if self.cursor_pos > 0 && self.cursor_pos <= self.search_query.len() {
            self.search_query.remove(self.cursor_pos - 1);
            self.cursor_pos -= 1;
        }
    }

    pub fn cursor_left(&mut self) {
        if self.cursor_pos > 0 {
            self.cursor_pos -= 1;
        }
    }

    pub fn cursor_right(&mut self) {
        if self.cursor_pos < self.search_query.len() {
            self.cursor_pos += 1;
        }
    }

    pub fn apply_search_result(&mut self, rows: Vec<SearchRom>) {
        if rows.is_empty() {
            let term = self.search_query.trim();
            let msg = if term.is_empty() {
                "No metadata matches found".to_string()
            } else {
                format!("No metadata matches for \"{term}\"")
            };
            self.phase = MetadataMatchPhase::Failed(msg.clone());
            self.message = Some(msg);
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

    pub fn selected_update_fields(&self) -> Option<romm_api::endpoints::roms::RomUpdateFields> {
        self.rows
            .get(self.selected)
            .map(romm_api::core::metadata::search_row_apply_fields)
    }

    fn footer_entries(&self) -> &'static [FooterHintEntry] {
        match self.phase {
            MetadataMatchPhase::QueryInput => &[
                FooterHintEntry {
                    key: "Enter",
                    label: "Search",
                },
                FooterHintEntry {
                    key: "Esc",
                    label: "Cancel",
                },
            ],
            MetadataMatchPhase::Ready => &[
                FooterHintEntry {
                    key: "Enter",
                    label: "Apply match",
                },
                FooterHintEntry {
                    key: "r",
                    label: "Edit search",
                },
                FooterHintEntry {
                    key: "Esc",
                    label: "Cancel",
                },
            ],
            MetadataMatchPhase::Failed(_) => &[
                FooterHintEntry {
                    key: "r",
                    label: "Edit search",
                },
                FooterHintEntry {
                    key: "Esc",
                    label: "Cancel",
                },
            ],
            MetadataMatchPhase::Loading | MetadataMatchPhase::Applying => &[],
        }
    }

    pub fn render(&mut self, f: &mut Frame, area: Rect, styles: &RommStyles) {
        let chunks = Layout::default()
            .constraints([Constraint::Min(3), Constraint::Length(3)])
            .direction(Direction::Vertical)
            .split(area);

        let title = self.previous.rom.name.clone();
        let block = styles.panel_block(format!("Match metadata — {title}"));

        match &self.phase {
            MetadataMatchPhase::QueryInput => {
                let input_line = format!("Search: {}", self.search_query);
                let p = Paragraph::new(input_line).style(styles.text()).block(block);
                f.render_widget(p, chunks[0]);
            }
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
            self.footer_entries(),
            self.message.as_deref(),
        );
    }
}

#[cfg(test)]
mod tests {
    use std::sync::{Arc, Mutex};

    use super::*;
    use crate::tui::screens::game_detail::GameDetailPrevious;
    use crate::tui::screens::SearchScreen;
    use romm_api::types::metadata::SearchRom;
    use romm_api::types::Rom;
    use serde_json::json;

    fn fixture_rom() -> Rom {
        Rom {
            id: 3850,
            platform_id: 1,
            platform_slug: None,
            platform_fs_slug: None,
            platform_custom_name: None,
            platform_display_name: None,
            fs_name: "3850 - LEGO Battles (US)(M3)(XenoPhobia).nds".into(),
            fs_name_no_tags: "3850 - LEGO Battles".into(),
            fs_name_no_ext: "3850 - LEGO Battles (US)(M3)(XenoPhobia)".into(),
            fs_extension: "nds".into(),
            fs_path: "/nds/3850 - LEGO Battles (US)(M3)(XenoPhobia).nds".into(),
            fs_size_bytes: 1,
            name: "3850 - LEGO Battles (US)(M3)(XenoPhobia).nds".into(),
            slug: None,
            summary: None,
            path_cover_small: None,
            path_cover_large: None,
            url_cover: None,
            has_manual: false,
            path_manual: None,
            url_manual: None,
            is_unidentified: true,
            is_identified: false,
            files: Vec::new(),
        }
    }

    fn fixture_screen() -> MetadataMatchScreen {
        let rom = fixture_rom();
        MetadataMatchScreen::new_for_rom(Box::new(GameDetailScreen::new(
            rom,
            vec![],
            GameDetailPrevious::Search(SearchScreen::new()),
            Arc::new(Mutex::new(vec![])),
            42,
        )))
    }

    fn fixture_row() -> SearchRom {
        serde_json::from_value(json!({
            "name": "Zelda",
            "platform_id": 1,
            "igdb_id": 5
        }))
        .expect("fixture")
    }

    #[test]
    fn opens_in_query_input_with_seeded_fs_name_no_tags() {
        let screen = fixture_screen();
        assert!(matches!(screen.phase, MetadataMatchPhase::QueryInput));
        assert_eq!(screen.search_query, "3850 - LEGO Battles");
    }

    #[test]
    fn enter_from_query_input_moves_to_loading() {
        let mut screen = fixture_screen();
        screen.search_query = "LEGO Battles".into();
        screen.cursor_pos = screen.search_query.len();
        screen.set_loading();
        assert!(matches!(screen.phase, MetadataMatchPhase::Loading));
    }

    #[test]
    fn query_input_typing_updates_text() {
        let mut screen = fixture_screen();
        screen.search_query.clear();
        screen.cursor_pos = 0;
        screen.add_char('A');
        assert_eq!(screen.search_query, "A");
        assert_eq!(screen.cursor_pos, 1);
    }

    #[test]
    fn selected_row_builds_update_fields() {
        let row = fixture_row();
        let mut screen = fixture_screen();
        screen.apply_search_result(vec![row]);
        let fields = screen.selected_update_fields().expect("fields");
        assert_eq!(fields.name.as_deref(), Some("Zelda"));
        assert_eq!(fields.match_fields.igdb_id, Some(5));
    }
}
