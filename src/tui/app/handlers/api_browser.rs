//! API browser flow key handlers.

use anyhow::Result;
use crossterm::event::{KeyCode, KeyEvent};

use crate::openapi::resolve_path_template;

use super::super::{App, AppScreen};
use crate::tui::screens::{
    BrowseScreen, ExecuteScreen, MainMenuScreen, ResultDetailScreen, ResultScreen,
};

impl App {
    pub(in crate::tui::app) fn handle_browse(&mut self, key: &KeyEvent) -> Result<bool> {
        use crate::tui::screens::browse::ViewMode;

        let browse = match &mut self.screen {
            AppScreen::Browse(b) => b,
            _ => return Ok(false),
        };
        match key.code {
            KeyCode::Up | KeyCode::Char('k') => browse.previous(),
            KeyCode::Down | KeyCode::Char('j') => browse.next(),
            KeyCode::Left | KeyCode::Char('h') if browse.view_mode == ViewMode::Endpoints => {
                browse.switch_view();
            }
            KeyCode::Right | KeyCode::Char('l') if browse.view_mode == ViewMode::Sections => {
                browse.switch_view();
            }
            KeyCode::Tab => browse.switch_view(),
            KeyCode::Enter => {
                if browse.view_mode == ViewMode::Endpoints {
                    if let Some(ep) = browse.get_selected_endpoint() {
                        self.screen = AppScreen::Execute(ExecuteScreen::new(ep.clone()));
                    }
                } else {
                    browse.switch_view();
                }
            }
            KeyCode::Esc => self.screen = AppScreen::MainMenu(MainMenuScreen::new()),
            _ => {}
        }
        Ok(false)
    }

    // -- Execute endpoint ---------------------------------------------------

    pub(in crate::tui::app) async fn handle_execute(&mut self, key: &KeyEvent) -> Result<bool> {
        let execute = match &mut self.screen {
            AppScreen::Execute(e) => e,
            _ => return Ok(false),
        };
        match key.code {
            KeyCode::Tab => execute.next_field(),
            KeyCode::BackTab => execute.previous_field(),
            KeyCode::Char(c) => execute.add_char_to_focused(c),
            KeyCode::Backspace => execute.delete_char_from_focused(),
            KeyCode::Enter => {
                let endpoint = execute.endpoint.clone();
                let query = execute.get_query_params();
                let body = if endpoint.has_body && !execute.body_text.is_empty() {
                    Some(serde_json::from_str(&execute.body_text)?)
                } else {
                    None
                };
                let resolved_path =
                    match resolve_path_template(&endpoint.path, &execute.get_path_params()) {
                        Ok(p) => p,
                        Err(e) => {
                            self.screen = AppScreen::Result(ResultScreen::new(
                                serde_json::json!({ "error": format!("{e}") }),
                                None,
                                None,
                            ));
                            return Ok(false);
                        }
                    };
                match self
                    .client
                    .request_json(&endpoint.method, &resolved_path, &query, body)
                    .await
                {
                    Ok(result) => {
                        self.screen = AppScreen::Result(ResultScreen::new(
                            result,
                            Some(&endpoint.method),
                            Some(resolved_path.as_str()),
                        ));
                    }
                    Err(e) => {
                        self.screen = AppScreen::Result(ResultScreen::new(
                            serde_json::json!({ "error": format!("{e}") }),
                            None,
                            None,
                        ));
                    }
                }
            }
            KeyCode::Esc => {
                self.screen = AppScreen::Browse(BrowseScreen::new(self.registry.clone()));
            }
            _ => {}
        }
        Ok(false)
    }

    // -- Result view --------------------------------------------------------

    pub(in crate::tui::app) fn handle_result(&mut self, key: &KeyEvent) -> Result<bool> {
        use crate::tui::screens::result::ResultViewMode;

        let result = match &mut self.screen {
            AppScreen::Result(r) => r,
            _ => return Ok(false),
        };
        match key.code {
            KeyCode::Up | KeyCode::Char('k') => {
                if result.view_mode == ResultViewMode::Json {
                    result.scroll_up(1);
                } else {
                    result.table_previous();
                }
            }
            KeyCode::Down => {
                if result.view_mode == ResultViewMode::Json {
                    result.scroll_down(1);
                } else {
                    result.table_next();
                }
            }
            KeyCode::Char('j') if result.view_mode == ResultViewMode::Json => {
                result.scroll_down(1);
            }
            KeyCode::PageUp => {
                if result.view_mode == ResultViewMode::Table {
                    result.table_page_up();
                } else {
                    result.scroll_up(10);
                }
            }
            KeyCode::PageDown => {
                if result.view_mode == ResultViewMode::Table {
                    result.table_page_down();
                } else {
                    result.scroll_down(10);
                }
            }
            KeyCode::Char('t') if result.table_row_count > 0 => {
                result.switch_view_mode();
            }
            KeyCode::Enter
                if result.view_mode == ResultViewMode::Table && result.table_row_count > 0 =>
            {
                if let Some(item) = result.get_selected_item_value() {
                    let prev = std::mem::replace(
                        &mut self.screen,
                        AppScreen::MainMenu(MainMenuScreen::new()),
                    );
                    if let AppScreen::Result(rs) = prev {
                        self.screen = AppScreen::ResultDetail(ResultDetailScreen::new(rs, item));
                    }
                }
            }
            KeyCode::Esc => {
                result.clear_message();
                self.screen = AppScreen::Browse(BrowseScreen::new(self.registry.clone()));
            }
            KeyCode::Char('q') => return Ok(true),
            _ => {}
        }
        Ok(false)
    }

    // -- Result detail ------------------------------------------------------

    pub(in crate::tui::app) fn handle_result_detail(&mut self, key: &KeyEvent) -> Result<bool> {
        let detail = match &mut self.screen {
            AppScreen::ResultDetail(d) => d,
            _ => return Ok(false),
        };
        match key.code {
            KeyCode::Up | KeyCode::Char('k') => detail.scroll_up(1),
            KeyCode::Down | KeyCode::Char('j') => detail.scroll_down(1),
            KeyCode::PageUp => detail.scroll_up(10),
            KeyCode::PageDown => detail.scroll_down(10),
            KeyCode::Char('o') => detail.open_image_url(),
            KeyCode::Esc => {
                detail.clear_message();
                let prev =
                    std::mem::replace(&mut self.screen, AppScreen::MainMenu(MainMenuScreen::new()));
                if let AppScreen::ResultDetail(d) = prev {
                    self.screen = AppScreen::Result(d.parent);
                }
            }
            KeyCode::Char('q') => return Ok(true),
            _ => {}
        }
        Ok(false)
    }
}
