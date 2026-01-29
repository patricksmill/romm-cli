use ratatui::widgets::{Block, Borders, List, ListItem, ListState};
use ratatui::Frame;

pub struct MainMenuScreen {
    pub selected: usize,
}

impl MainMenuScreen {
    pub fn new() -> Self {
        Self { selected: 0 }
    }

    pub fn next(&mut self) {
        self.selected = (self.selected + 1) % 3;
    }

    pub fn previous(&mut self) {
        self.selected = if self.selected == 0 { 2 } else { self.selected - 1 };
    }

    pub fn render(&self, f: &mut Frame, area: ratatui::layout::Rect) {
        let items = vec![
            ListItem::new("Browse Endpoints"),
            ListItem::new("Search Endpoint"),
            ListItem::new("Exit"),
        ];

        let list = List::new(items)
            .block(Block::default().title("ROMM API Explorer").borders(Borders::ALL))
            .highlight_symbol(">> ");

        let mut state = ListState::default();
        state.select(Some(self.selected));

        f.render_stateful_widget(list, area, &mut state);
    }
}
