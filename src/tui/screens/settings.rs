use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::widgets::{Block, Borders, Paragraph};
use ratatui::Frame;

use crate::config::Config;

/// Read-only settings screen summarising current config and keybindings.
pub struct SettingsScreen {
    pub base_url: String,
    pub auth_status: String,
}

impl SettingsScreen {
    pub fn new(config: &Config) -> Self {
        let auth_status = match &config.auth {
            Some(crate::config::AuthConfig::Basic { username, .. }) => {
                format!("Basic (user: {})", username)
            }
            Some(crate::config::AuthConfig::Bearer { .. }) => "Bearer token".to_string(),
            Some(crate::config::AuthConfig::ApiKey { header, .. }) => {
                format!("API key (header: {})", header)
            }
            None => "None".to_string(),
        };

        Self {
            base_url: config.base_url.clone(),
            auth_status,
        }
    }

    pub fn render(&self, f: &mut Frame, area: Rect) {
        let chunks = Layout::default()
            .constraints([Constraint::Min(5), Constraint::Length(5)])
            .direction(ratatui::layout::Direction::Vertical)
            .split(area);

        let lines = [
            format!("Base URL: {}", self.base_url),
            format!("Auth: {}", self.auth_status),
            String::new(),
            "Change via environment: API_BASE_URL, API_USERNAME/API_PASSWORD,".to_string(),
            "API_TOKEN, or API_KEY + API_KEY_HEADER. Restart the app after changes.".to_string(),
        ];
        let text = lines.join("\n");
        let p =
            Paragraph::new(text).block(Block::default().title("Settings").borders(Borders::ALL));
        f.render_widget(p, chunks[0]);

        let help = "Esc: Back to menu";
        let p = Paragraph::new(help).block(Block::default().borders(Borders::ALL));
        f.render_widget(p, chunks[1]);
    }
}
