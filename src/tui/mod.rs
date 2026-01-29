pub mod openapi;
pub mod screens;
pub mod utils;

use anyhow::{anyhow, Result};
use crossterm::event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEventKind};
use crossterm::execute;
use crossterm::terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen};
use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;

use crate::client::RommClient;

use self::screens::{BrowseScreen, ExecuteScreen, MainMenuScreen, ResultScreen};

pub enum AppScreen {
    MainMenu(MainMenuScreen),
    Browse(BrowseScreen),
    Execute(ExecuteScreen),
    Result(ResultScreen),
}

pub struct App {
    pub screen: AppScreen,
    pub client: RommClient,
    pub registry: openapi::EndpointRegistry,
    pub should_quit: bool,
}

impl App {
    pub fn new(client: RommClient, registry: openapi::EndpointRegistry) -> Self {
        Self {
            screen: AppScreen::MainMenu(MainMenuScreen::new()),
            client,
            registry,
            should_quit: false,
        }
    }

    pub async fn run(&mut self) -> Result<()> {
        enable_raw_mode()?;
        let mut stdout = std::io::stdout();
        execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
        let backend = CrosstermBackend::new(stdout);
        let mut terminal = Terminal::new(backend)?;

        loop {
            terminal.draw(|f| self.render(f))?;

            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    if self.handle_key(key.code).await? {
                        break;
                    }
                }
            }

            if self.should_quit {
                break;
            }
        }

        disable_raw_mode()?;
        execute!(
            terminal.backend_mut(),
            LeaveAlternateScreen,
            DisableMouseCapture
        )?;
        terminal.show_cursor()?;

        Ok(())
    }

    async fn handle_key(&mut self, key: KeyCode) -> Result<bool> {
        match &mut self.screen {
            AppScreen::MainMenu(menu) => {
                match key {
                    KeyCode::Up | KeyCode::Char('k') => menu.previous(),
                    KeyCode::Down | KeyCode::Char('j') => menu.next(),
                    KeyCode::Enter => {
                        match menu.selected {
                            0 => {
                                self.screen = AppScreen::Browse(BrowseScreen::new(self.registry.clone()));
                            }
                            1 => {
                                return Ok(false);
                            }
                            2 => return Ok(true),
                            _ => {}
                        }
                    }
                    KeyCode::Esc | KeyCode::Char('q') => return Ok(true),
                    _ => {}
                }
            }
            AppScreen::Browse(browse) => {
                match key {
                    KeyCode::Up | KeyCode::Char('k') => browse.previous(),
                    KeyCode::Down | KeyCode::Char('j') => browse.next(),
                    KeyCode::Left | KeyCode::Char('h') => {
                        if browse.view_mode == screens::browse::ViewMode::Endpoints {
                            browse.switch_view();
                        }
                    }
                    KeyCode::Right | KeyCode::Char('l') => {
                        if browse.view_mode == screens::browse::ViewMode::Sections {
                            browse.switch_view();
                        }
                    }
                    KeyCode::Tab => browse.switch_view(),
                    KeyCode::Enter => {
                        if browse.view_mode == screens::browse::ViewMode::Endpoints {
                            if let Some(endpoint) = browse.get_selected_endpoint() {
                                let endpoint_clone = endpoint.clone();
                                self.screen = AppScreen::Execute(ExecuteScreen::new(endpoint_clone));
                            }
                        } else {
                            browse.switch_view();
                        }
                    }
                    KeyCode::Esc => {
                        self.screen = AppScreen::MainMenu(MainMenuScreen::new());
                    }
                    _ => {}
                }
            }
            AppScreen::Execute(execute) => {
                match key {
                    KeyCode::Tab => execute.next_field(),
                    KeyCode::BackTab => execute.previous_field(),
                    KeyCode::Char(c) => execute.add_char_to_focused(c),
                    KeyCode::Backspace => execute.delete_char_from_focused(),
                    KeyCode::Enter => {
                        let endpoint = execute.endpoint.clone();
                        let query = execute.get_query_params();
                        let body = if execute.endpoint.has_body && !execute.body_text.is_empty() {
                            Some(serde_json::from_str(&execute.body_text)?)
                        } else {
                            None
                        };

                        match self
                            .client
                            .request_json(&endpoint.method, &endpoint.path, &query, body)
                            .await
                        {
                            Ok(result) => {
                                self.screen = AppScreen::Result(ResultScreen::new(result));
                            }
                            Err(e) => {
                                let error_json = serde_json::json!({
                                    "error": format!("{}", e)
                                });
                                self.screen = AppScreen::Result(ResultScreen::new(error_json));
                            }
                        }
                    }
                    KeyCode::Esc => {
                        self.screen = AppScreen::Browse(BrowseScreen::new(self.registry.clone()));
                    }
                    _ => {}
                }
            }
            AppScreen::Result(result) => {
                match key {
                    KeyCode::Up | KeyCode::Char('k') => {
                        if result.view_mode == screens::result::ResultViewMode::Json {
                            result.scroll_up(1);
                        } else {
                            result.table_previous();
                        }
                    }
                    KeyCode::Down => {
                        if result.view_mode == screens::result::ResultViewMode::Json {
                            result.scroll_down(1);
                        } else {
                            result.table_next();
                        }
                    }
                    KeyCode::Char('j') => {
                        if result.view_mode == screens::result::ResultViewMode::Table {
                            result.switch_view_mode();
                        } else {
                            result.scroll_down(1);
                        }
                    }
                    KeyCode::PageUp => {
                        if result.view_mode == screens::result::ResultViewMode::Table {
                            result.table_page_up();
                        } else {
                            result.scroll_up(10);
                        }
                    }
                    KeyCode::PageDown => {
                        if result.view_mode == screens::result::ResultViewMode::Table {
                            result.table_page_down();
                        } else {
                            result.scroll_down(10);
                        }
                    }
                    KeyCode::Char('t') => {
                        if result.view_mode == screens::result::ResultViewMode::Json && result.table_row_count > 0 {
                            result.switch_view_mode();
                        }
                    }
                    KeyCode::Char('o') => result.open_selected_url(),
                    KeyCode::Esc => {
                        result.clear_message();
                        self.screen = AppScreen::Browse(BrowseScreen::new(self.registry.clone()));
                    }
                    KeyCode::Char('q') => return Ok(true),
                    _ => {}
                }
            }
        }

        Ok(false)
    }

    fn render(&self, f: &mut ratatui::Frame) {
        let area = f.size();
        match &self.screen {
            AppScreen::MainMenu(menu) => menu.render(f, area),
            AppScreen::Browse(browse) => browse.render(f, area),
            AppScreen::Execute(execute) => {
                execute.render(f, area);
                if let Some((x, y)) = execute.cursor_position(area) {
                    f.set_cursor(x, y);
                }
            }
            AppScreen::Result(result) => result.render(f, area),
        }
    }

}

pub async fn run(client: RommClient) -> Result<()> {
    let openapi_path = std::env::var("ROMM_OPENAPI_PATH")
        .unwrap_or_else(|_| "openapi.json".to_string());

    let registry = if std::path::Path::new(&openapi_path).exists() {
        openapi::EndpointRegistry::from_file(&openapi_path)?
    } else {
        return Err(anyhow!(
            "OpenAPI file not found at {}. Please provide openapi.json or set ROMM_OPENAPI_PATH environment variable.",
            openapi_path
        ));
    };

    let mut app = App::new(client, registry);
    app.run().await?;

    Ok(())
}
