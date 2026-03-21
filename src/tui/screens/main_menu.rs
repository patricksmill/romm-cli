use ratatui::widgets::{Block, Borders, List, ListItem, ListState};
use ratatui::Frame;

/// Simple main menu screen for choosing the high-level mode.
pub struct MainMenuScreen {
    pub selected: usize,
}

impl Default for MainMenuScreen {
    fn default() -> Self {
        Self::new()
    }
}

impl MainMenuScreen {
    pub fn new() -> Self {
        Self { selected: 0 }
    }

    pub fn next(&mut self) {
        self.selected = (self.selected + 1) % 6;
    }

    pub fn previous(&mut self) {
        self.selected = if self.selected == 0 {
            5
        } else {
            self.selected - 1
        };
    }

    pub fn render(&self, f: &mut Frame, area: ratatui::layout::Rect) {
        let items = vec![
            ListItem::new("Browse Games"),
            ListItem::new("Search"),
            ListItem::new("Downloads"),
            ListItem::new("Settings"),
            ListItem::new("API (Expert)"),
            ListItem::new("Exit"),
        ];

        let list = List::new(items)
            .block(Block::default().title("Game Library").borders(Borders::ALL))
            .highlight_symbol(">> ");

        let mut state = ListState::default();
        state.select(Some(self.selected));

        f.render_stateful_widget(list, area, &mut state);
    }
}
