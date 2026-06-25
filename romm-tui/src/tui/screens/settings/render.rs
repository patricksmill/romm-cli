use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::Modifier;
use ratatui::text::{Line, Span};
use ratatui::widgets::{List, ListItem, ListState, Paragraph, Tabs};
use ratatui::Frame;

use crate::tui::footer_hint::{render_footer_panel, FooterHintEntry, PATH_PICKER_HINTS};
use crate::tui::theme::RommStyles;

use super::types::{
    ConsolePathKind, SettingsConfirm, SettingsPickerKind, SettingsRow, SettingsScreen, SettingsTab,
};

const SETTINGS_DEFAULT_HINTS: &[FooterHintEntry] = &[
    FooterHintEntry {
        key: "Tab/←/→",
        label: "Tabs",
    },
    FooterHintEntry {
        key: "S",
        label: "Save to disk",
    },
];

const SETTINGS_CONFIRM_EXIT_HINTS: &[FooterHintEntry] = &[
    FooterHintEntry {
        key: "Enter",
        label: "Save",
    },
    FooterHintEntry {
        key: "N",
        label: "Don't save",
    },
    FooterHintEntry {
        key: "Esc",
        label: "Cancel",
    },
];

const SETTINGS_CONFIRM_HINTS: &[FooterHintEntry] = &[
    FooterHintEntry {
        key: "Enter",
        label: "Confirm",
    },
    FooterHintEntry {
        key: "Esc",
        label: "Cancel",
    },
];

const SETTINGS_EDITING_HINTS: &[FooterHintEntry] = &[
    FooterHintEntry {
        key: "Backspace",
        label: "Delete",
    },
    FooterHintEntry {
        key: "Enter",
        label: "Save",
    },
    FooterHintEntry {
        key: "Esc",
        label: "Cancel",
    },
];

const DEVICE_PICKER_HINTS: &[FooterHintEntry] = &[
    FooterHintEntry {
        key: "Enter",
        label: "Choose",
    },
    FooterHintEntry {
        key: "Esc",
        label: "Cancel",
    },
];

const CONSOLE_PICKER_HINTS: &[FooterHintEntry] = &[
    FooterHintEntry {
        key: "Enter",
        label: "Set path",
    },
    FooterHintEntry {
        key: "Del",
        label: "Clear custom",
    },
];

impl SettingsScreen {
    pub fn render(&mut self, f: &mut Frame, area: Rect, styles: &RommStyles) {
        if let Some((platform_id, ref mut picker)) = self.console_path_picker {
            let chunks = Layout::default()
                .constraints([
                    Constraint::Length(4),
                    Constraint::Min(12),
                    Constraint::Length(3),
                ])
                .direction(ratatui::layout::Direction::Vertical)
                .split(area);
            let platform_name = self
                .console_platforms
                .iter()
                .find(|p| p.id == platform_id)
                .map(Self::platform_display_name)
                .unwrap_or_else(|| format!("Platform {platform_id}"));
            let info = [
                format!(
                    "romm-cli: v{} | RomM server: {}",
                    self.version, self.server_version
                ),
                format!("Console: {platform_name}"),
            ];
            f.render_widget(
                Paragraph::new(info.join("\n"))
                    .style(styles.text())
                    .block(styles.header_block()),
                chunks[0],
            );
            picker.render(
                f,
                chunks[1],
                "Choose console directory",
                PATH_PICKER_HINTS,
                styles,
            );
            f.render_widget(
                Paragraph::new("Console directory picker — Esc returns without changing")
                    .style(styles.footer_hint())
                    .block(styles.panel_block_untitled()),
                chunks[2],
            );
            return;
        }

        if self.console_picker_open {
            self.render_console_picker(f, area, styles);
            return;
        }

        if let Some((kind, ref mut picker)) = self.path_picker {
            let chunks = Layout::default()
                .constraints([
                    Constraint::Length(4),
                    Constraint::Min(12),
                    Constraint::Length(3),
                ])
                .direction(ratatui::layout::Direction::Vertical)
                .split(area);
            let info = [
                format!(
                    "romm-cli: v{} | RomM server: {}",
                    self.version, self.server_version
                ),
                format!("GitHub:   {}", self.github_url),
                format!("Auth:     {}", self.auth_status),
            ];
            f.render_widget(
                Paragraph::new(info.join("\n"))
                    .style(styles.text())
                    .block(styles.header_block()),
                chunks[0],
            );
            let title = match kind {
                SettingsPickerKind::RomsDir => "Choose ROMs directory",
                SettingsPickerKind::SaveDir => "Choose save directory",
            };
            picker.render(f, chunks[1], title, PATH_PICKER_HINTS, styles);
            f.render_widget(
                Paragraph::new("ROMs directory picker — Esc returns without changing")
                    .style(styles.footer_hint())
                    .block(styles.panel_block_untitled()),
                chunks[2],
            );
            return;
        }

        if self.device_picker_open {
            self.render_device_picker(f, area, styles);
            return;
        }

        let chunks = Layout::default()
            .constraints([
                Constraint::Length(4), // Header info
                Constraint::Length(3), // Settings tabs
                Constraint::Min(10),   // Editable list
                Constraint::Length(3), // Message/Hint
                Constraint::Length(3), // Footer help
            ])
            .direction(ratatui::layout::Direction::Vertical)
            .split(area);

        // -- Header Info --
        let info = [
            format!(
                "romm-cli: v{} | RomM server: {}",
                self.version, self.server_version
            ),
            format!("GitHub:   {}", self.github_url),
            format!("Auth:     {}", self.auth_status),
        ];
        f.render_widget(
            Paragraph::new(info.join("\n"))
                .style(styles.text())
                .block(styles.header_block()),
            chunks[0],
        );

        // -- Tabs --
        let titles = SettingsTab::ALL
            .iter()
            .map(|tab| Line::from(Span::raw(tab.title())))
            .collect::<Vec<_>>();
        let tabs = Tabs::new(titles)
            .select(self.selected_tab.index())
            .block(styles.panel_block_untitled())
            .style(styles.muted())
            .highlight_style(styles.selection());
        f.render_widget(tabs, chunks[1]);

        // -- Editable List --
        let items = self
            .visible_rows()
            .iter()
            .copied()
            .map(|row| self.render_row_item(row, styles))
            .collect::<Vec<_>>();

        let mut state = ListState::default();
        state.select(Some(self.selected_row_index()));

        let list = List::new(items)
            .block(styles.panel_block(format!(" {} ", self.selected_tab.title())))
            .highlight_style(styles.selection())
            .highlight_symbol(">> ");

        f.render_stateful_widget(list, chunks[2], &mut state);

        // -- Message Area --
        if let Some(confirm) = &self.confirm {
            let msg = match confirm {
                SettingsConfirm::Reset => {
                    "Are you sure you want to delete all settings? (Enter: Yes, Esc: Cancel)"
                }
                SettingsConfirm::ClearCache => {
                    "Are you sure you want to clear the ROM cache? (Enter: Yes, Esc: Cancel)"
                }
                SettingsConfirm::ExitUnsaved => {
                    "Save changes before leaving? (Enter: Save, N: Don't save, Esc: Cancel)"
                }
            };
            f.render_widget(
                Paragraph::new(msg).style(styles.error().add_modifier(Modifier::BOLD)),
                chunks[3],
            );
        } else if let Some((msg, tone)) = &self.message {
            f.render_widget(
                Paragraph::new(msg.as_str()).style(styles.tone(*tone)),
                chunks[3],
            );
        } else if self.editing {
            f.render_widget(
                Paragraph::new("Editing... Enter: save   Esc: cancel").style(styles.label()),
                chunks[3],
            );
        }

        // -- Footer Help --
        let help = if self.confirm == Some(SettingsConfirm::ExitUnsaved) {
            SETTINGS_CONFIRM_EXIT_HINTS
        } else if self.confirm.is_some() {
            SETTINGS_CONFIRM_HINTS
        } else if self.editing {
            SETTINGS_EDITING_HINTS
        } else {
            SETTINGS_DEFAULT_HINTS
        };
        render_footer_panel(f, chunks[4], styles, help, None);
    }

    fn render_row_item(&self, row: SettingsRow, styles: &RommStyles) -> ListItem<'static> {
        let label = self.row_label(row);
        match row {
            SettingsRow::SyncDevice | SettingsRow::SyncNow if !self.save_sync_supported() => {
                ListItem::new(label).style(styles.muted())
            }
            _ => ListItem::new(label).style(styles.text()),
        }
    }

    pub(crate) fn row_label(&self, row: SettingsRow) -> String {
        let label = match row {
            SettingsRow::BaseUrl => format!(
                "Base URL:     {}",
                if self.editing && self.selected_row() == SettingsRow::BaseUrl {
                    &self.edit_buffer
                } else {
                    &self.base_url
                }
            ),
            SettingsRow::RomsDir => format!("Roms Dir:     {}", self.download_dir),
            SettingsRow::ConsolePaths => {
                let mapped = self.rom_platform_dirs.len();
                format!("Console paths: {mapped} custom · Enter to edit")
            }
            SettingsRow::SaveConsolePaths => {
                let mapped = self.save_platform_dirs.len();
                format!("Save console paths: {mapped} custom · Enter to edit")
            }
            SettingsRow::UseHttps => format!(
                "Use HTTPS:    {}",
                if self.use_https { "[X] Yes" } else { "[ ] No" }
            ),
            SettingsRow::SaveDir => format!("Save Dir:     {}", self.save_dir),
            SettingsRow::SyncDevice => format!(
                "Sync Device:  {}",
                self.sync_device_id.as_deref().unwrap_or("(not selected)")
            ),
            SettingsRow::SyncNow => "Sync Saves Now".to_string(),
            SettingsRow::ExtrasRelatedRoms => format!(
                "Incl. updates/DLC (picker default): {}",
                if self.extras_include_related_roms {
                    "[X] Yes"
                } else {
                    "[ ] No"
                }
            ),
            SettingsRow::ExtrasCover => format!(
                "Incl. cover (picker default):       {}",
                if self.extras_include_cover {
                    "[X] Yes"
                } else {
                    "[ ] No"
                }
            ),
            SettingsRow::ExtrasManual => format!(
                "Incl. manual (picker default):      {}",
                if self.extras_include_manual {
                    "[X] Yes"
                } else {
                    "[ ] No"
                }
            ),
            SettingsRow::Theme => format!(
                "Theme:        {} (Enter to change)",
                self.theme_display_name()
            ),
            SettingsRow::Auth => format!("Auth:         {} (Enter to change)", self.auth_status),
            SettingsRow::ClearCache => "Clear Cache (Remove cached ROM data)".to_string(),
            SettingsRow::ResetConfiguration => {
                "Reset Configuration (Delete settings from disk & keyring)".to_string()
            }
        };
        if matches!(row, SettingsRow::SyncDevice | SettingsRow::SyncNow)
            && !self.save_sync_supported()
        {
            format!("{label} (requires newer RomM server)")
        } else {
            label
        }
    }

    pub fn cursor_position(&self, area: Rect) -> Option<(u16, u16)> {
        if let Some((kind, ref picker)) = self.path_picker {
            let chunks = Layout::default()
                .constraints([
                    Constraint::Length(4),
                    Constraint::Min(12),
                    Constraint::Length(3),
                ])
                .direction(ratatui::layout::Direction::Vertical)
                .split(area);
            let title = match kind {
                SettingsPickerKind::RomsDir => "Choose ROMs directory",
                SettingsPickerKind::SaveDir => "Choose save directory",
            };
            return picker.cursor_position(chunks[1], title);
        }

        if !self.editing || self.selected_row() != SettingsRow::BaseUrl {
            return None;
        }

        let chunks = Layout::default()
            .constraints([
                Constraint::Length(4),
                Constraint::Length(3),
                Constraint::Min(10),
                Constraint::Length(3),
                Constraint::Length(3),
            ])
            .direction(ratatui::layout::Direction::Vertical)
            .split(area);

        let list_area = chunks[2];
        let y = list_area.y + 1 + self.selected_row_index() as u16;
        let label_len = 14; // "Base URL:     ".len()
        let x = list_area.x + 1 /* border */ + 3 /* highlight symbol */ + label_len + self.edit_cursor as u16;

        Some((x, y))
    }

    fn render_device_picker(&mut self, f: &mut Frame, area: Rect, styles: &RommStyles) {
        let chunks = Layout::default()
            .constraints([
                Constraint::Length(4),
                Constraint::Min(10),
                Constraint::Length(3),
            ])
            .direction(ratatui::layout::Direction::Vertical)
            .split(area);
        let info = [
            format!(
                "romm-cli: v{} | RomM server: {}",
                self.version, self.server_version
            ),
            "Select the RomM sync device used for manual push-pull.".to_string(),
        ];
        f.render_widget(
            Paragraph::new(info.join("\n"))
                .style(styles.text())
                .block(styles.header_block()),
            chunks[0],
        );
        if self.device_picker_loading {
            f.render_widget(
                Paragraph::new("Loading devices...")
                    .style(styles.text())
                    .block(styles.panel_block(" Devices ")),
                chunks[1],
            );
        } else if let Some(error) = &self.device_picker_error {
            f.render_widget(
                Paragraph::new(format!("Could not load devices: {error}"))
                    .style(styles.error())
                    .block(styles.panel_block(" Devices ")),
                chunks[1],
            );
        } else {
            let items: Vec<ListItem> = self
                .devices
                .iter()
                .map(|d| {
                    let name = d.name.as_deref().unwrap_or("(unnamed)");
                    ListItem::new(format!("{name}  [{}]  mode={:?}", d.id, d.sync_mode))
                        .style(styles.text())
                })
                .collect();
            let mut state = ListState::default();
            state.select(Some(self.device_selected_index));
            f.render_stateful_widget(
                List::new(items)
                    .block(styles.panel_block(" Devices "))
                    .highlight_symbol(">> ")
                    .highlight_style(styles.selection()),
                chunks[1],
                &mut state,
            );
        }
        render_footer_panel(f, chunks[2], styles, DEVICE_PICKER_HINTS, None);
    }

    fn render_console_picker(&mut self, f: &mut Frame, area: Rect, styles: &RommStyles) {
        let kind = self.active_console_kind.unwrap_or(ConsolePathKind::Roms);
        let chunks = Layout::default()
            .constraints([
                Constraint::Length(4),
                Constraint::Min(10),
                Constraint::Length(3),
            ])
            .direction(ratatui::layout::Direction::Vertical)
            .split(area);
        let subtitle = match kind {
            ConsolePathKind::Roms => "Set a custom ROM path for consoles on other drives.",
            ConsolePathKind::Saves => "Set a custom save path for consoles on other drives.",
        };
        let info = [
            format!(
                "romm-cli: v{} | RomM server: {}",
                self.version, self.server_version
            ),
            subtitle.to_string(),
        ];
        f.render_widget(
            Paragraph::new(info.join("\n"))
                .style(styles.text())
                .block(styles.header_block()),
            chunks[0],
        );
        if self.console_picker_loading {
            f.render_widget(
                Paragraph::new("Loading platforms...")
                    .style(styles.text())
                    .block(styles.panel_block(" Consoles ")),
                chunks[1],
            );
        } else if let Some(error) = &self.console_picker_error {
            f.render_widget(
                Paragraph::new(format!("Could not load platforms: {error}"))
                    .style(styles.error())
                    .block(styles.panel_block(" Consoles ")),
                chunks[1],
            );
        } else if self.console_platforms.is_empty() {
            f.render_widget(
                Paragraph::new("No platforms returned from the server.")
                    .style(styles.warning())
                    .block(styles.panel_block(" Consoles ")),
                chunks[1],
            );
        } else {
            let items: Vec<ListItem> = self
                .console_platforms
                .iter()
                .map(|platform| {
                    let name = Self::platform_display_name(platform);
                    let path = self.console_dir_preview(kind, platform);
                    let custom = self
                        .console_dirs(kind)
                        .get(&platform.id)
                        .is_some_and(|s| !s.trim().is_empty());
                    let tag = if custom {
                        "custom path"
                    } else {
                        "base default"
                    };
                    ListItem::new(format!("{name}  [{tag}]  {path}")).style(styles.text())
                })
                .collect();
            let mut state = ListState::default();
            state.select(Some(self.console_selected_index));
            f.render_stateful_widget(
                List::new(items)
                    .block(styles.panel_block(" Consoles "))
                    .highlight_symbol(">> ")
                    .highlight_style(styles.selection()),
                chunks[1],
                &mut state,
            );
        }
        render_footer_panel(f, chunks[2], styles, CONSOLE_PICKER_HINTS, None);
    }
}
