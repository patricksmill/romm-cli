//! First-run setup: server URL, download directory, authentication, test connection, persist config.

use anyhow::{anyhow, Result};
use crossterm::event::{
    self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEventKind,
};
use crossterm::execute;
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use ratatui::backend::CrosstermBackend;
use ratatui::layout::{Constraint, Direction, Layout};
use ratatui::style::{Color, Modifier, Style};
use ratatui::widgets::{Block, Borders, List, ListItem, ListState, Paragraph};
use ratatui::Terminal;
use std::io::stdout;

use crate::client::RommClient;
use crate::config::{
    load_config, load_layered_env, normalize_romm_origin, persist_user_config, AuthConfig, Config,
};

#[derive(Clone, Copy, PartialEq, Eq)]
enum AuthKind {
    None,
    Basic,
    Bearer,
    ApiKey,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Step {
    Url,
    Download,
    AuthMenu,
    BasicUser,
    BasicPass,
    Bearer,
    ApiHeader,
    ApiKey,
    Summary,
}

/// Interactive setup run before the main TUI when `API_BASE_URL` is missing.
pub struct SetupWizard {
    step: Step,
    auth_kind: AuthKind,
    auth_menu_selected: usize,
    url: String,
    url_cursor: usize,
    download_dir: String,
    dl_cursor: usize,
    username: String,
    user_cursor: usize,
    password: String,
    bearer_token: String,
    bearer_cursor: usize,
    api_header: String,
    header_cursor: usize,
    api_key: String,
    api_key_cursor: usize,
    testing: bool,
    error: Option<String>,
}

impl SetupWizard {
    pub fn new() -> Self {
        let default_dl = dirs::download_dir()
            .unwrap_or_else(|| dirs::home_dir().unwrap_or_default().join("Downloads"))
            .join("romm-cli")
            .display()
            .to_string();
        Self {
            step: Step::Url,
            auth_kind: AuthKind::None,
            auth_menu_selected: 0,
            url: "https://".to_string(),
            url_cursor: "https://".len(),
            download_dir: default_dl,
            dl_cursor: 0,
            username: String::new(),
            user_cursor: 0,
            password: String::new(),
            bearer_token: String::new(),
            bearer_cursor: 0,
            api_header: String::new(),
            header_cursor: 0,
            api_key: String::new(),
            api_key_cursor: 0,
            testing: false,
            error: None,
        }
    }

    fn auth_labels() -> [&'static str; 4] {
        [
            "No authentication",
            "Basic (username + password)",
            "Bearer token",
            "API key in custom header",
        ]
    }

    fn auth_kind_from_index(i: usize) -> AuthKind {
        match i {
            1 => AuthKind::Basic,
            2 => AuthKind::Bearer,
            3 => AuthKind::ApiKey,
            _ => AuthKind::None,
        }
    }

    fn build_config(&self) -> Result<Config> {
        let base_url = normalize_romm_origin(self.url.trim());
        if base_url.is_empty() {
            return Err(anyhow!("Server URL cannot be empty"));
        }
        let auth: Option<AuthConfig> = match self.auth_kind {
            AuthKind::None => None,
            AuthKind::Basic => {
                let u = self.username.trim();
                if u.is_empty() {
                    return Err(anyhow!("Username cannot be empty"));
                }
                if self.password.is_empty() {
                    return Err(anyhow!("Password cannot be empty"));
                }
                Some(AuthConfig::Basic {
                    username: u.to_string(),
                    password: self.password.clone(),
                })
            }
            AuthKind::Bearer => {
                if self.bearer_token.trim().is_empty() {
                    return Err(anyhow!("Bearer token cannot be empty"));
                }
                Some(AuthConfig::Bearer {
                    token: self.bearer_token.trim().to_string(),
                })
            }
            AuthKind::ApiKey => {
                let h = self.api_header.trim();
                if h.is_empty() {
                    return Err(anyhow!("Header name cannot be empty"));
                }
                if self.api_key.is_empty() {
                    return Err(anyhow!("API key cannot be empty"));
                }
                Some(AuthConfig::ApiKey {
                    header: h.to_string(),
                    key: self.api_key.clone(),
                })
            }
        };
        Ok(Config { base_url, auth })
    }

    fn render(&mut self, f: &mut ratatui::Frame, area: ratatui::layout::Rect) {
        let title = match self.step {
            Step::Url => "Step 1/4 — RomM server URL",
            Step::Download => "Step 2/4 — Download directory",
            Step::AuthMenu => "Step 3/4 — Authentication",
            Step::BasicUser | Step::BasicPass => "Step 4/4 — Basic auth",
            Step::Bearer => "Step 4/4 — Bearer token",
            Step::ApiHeader | Step::ApiKey => "Step 4/4 — API key",
            Step::Summary => "Review & connect",
        };

        let main = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),
                Constraint::Min(6),
                Constraint::Length(4),
            ])
            .split(area);

        let hint_top = "Same origin as in your browser (no trailing /api). Esc: quit";
        let p = Paragraph::new(hint_top).style(Style::default().fg(Color::DarkGray));
        f.render_widget(p, main[0]);

        match self.step {
            Step::Url => {
                let line = format!(
                    "{}▏",
                    self.url.chars().take(self.url_cursor).collect::<String>()
                );
                let rest: String = self.url.chars().skip(self.url_cursor).collect();
                let text = format!("{line}{rest}");
                let block = Block::default().title(title).borders(Borders::ALL);
                let p = Paragraph::new(text).block(block);
                f.render_widget(p, main[1]);
            }
            Step::Download => {
                let line = format!(
                    "{}▏",
                    self.download_dir
                        .chars()
                        .take(self.dl_cursor)
                        .collect::<String>()
                );
                let rest: String = self.download_dir.chars().skip(self.dl_cursor).collect();
                let text = format!("{line}{rest}");
                let block = Block::default().title(title).borders(Borders::ALL);
                let p = Paragraph::new(text).block(block);
                f.render_widget(p, main[1]);
            }
            Step::AuthMenu => {
                let items: Vec<ListItem> = Self::auth_labels()
                    .iter()
                    .map(|s| ListItem::new(*s))
                    .collect();
                let mut state = ListState::default();
                state.select(Some(self.auth_menu_selected));
                let list = List::new(items)
                    .block(Block::default().title(title).borders(Borders::ALL))
                    .highlight_style(
                        Style::default()
                            .fg(Color::Yellow)
                            .add_modifier(Modifier::BOLD),
                    )
                    .highlight_symbol(">> ");
                f.render_stateful_widget(list, main[1], &mut state);
            }
            Step::BasicUser | Step::BasicPass => {
                let user_line = if self.step == Step::BasicUser {
                    format!(
                        "{}▏{}",
                        self.username
                            .chars()
                            .take(self.user_cursor)
                            .collect::<String>(),
                        self.username
                            .chars()
                            .skip(self.user_cursor)
                            .collect::<String>()
                    )
                } else {
                    self.username.clone()
                };
                let pass_display: String = if self.step == Step::BasicPass {
                    "•".repeat(self.password.len()) + "▏"
                } else {
                    "•".repeat(self.password.len())
                };
                let block = Block::default().title(title).borders(Borders::ALL);
                let body = format!("Username\n{user_line}\n\nPassword (hidden)\n{pass_display}\n\nTab: switch field");
                let p = Paragraph::new(body).block(block);
                f.render_widget(p, main[1]);
            }
            Step::Bearer => {
                let line = format!(
                    "{}▏{}",
                    self.bearer_token
                        .chars()
                        .take(self.bearer_cursor)
                        .collect::<String>(),
                    self.bearer_token
                        .chars()
                        .skip(self.bearer_cursor)
                        .collect::<String>()
                );
                let block = Block::default().title(title).borders(Borders::ALL);
                let p = Paragraph::new(line).block(block);
                f.render_widget(p, main[1]);
            }
            Step::ApiHeader | Step::ApiKey => {
                let header_line = if self.step == Step::ApiHeader {
                    format!(
                        "{}▏{}",
                        self.api_header
                            .chars()
                            .take(self.header_cursor)
                            .collect::<String>(),
                        self.api_header
                            .chars()
                            .skip(self.header_cursor)
                            .collect::<String>()
                    )
                } else {
                    self.api_header.clone()
                };
                let key_line = if self.step == Step::ApiKey {
                    "•".repeat(self.api_key.len()) + "▏"
                } else {
                    "•".repeat(self.api_key.len())
                };
                let body = format!(
                    "Header name\n{header_line}\n\nKey (hidden)\n{key_line}\n\nTab: switch field"
                );
                let block = Block::default().title(title).borders(Borders::ALL);
                let p = Paragraph::new(body).block(block);
                f.render_widget(p, main[1]);
            }
            Step::Summary => {
                let url_line = normalize_romm_origin(self.url.trim());
                let auth_desc = match self.auth_kind {
                    AuthKind::None => "None",
                    AuthKind::Basic => "Basic",
                    AuthKind::Bearer => "Bearer",
                    AuthKind::ApiKey => "API key header",
                };
                let mut lines = vec![
                    format!("Server: {url_line}"),
                    format!("Downloads: {}", self.download_dir.trim()),
                    format!("Auth: {auth_desc}"),
                    String::new(),
                ];
                if self.testing {
                    lines.push("Connecting to server…".to_string());
                } else if let Some(ref e) = self.error {
                    lines.push(format!("Last error: {e}"));
                } else {
                    lines.push("Enter: test connection and save   Esc: quit".to_string());
                }
                let block = Block::default().title(title).borders(Borders::ALL);
                let p = Paragraph::new(lines.join("\n")).block(block);
                f.render_widget(p, main[1]);
            }
        }

        let footer = match self.step {
            Step::Url => "Enter: next   Backspace: delete   Esc: quit",
            Step::Download => "Enter: next   Backspace: delete   Esc: quit",
            Step::AuthMenu => "↑/↓: choose   Enter: next   Esc: quit",
            Step::BasicUser | Step::BasicPass => {
                "Type text   Tab: switch field   Enter: next step   Esc: quit"
            }
            Step::Bearer => "Enter: next step   Esc: quit",
            Step::ApiHeader | Step::ApiKey => "Tab: switch field   Enter: next step   Esc: quit",
            Step::Summary => {
                if self.testing {
                    "Please wait…"
                } else {
                    "Enter: connect & save"
                }
            }
        };
        let p = Paragraph::new(footer)
            .style(Style::default().fg(Color::Cyan))
            .block(Block::default().borders(Borders::ALL));
        f.render_widget(p, main[2]);
    }

    fn cursor_pos(&self, area: ratatui::layout::Rect) -> Option<(u16, u16)> {
        let main = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),
                Constraint::Min(6),
                Constraint::Length(4),
            ])
            .split(area);
        let inner = main[1];
        match self.step {
            Step::Url => {
                let x = inner.x + 1 + self.url_cursor.min(self.url.len()) as u16;
                Some((x, inner.y + 1))
            }
            Step::Download => {
                let x = inner.x + 1 + self.dl_cursor.min(self.download_dir.len()) as u16;
                Some((x, inner.y + 1))
            }
            Step::Bearer => {
                let x = inner.x + 1 + self.bearer_cursor.min(self.bearer_token.len()) as u16;
                Some((x, inner.y + 1))
            }
            Step::BasicUser => {
                let x = inner.x + 1 + self.user_cursor.min(self.username.len()) as u16;
                Some((x, inner.y + 2))
            }
            Step::BasicPass => {
                let x = inner.x + 1 + "•".repeat(self.password.len()).len() as u16;
                Some((x, inner.y + 6))
            }
            Step::ApiHeader => {
                let x = inner.x + 1 + self.header_cursor.min(self.api_header.len()) as u16;
                Some((x, inner.y + 2))
            }
            Step::ApiKey => {
                let x = inner.x + 1 + self.api_key_cursor.min(self.api_key.len()) as u16;
                Some((x, inner.y + 6))
            }
            _ => None,
        }
    }

    fn add_char_url(&mut self, c: char) {
        let pos = self.url_cursor.min(self.url.len());
        self.url.insert(pos, c);
        self.url_cursor = pos + 1;
    }

    fn del_char_url(&mut self) {
        if self.url_cursor > 0 && self.url_cursor <= self.url.len() {
            self.url.remove(self.url_cursor - 1);
            self.url_cursor -= 1;
        }
    }

    fn add_char_dl(&mut self, c: char) {
        let pos = self.dl_cursor.min(self.download_dir.len());
        self.download_dir.insert(pos, c);
        self.dl_cursor = pos + 1;
    }

    fn del_char_dl(&mut self) {
        if self.dl_cursor > 0 && self.dl_cursor <= self.download_dir.len() {
            self.download_dir.remove(self.dl_cursor - 1);
            self.dl_cursor -= 1;
        }
    }

    fn advance_from_auth_menu(&mut self) {
        self.auth_kind = Self::auth_kind_from_index(self.auth_menu_selected);
        self.step = match self.auth_kind {
            AuthKind::None => Step::Summary,
            AuthKind::Basic => Step::BasicUser,
            AuthKind::Bearer => Step::Bearer,
            AuthKind::ApiKey => Step::ApiHeader,
        };
    }

    fn advance_step(&mut self) -> Result<()> {
        self.error = None;
        match self.step {
            Step::Url => {
                if normalize_romm_origin(self.url.trim()).is_empty() {
                    self.error = Some("Enter a valid server URL".to_string());
                    return Ok(());
                }
                self.step = Step::Download;
                self.dl_cursor = self.download_dir.len();
            }
            Step::Download => {
                if self.download_dir.trim().is_empty() {
                    self.error = Some("Download path cannot be empty".to_string());
                    return Ok(());
                }
                self.step = Step::AuthMenu;
            }
            Step::AuthMenu => self.advance_from_auth_menu(),
            Step::BasicUser => self.step = Step::BasicPass,
            Step::BasicPass => self.step = Step::Summary,
            Step::Bearer => self.step = Step::Summary,
            Step::ApiHeader => self.step = Step::ApiKey,
            Step::ApiKey => self.step = Step::Summary,
            Step::Summary => {}
        }
        Ok(())
    }

    async fn try_connect_and_persist(&mut self, verbose: bool) -> Result<Config> {
        let cfg = self.build_config()?;
        let client = RommClient::new(&cfg, verbose)?;
        client.fetch_openapi_json().await?;
        let base = cfg.base_url.clone();
        let download = self.download_dir.trim().to_string();
        persist_user_config(&base, &download, cfg.auth.clone())?;
        load_layered_env();
        load_config()
    }

    pub async fn run(mut self, verbose: bool) -> Result<Config> {
        enable_raw_mode()?;
        let mut stdout = stdout();
        execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
        let backend = CrosstermBackend::new(stdout);
        let mut terminal = Terminal::new(backend)?;

        loop {
            terminal.draw(|f| {
                let area = f.size();
                self.render(f, area);
                if let Some((x, y)) = self.cursor_pos(area) {
                    f.set_cursor(x, y);
                }
            })?;

            if event::poll(std::time::Duration::from_millis(100))? {
                if let Event::Key(key) = event::read()? {
                    if key.kind != KeyEventKind::Press {
                        continue;
                    }
                    if key.code == KeyCode::Esc {
                        disable_raw_mode()?;
                        execute!(
                            terminal.backend_mut(),
                            LeaveAlternateScreen,
                            DisableMouseCapture
                        )?;
                        terminal.show_cursor()?;
                        return Err(anyhow!("setup cancelled"));
                    }

                    if self.testing {
                        continue;
                    }

                    match self.step {
                        Step::Url => match key.code {
                            KeyCode::Enter => {
                                let _ = self.advance_step();
                            }
                            KeyCode::Char(c) => self.add_char_url(c),
                            KeyCode::Backspace => self.del_char_url(),
                            KeyCode::Left => {
                                if self.url_cursor > 0 {
                                    self.url_cursor -= 1;
                                }
                            }
                            KeyCode::Right => {
                                if self.url_cursor < self.url.len() {
                                    self.url_cursor += 1;
                                }
                            }
                            _ => {}
                        },
                        Step::Download => match key.code {
                            KeyCode::Enter => {
                                let _ = self.advance_step();
                            }
                            KeyCode::Char(c) => self.add_char_dl(c),
                            KeyCode::Backspace => self.del_char_dl(),
                            KeyCode::Left => {
                                if self.dl_cursor > 0 {
                                    self.dl_cursor -= 1;
                                }
                            }
                            KeyCode::Right => {
                                if self.dl_cursor < self.download_dir.len() {
                                    self.dl_cursor += 1;
                                }
                            }
                            _ => {}
                        },
                        Step::AuthMenu => match key.code {
                            KeyCode::Up | KeyCode::Char('k') => {
                                if self.auth_menu_selected > 0 {
                                    self.auth_menu_selected -= 1;
                                }
                            }
                            KeyCode::Down | KeyCode::Char('j') => {
                                if self.auth_menu_selected < 3 {
                                    self.auth_menu_selected += 1;
                                }
                            }
                            KeyCode::Enter => {
                                let _ = self.advance_step();
                            }
                            _ => {}
                        },
                        Step::BasicUser => match key.code {
                            KeyCode::Tab => self.step = Step::BasicPass,
                            KeyCode::Enter => {
                                let _ = self.advance_step();
                            }
                            KeyCode::Char(c) => {
                                let pos = self.user_cursor.min(self.username.len());
                                self.username.insert(pos, c);
                                self.user_cursor = pos + 1;
                            }
                            KeyCode::Backspace => {
                                if self.user_cursor > 0 && self.user_cursor <= self.username.len() {
                                    self.username.remove(self.user_cursor - 1);
                                    self.user_cursor -= 1;
                                }
                            }
                            KeyCode::Left => {
                                if self.user_cursor > 0 {
                                    self.user_cursor -= 1;
                                }
                            }
                            KeyCode::Right => {
                                if self.user_cursor < self.username.len() {
                                    self.user_cursor += 1;
                                }
                            }
                            _ => {}
                        },
                        Step::BasicPass => match key.code {
                            KeyCode::Tab => self.step = Step::BasicUser,
                            KeyCode::Enter => {
                                let _ = self.advance_step();
                            }
                            KeyCode::Char(c) => self.password.push(c),
                            KeyCode::Backspace => {
                                self.password.pop();
                            }
                            _ => {}
                        },
                        Step::Bearer => match key.code {
                            KeyCode::Enter => {
                                let _ = self.advance_step();
                            }
                            KeyCode::Char(c) => {
                                let pos = self.bearer_cursor.min(self.bearer_token.len());
                                self.bearer_token.insert(pos, c);
                                self.bearer_cursor = pos + 1;
                            }
                            KeyCode::Backspace => {
                                if self.bearer_cursor > 0
                                    && self.bearer_cursor <= self.bearer_token.len()
                                {
                                    self.bearer_token.remove(self.bearer_cursor - 1);
                                    self.bearer_cursor -= 1;
                                }
                            }
                            KeyCode::Left => {
                                if self.bearer_cursor > 0 {
                                    self.bearer_cursor -= 1;
                                }
                            }
                            KeyCode::Right => {
                                if self.bearer_cursor < self.bearer_token.len() {
                                    self.bearer_cursor += 1;
                                }
                            }
                            _ => {}
                        },
                        Step::ApiHeader => match key.code {
                            KeyCode::Tab => self.step = Step::ApiKey,
                            KeyCode::Enter => {
                                let _ = self.advance_step();
                            }
                            KeyCode::Char(c) => {
                                let pos = self.header_cursor.min(self.api_header.len());
                                self.api_header.insert(pos, c);
                                self.header_cursor = pos + 1;
                            }
                            KeyCode::Backspace => {
                                if self.header_cursor > 0
                                    && self.header_cursor <= self.api_header.len()
                                {
                                    self.api_header.remove(self.header_cursor - 1);
                                    self.header_cursor -= 1;
                                }
                            }
                            KeyCode::Left => {
                                if self.header_cursor > 0 {
                                    self.header_cursor -= 1;
                                }
                            }
                            KeyCode::Right => {
                                if self.header_cursor < self.api_header.len() {
                                    self.header_cursor += 1;
                                }
                            }
                            _ => {}
                        },
                        Step::ApiKey => match key.code {
                            KeyCode::Tab => self.step = Step::ApiHeader,
                            KeyCode::Enter => {
                                let _ = self.advance_step();
                            }
                            KeyCode::Char(c) => {
                                let pos = self.api_key_cursor.min(self.api_key.len());
                                self.api_key.insert(pos, c);
                                self.api_key_cursor = pos + 1;
                            }
                            KeyCode::Backspace => {
                                if self.api_key_cursor > 0
                                    && self.api_key_cursor <= self.api_key.len()
                                {
                                    self.api_key.remove(self.api_key_cursor - 1);
                                    self.api_key_cursor -= 1;
                                }
                            }
                            KeyCode::Left => {
                                if self.api_key_cursor > 0 {
                                    self.api_key_cursor -= 1;
                                }
                            }
                            KeyCode::Right => {
                                if self.api_key_cursor < self.api_key.len() {
                                    self.api_key_cursor += 1;
                                }
                            }
                            _ => {}
                        },
                        Step::Summary => {
                            if key.code == KeyCode::Enter {
                                self.testing = true;
                                self.error = None;
                                terminal.draw(|f| {
                                    let area = f.size();
                                    self.render(f, area);
                                })?;
                                let result = self.try_connect_and_persist(verbose).await;
                                self.testing = false;
                                match result {
                                    Ok(cfg) => {
                                        disable_raw_mode()?;
                                        execute!(
                                            terminal.backend_mut(),
                                            LeaveAlternateScreen,
                                            DisableMouseCapture
                                        )?;
                                        terminal.show_cursor()?;
                                        return Ok(cfg);
                                    }
                                    Err(e) => {
                                        self.error = Some(format!("{e:#}"));
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

impl Default for SetupWizard {
    fn default() -> Self {
        Self::new()
    }
}
