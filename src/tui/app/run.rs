//! Main TUI event loop.

use anyhow::Result;
use crossterm::event::{
    self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEventKind, KeyModifiers,
};
use crossterm::execute;
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;
use std::time::Duration;

use crate::commands::library_scan::ScanCacheInvalidate;
use crate::types::RomList;

use super::background::types::{RomLoadDone, RomLoadEvent};
use super::AppScreen;

impl super::App {
    pub async fn run(&mut self) -> Result<()> {
        enable_raw_mode()?;
        let mut stdout = std::io::stdout();
        execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
        let backend = CrosstermBackend::new(stdout);
        let mut terminal = Terminal::new(backend)?;

        loop {
            self.poll_background_tasks();
            if self
                .startup_splash
                .as_ref()
                .is_some_and(|s| s.should_auto_dismiss())
            {
                self.startup_splash = None;
            }
            // Draw the current screen. `App::render` delegates to the
            // appropriate screen type based on `self.screen`.
            terminal.draw(|f| self.render(f))?;

            // If an update was triggered, execute it now (this will block the loop and show the "Updating..." message)
            if let Some(ref mut prompt) = self.startup_update_prompt {
                if prompt.updating {
                    // Safety: Don't actually run self_update if this is a mock
                    if prompt.status.latest_version == "9.9.9-mock" {
                        tokio::time::sleep(std::time::Duration::from_secs(2)).await; // Simulate some work
                        self.global_notice =
                            Some("Mock update successful! (No files were changed)".into());
                        self.startup_update_prompt = None;
                    } else {
                        let options = crate::update::ApplyUpdateOptions {
                            show_progress: false,
                            show_output: false,
                            no_confirm: true,
                            target_version_tag: Some(prompt.status.release_tag.clone()),
                        };
                        match crate::update::apply_update(None, options).await {
                            Ok(crate::update::ApplyUpdateOutcome::Updated(version)) => {
                                self.global_notice = Some(format!(
                                    "Updated to {version}. Restart romm-cli to use the new version."
                                ));
                            }
                            Ok(crate::update::ApplyUpdateOutcome::UpToDate(version)) => {
                                self.global_notice =
                                    Some(format!("Already up to date (`{version}`)."));
                            }
                            Err(err) => {
                                self.global_error = Some(format!("Update failed: {err:#}"));
                            }
                        }
                        self.startup_update_prompt = None;
                    }
                    continue;
                }
            }

            // Poll with a short timeout so the UI refreshes during downloads
            // even when the user is not pressing any keys.
            if event::poll(Duration::from_millis(100))? {
                if let Event::Key(key_event) = event::read()? {
                    if Self::is_force_quit_key(&key_event) {
                        break;
                    }
                    if key_event.kind == KeyEventKind::Press
                        && key_event.modifiers.contains(KeyModifiers::CONTROL)
                        && matches!(key_event.code, KeyCode::Char('r') | KeyCode::Char('R'))
                        && !self.blocks_global_chord_shortcuts()
                    {
                        if let AppScreen::LibraryBrowse(ref lib) = self.screen {
                            if !lib.any_search_bar_open()
                                && !lib.any_upload_prompt_open()
                                && !self.library_upload_inflight
                                && !self.library_scan_inflight
                            {
                                self.spawn_library_rescan_worker(ScanCacheInvalidate::AllPlatforms);
                            }
                        }
                        continue;
                    }
                    if key_event.kind == KeyEventKind::Press
                        && key_event.modifiers.contains(KeyModifiers::CONTROL)
                        && matches!(key_event.code, KeyCode::Char('u') | KeyCode::Char('U'))
                        && !self.blocks_global_chord_shortcuts()
                    {
                        if let AppScreen::LibraryBrowse(ref mut lib) = self.screen {
                            if lib.any_upload_prompt_open() {
                                lib.close_upload_prompt();
                            } else if !lib.any_search_bar_open()
                                && !self.library_upload_inflight
                                && !self.library_scan_inflight
                            {
                                if lib.subsection
                                    == crate::tui::screens::library_browse::LibrarySubsection::ByConsole
                                {
                                    lib.open_upload_prompt();
                                } else {
                                    lib.set_metadata_footer(Some(
                                        "Upload requires Consoles view — press t".into(),
                                    ));
                                }
                            }
                        }
                        continue;
                    }
                    if key_event.kind == KeyEventKind::Press
                        && self.handle_key_event(&key_event).await?
                    {
                        break;
                    }
                }
            }

            // Process deferred ROM fetch (set during LibraryBrowse ↑/↓, subsection switch, refresh).
            // Cache hits apply synchronously; network fetch runs in a background task so the loop
            // never awaits HTTP and the UI stays responsive (see `poll_rom_load_results`).
            if let Some((key, req, expected, context, started)) = self.deferred_load_roms.take() {
                // Fast path: valid disk cache — no await, no spawn, load immediately.
                if let Some(ref k) = key {
                    if let Some(cached) = self.rom_cache.get_valid(k, expected) {
                        if let AppScreen::LibraryBrowse(ref mut lib) = self.screen {
                            if crate::tui::app::rom_load::primary_rom_load_result_matches_selection(
                                lib, &key,
                            ) {
                                lib.set_roms(cached.clone());
                                lib.set_rom_loading(false);
                                tracing::debug!(
                                    "rom-list-render context={} latency_ms={} (cache_hit)",
                                    context,
                                    started.elapsed().as_millis()
                                );
                            } else {
                                lib.set_rom_loading(false);
                                tracing::debug!(
                                    "rom-list-render context={} skipped stale cache hit",
                                    context
                                );
                            }
                        }
                        continue;
                    }
                }

                // Debounce network fetches
                if started.elapsed() < std::time::Duration::from_millis(250) {
                    // Put it back to keep waiting
                    self.deferred_load_roms = Some((key, req, expected, context, started));
                    continue;
                }

                let gen = self.rom_load_gen;
                if let AppScreen::LibraryBrowse(ref mut lib) = self.screen {
                    lib.set_rom_loading(expected > 0);
                }
                if expected == 0 {
                    if let AppScreen::LibraryBrowse(ref mut lib) = self.screen {
                        lib.set_rom_loading(false);
                    }
                    continue;
                }

                let Some(r) = req else {
                    if let AppScreen::LibraryBrowse(ref mut lib) = self.screen {
                        lib.set_rom_loading(false);
                    }
                    continue;
                };
                let client = self.client.clone();
                let tx = self.rom_load_tx.clone();

                self.rom_load_task = Some(tokio::spawn(async move {
                    let mut req = r;
                    let mut aggregated: Option<RomList> = None;

                    loop {
                        match client.call(&req).await {
                            Ok(mut batch) => {
                                if let Some(ref mut all) = aggregated {
                                    if batch.items.is_empty() {
                                        break;
                                    }
                                    all.items.append(&mut batch.items);
                                    let _ = tx.send(RomLoadDone {
                                        gen,
                                        key: key.clone(),
                                        expected,
                                        event: RomLoadEvent::Batch(all.clone()),
                                        context,
                                        started,
                                    });
                                    if all.items.len() as u64 >= all.total {
                                        break;
                                    }
                                    req.offset = Some(all.items.len() as u32);
                                } else {
                                    let loaded = batch.items.len() as u64;
                                    let total = batch.total;
                                    let _ = tx.send(RomLoadDone {
                                        gen,
                                        key: key.clone(),
                                        expected,
                                        event: RomLoadEvent::Batch(batch.clone()),
                                        context,
                                        started,
                                    });
                                    req.offset = Some(loaded as u32);
                                    aggregated = Some(batch);
                                    if loaded >= total {
                                        break;
                                    }
                                }
                            }
                            Err(e) => {
                                let _ = tx.send(RomLoadDone {
                                    gen,
                                    key: key.clone(),
                                    expected,
                                    event: RomLoadEvent::Failed(format!("{e:#}")),
                                    context,
                                    started,
                                });
                                return;
                            }
                        }
                        // Cap at 20k
                        if let Some(ref all) = aggregated {
                            if all.items.len() >= 20000 {
                                break;
                            }
                        }
                    }

                    let _ = tx.send(RomLoadDone {
                        gen,
                        key,
                        expected,
                        event: RomLoadEvent::Complete,
                        context,
                        started,
                    });
                }));
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
}
