//! TUI render dispatch and global overlays.

use super::AppScreen;
use crate::tui::keyboard_help;
use crate::tui::screens::connected_splash;
use crate::tui::theme::RommStyles;

impl super::App {
    pub(in crate::tui::app) fn render(&mut self, f: &mut ratatui::Frame) {
        let area = f.area();
        let theme = self.theme.as_ref();
        let styles = RommStyles::new(theme);
        styles.fill_background(f, area);

        if let Some(ref splash) = self.startup_splash {
            connected_splash::render(f, area, splash, &styles);
        } else {
            match &mut self.screen {
                AppScreen::MainMenu(menu) => menu.render(f, area, &styles),
                AppScreen::LibraryBrowse(lib) => {
                    lib.render(f, area, &styles);
                    if let Some((x, y)) = lib.upload_prompt_cursor(area) {
                        f.set_cursor_position((x, y));
                    }
                }
                AppScreen::Search(search) => {
                    search.render(f, area, &styles);
                    if let Some((x, y)) = search.cursor_position(area) {
                        f.set_cursor_position((x, y));
                    }
                }
                AppScreen::Settings(settings) => {
                    settings.render(f, area, &styles);
                    if let Some((x, y)) = settings.cursor_position(area) {
                        f.set_cursor_position((x, y));
                    }
                }
                AppScreen::GameDetail(detail) => detail.render(f, area, &styles),
                AppScreen::ExtrasPicker(picker) => picker.render(f, area, &styles),
                AppScreen::Download(d) => d.render(f, area, &styles),
                AppScreen::SetupWizard(wizard) => {
                    wizard.render(f, area, &styles);
                    if let Some((x, y)) = wizard.cursor_pos(area) {
                        f.set_cursor_position((x, y));
                    }
                }
            }

            if self.show_keyboard_help {
                keyboard_help::render_keyboard_help(f, area, &styles);
            }
        }

        if let Some(prompt) = &self.startup_update_prompt {
            let popup_w = 44;
            let popup_h = 10;
            let popup_area = ratatui::layout::Rect {
                x: area.width.saturating_sub(popup_w) / 2,
                y: area.height.saturating_sub(popup_h) / 2,
                width: popup_w.min(area.width),
                height: popup_h.min(area.height),
            };
            styles.fill_surface(f, popup_area);

            let block = styles
                .panel_block(" Update Available ")
                .title_alignment(ratatui::layout::Alignment::Center);

            if prompt.updating {
                let text = vec![
                    ratatui::text::Line::from(""),
                    ratatui::text::Line::from("Downloading and installing...")
                        .alignment(ratatui::layout::Alignment::Center)
                        .style(styles.text()),
                    ratatui::text::Line::from("Please wait.")
                        .alignment(ratatui::layout::Alignment::Center)
                        .style(styles.text()),
                    ratatui::text::Line::from(""),
                    ratatui::text::Line::from("This may take a few moments.")
                        .alignment(ratatui::layout::Alignment::Center)
                        .style(styles.muted()),
                ];
                let paragraph = ratatui::widgets::Paragraph::new(text).block(block);
                f.render_widget(paragraph, popup_area);
            } else {
                let text = vec![
                    ratatui::text::Line::from(vec![
                        ratatui::text::Span::raw("Current: "),
                        ratatui::text::Span::styled(&prompt.status.current_version, styles.muted()),
                    ])
                    .alignment(ratatui::layout::Alignment::Center),
                    ratatui::text::Line::from(vec![
                        ratatui::text::Span::raw("Latest:  "),
                        ratatui::text::Span::styled(
                            &prompt.status.latest_version,
                            styles.success().add_modifier(ratatui::style::Modifier::BOLD),
                        ),
                    ])
                    .alignment(ratatui::layout::Alignment::Center),
                    ratatui::text::Line::from(""),
                    ratatui::text::Line::from("Would you like to update?")
                        .alignment(ratatui::layout::Alignment::Center)
                        .style(styles.text()),
                    ratatui::text::Line::from(""),
                    ratatui::text::Line::from(vec![
                        ratatui::text::Span::styled("Y/Enter", styles.selection()),
                        ratatui::text::Span::raw(": Yes (update)  "),
                        ratatui::text::Span::styled("N/Esc", styles.selection()),
                        ratatui::text::Span::raw(": No (skip)"),
                    ])
                    .alignment(ratatui::layout::Alignment::Center),
                    ratatui::text::Line::from(vec![
                        ratatui::text::Span::styled("C", styles.selection()),
                        ratatui::text::Span::raw(": View changelog"),
                    ])
                    .alignment(ratatui::layout::Alignment::Center),
                ];
                let paragraph = ratatui::widgets::Paragraph::new(text).block(block);
                f.render_widget(paragraph, popup_area);
            }
        }

        if let Some(ref err) = self.global_error {
            let popup_area = ratatui::layout::Rect {
                x: area.width.saturating_sub(60) / 2,
                y: area.height.saturating_sub(10) / 2,
                width: 60.min(area.width),
                height: 10.min(area.height),
            };
            styles.fill_surface(f, popup_area);
            let block = styles
                .panel_block("Error")
                .border_style(styles.error());
            let text = format!("{}\n\nPress Esc to dismiss", err);
            let paragraph = ratatui::widgets::Paragraph::new(text)
                .block(block)
                .style(styles.text())
                .wrap(ratatui::widgets::Wrap { trim: true });
            f.render_widget(paragraph, popup_area);
        }

        if let Some(ref notice) = self.global_notice {
            let popup_area = ratatui::layout::Rect {
                x: area.width.saturating_sub(60) / 2,
                y: area.height.saturating_sub(10) / 2,
                width: 60.min(area.width),
                height: 10.min(area.height),
            };
            styles.fill_surface(f, popup_area);
            let block = styles.panel_block("Notice");
            let text = format!("{notice}\n\nPress Esc to dismiss");
            let paragraph = ratatui::widgets::Paragraph::new(text)
                .block(block)
                .style(styles.text())
                .wrap(ratatui::widgets::Wrap { trim: true });
            f.render_widget(paragraph, popup_area);
        }
    }
}
