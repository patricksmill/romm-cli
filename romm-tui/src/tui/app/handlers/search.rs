//! Search overlay key handler.

use anyhow::Result;
use crossterm::event::{KeyCode, KeyEvent};

use romm_api::core::roms::{rom_list_fetch_complete, ROM_PAGE_CEILING};
use romm_api::endpoints::roms::GetRoms;
use romm_api::types::RomList;

use romm_api::error::RommError;

use super::super::background::types::{SearchLoadDone, SearchLoadEvent};
use super::super::{App, AppScreen};
use crate::tui::screens::{GameDetailPrevious, GameDetailScreen, SearchScreen};

fn cap_search_results(roms: &mut RomList) {
    roms.items.truncate(ROM_PAGE_CEILING as usize);
}

impl App {
    pub(in crate::tui::app) fn toggle_search_screen(&mut self) {
        match &self.screen {
            AppScreen::Search(_) => self.close_search_overlay(),
            _ => {
                let prev =
                    std::mem::replace(&mut self.screen, AppScreen::Search(SearchScreen::new()));
                if !Self::is_overlay_screen(&prev) {
                    self.screen_before_search = Some(prev);
                }
            }
        }
    }

    pub(in crate::tui::app) fn close_search_overlay(&mut self) {
        if !matches!(self.screen, AppScreen::Search(_)) {
            return;
        }
        let stored = self.screen_before_search.take();
        self.restore_screen_or_library(stored);
    }

    pub(in crate::tui::app) async fn handle_search(&mut self, key: &KeyEvent) -> Result<bool> {
        let search = match &mut self.screen {
            AppScreen::Search(s) => s,
            _ => return Ok(false),
        };
        match key.code {
            KeyCode::Backspace => search.delete_char(),
            KeyCode::Left => search.cursor_left(),
            KeyCode::Right => search.cursor_right(),
            KeyCode::Up => search.previous(),
            KeyCode::Down => search.next(),
            KeyCode::Char(c) => search.add_char(c),
            KeyCode::Enter => {
                if search.query.is_empty() {
                    // no-op (same as before: empty query does not search)
                } else if search.result_groups.is_some() && search.results_match_current_query() {
                    if let Some((primary, others)) = search.get_selected_group() {
                        let placeholder = self.transient_screen_placeholder();
                        let prev = std::mem::replace(&mut self.screen, placeholder);
                        if let AppScreen::Search(s) = prev {
                            self.screen = AppScreen::GameDetail(Box::new(GameDetailScreen::new(
                                primary,
                                others,
                                GameDetailPrevious::Search(s),
                                self.downloads.shared(),
                                self.config.tui_layout.game_detail_cover_panel_width,
                            )));
                            self.maybe_start_game_detail_cover_load();
                            self.refresh_current_game_saves();
                            self.refresh_current_game_achievements();
                        }
                    }
                } else {
                    let query = search.query.clone();
                    let req = GetRoms {
                        search_term: Some(query.clone()),
                        limit: Some(50),
                        ..Default::default()
                    };
                    search.loading = true;
                    if let Some(task) = self.search_load_task.take() {
                        task.abort();
                    }
                    let client = self.client.clone();
                    let tx = self.search_load_tx.clone();
                    self.search_load_task = Some(tokio::spawn(async move {
                        let mut req = req;
                        let mut aggregated: Option<RomList> = None;

                        loop {
                            match client.call(&req).await {
                                Ok(mut batch) => {
                                    if let Some(ref mut all) = aggregated {
                                        if batch.items.is_empty() {
                                            break;
                                        }
                                        all.items.append(&mut batch.items);
                                        cap_search_results(all);
                                        let _ = tx.send(SearchLoadDone {
                                            query: query.clone(),
                                            event: SearchLoadEvent::Batch(all.clone()),
                                        });
                                        if rom_list_fetch_complete(all) {
                                            break;
                                        }
                                        req.offset = Some(all.items.len() as u32);
                                    } else {
                                        cap_search_results(&mut batch);
                                        let loaded = batch.items.len() as u64;
                                        let _ = tx.send(SearchLoadDone {
                                            query: query.clone(),
                                            event: SearchLoadEvent::Batch(batch.clone()),
                                        });
                                        req.offset = Some(loaded as u32);
                                        aggregated = Some(batch);
                                        if aggregated.as_ref().is_some_and(rom_list_fetch_complete)
                                        {
                                            break;
                                        }
                                    }
                                }
                                Err(e) => {
                                    let _ = tx.send(SearchLoadDone {
                                        query: query.clone(),
                                        event: SearchLoadEvent::Failed(RommError::from(e)),
                                    });
                                    return;
                                }
                            }
                        }

                        let _ = tx.send(SearchLoadDone {
                            query,
                            event: SearchLoadEvent::Complete,
                        });
                    }));
                }
            }
            KeyCode::Esc => {
                if search.results.is_some() {
                    search.clear_results();
                } else {
                    self.close_search_overlay();
                }
            }
            _ => {}
        }
        Ok(false)
    }
}
