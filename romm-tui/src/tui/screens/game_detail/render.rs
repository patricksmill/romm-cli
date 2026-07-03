use ratatui::layout::{Alignment, Constraint, Direction, Layout, Rect};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Gauge, Paragraph, Tabs};
use ratatui::Frame;
use ratatui_image::{Resize, StatefulImage};

use crate::tui::footer_hint::{render_footer_panel, FooterHintEntry};
use crate::tui::theme::RommStyles;
use romm_api::core::download::DownloadStatus;
use romm_api::core::utils::format_size;
use romm_api::core::utils::truncate;

use super::achievements::achievement_lines;
use super::saves::save_lines;
use super::types::{CoverState, DetailTab, GameDetailScreen};

impl GameDetailScreen {
    pub fn render(&mut self, f: &mut Frame, area: Rect, styles: &RommStyles) {
        if let Some(picker) = self.save_upload_picker.as_mut() {
            picker.render(
                f,
                area,
                "Upload save file",
                &[
                    FooterHintEntry {
                        key: "Ctrl+Enter",
                        label: "Apply file",
                    },
                    FooterHintEntry {
                        key: "Enter",
                        label: "Choose file",
                    },
                    FooterHintEntry {
                        key: "Esc",
                        label: "Cancel",
                    },
                ],
                styles,
            );
            return;
        }
        let chunks = Layout::default()
            .constraints([Constraint::Min(10), Constraint::Length(3)])
            .direction(Direction::Vertical)
            .split(area);
        let body = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Min(10),
                Constraint::Length(self.cover_panel_width),
            ])
            .split(chunks[0]);

        self.render_metadata_panel(f, body[0], styles);
        self.render_cover_panel(f, body[1], styles);
        self.render_footer_panel(f, chunks[1], styles);
    }

    fn render_cover_panel(&mut self, f: &mut Frame, area: Rect, styles: &RommStyles) {
        let platform = self
            .rom
            .platform_display_name
            .as_deref()
            .or(self.rom.platform_custom_name.as_deref())
            .unwrap_or("—");
        let name = truncate(&self.rom.name, 28);
        if matches!(self.cover_state, CoverState::Ready) {
            if let Some(image_state) = self.cover_image.as_mut() {
                let block = styles.panel_block("Cover");
                let inner = block.inner(area);
                f.render_widget(block, area);
                let widget = StatefulImage::default().resize(Resize::Fit(None));
                f.render_stateful_widget(widget, inner, image_state);
                return;
            }
        }

        let content = match &self.cover_state {
            CoverState::Ready => vec![
                Line::from(""),
                Line::from(Span::styled("Inline cover ready", styles.success())),
                Line::from(""),
                Line::from(self.cover_pipeline_label()),
                Line::from("Press o for browser view"),
            ],
            CoverState::Loading => vec![
                Line::from(""),
                Line::from(Span::styled("Loading cover...", styles.warning())),
                Line::from(""),
                Line::from("Fetching image"),
                Line::from("in background"),
            ],
            CoverState::Failed(message) => vec![
                Line::from(""),
                Line::from(Span::styled("Cover unavailable", styles.error())),
                Line::from(""),
                Line::from(truncate(message, 26)),
                Line::from(""),
                Line::from("Press o to open URL"),
            ],
            CoverState::Idle => vec![
                Line::from(""),
                Line::from(if self.rom.url_cover.is_some() {
                    "Cover available"
                } else {
                    "No cover URL"
                }),
                Line::from(""),
                Line::from("Press o to open cover"),
                Line::from("in browser"),
            ],
        };
        let lines = vec![
            Line::from(Span::styled(format!("[{}]", platform), styles.label())),
            Line::from(Span::styled(name, styles.primary_text())),
            Line::from(""),
        ]
        .into_iter()
        .chain(content)
        .collect::<Vec<_>>();
        let widget = Paragraph::new(lines)
            .alignment(Alignment::Center)
            .block(styles.panel_block("Cover"))
            .wrap(ratatui::widgets::Wrap { trim: true });
        f.render_widget(widget, area);
    }

    fn render_metadata_panel(&self, f: &mut Frame, area: Rect, styles: &RommStyles) {
        let meta_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(3), Constraint::Min(1)])
            .split(area);

        let titles = DetailTab::ALL
            .iter()
            .map(|tab| Line::from(Span::raw(tab.title())))
            .collect::<Vec<_>>();
        let tabs = Tabs::new(titles)
            .select(self.active_tab.index())
            .block(styles.panel_block_untitled())
            .style(styles.muted())
            .highlight_style(styles.selection());
        f.render_widget(tabs, meta_chunks[0]);

        match self.active_tab {
            DetailTab::Info => self.render_info_tab(f, meta_chunks[1], styles),
            DetailTab::Saves => self.render_saves_tab(f, meta_chunks[1], styles),
            DetailTab::Achievements => self.render_achievements_tab(f, meta_chunks[1], styles),
        }
    }

    fn render_info_tab(&self, f: &mut Frame, area: Rect, styles: &RommStyles) {
        let title = self.rom.name.as_str();
        let platform = self
            .rom
            .platform_display_name
            .as_deref()
            .or(self.rom.platform_custom_name.as_deref())
            .unwrap_or("—");
        let summary = self.rom.summary.as_deref().unwrap_or("").trim();
        let path = self.rom.fs_path.as_str();
        let size = format_size(self.rom.fs_size_bytes);
        let mut lines: Vec<Line> = vec![
            Line::from(vec![
                Span::styled("Title: ", styles.label()),
                Span::raw(title),
            ]),
            Line::from(vec![
                Span::styled("Platform: ", styles.label()),
                Span::raw(platform),
            ]),
            Line::from(""),
            Line::from(Span::styled("Overview:", styles.label())),
            Line::from(vec![
                Span::styled("Download: ", styles.muted()),
                Span::raw(if self.has_started_download {
                    "Started"
                } else {
                    "Not started"
                }),
            ]),
            Line::from(vec![
                Span::styled("Cover URL: ", styles.muted()),
                Span::raw(if self.rom.url_cover.is_some() {
                    "Available (o to open)"
                } else {
                    "Missing"
                }),
            ]),
            Line::from(""),
            Line::from(vec![Span::styled("Summary: ", styles.label())]),
            Line::from(if summary.is_empty() { "—" } else { summary }),
            Line::from(""),
            Line::from(vec![
                Span::styled("File: ", styles.label()),
                Span::raw(path),
            ]),
            Line::from(vec![
                Span::styled("Size: ", styles.label()),
                Span::raw(size),
            ]),
        ];

        if !self.other_files.is_empty() {
            lines.push(Line::from(""));
            lines.push(Line::from(vec![
                Span::styled("Other files (updates/DLC): ", styles.label()),
                Span::raw(format!("{} file(s)", self.other_files.len())),
            ]));
            for other in self.other_files.iter().take(10) {
                let label = other.fs_name.as_str();
                lines.push(Line::from(format!("  • {}", label)));
            }
            if self.other_files.len() > 10 {
                lines.push(Line::from(format!(
                    "  … and {} more",
                    self.other_files.len() - 10
                )));
            }
        }

        if self.show_technical {
            lines.push(Line::from(""));
            lines.push(Line::from(Span::styled("Technical:", styles.warning())));
            lines.push(Line::from(format!("  ID: {}", self.rom.id)));
            lines.push(Line::from(format!(
                "  Platform ID: {}",
                self.rom.platform_id
            )));
            if let Some(s) = &self.rom.slug {
                lines.push(Line::from(format!("  Slug: {}", s)));
            }
            lines.push(Line::from(format!(
                "  Identified: {}",
                self.rom.is_identified
            )));
        }

        let block = styles.panel_block("Info");
        let p = Paragraph::new(lines)
            .block(block)
            .style(styles.text())
            .wrap(ratatui::widgets::Wrap { trim: true });
        f.render_widget(p, area);
    }

    fn render_saves_tab(&self, f: &mut Frame, area: Rect, styles: &RommStyles) {
        let block = styles.panel_block("Saves");
        let inner = block.inner(area);
        let visible_height = inner.height as usize;
        let lines = save_lines(&self.saves_state, self.selected_save_index);
        let start = self
            .selected_save_index
            .saturating_sub(visible_height.saturating_sub(1));
        let windowed: Vec<_> = lines.into_iter().skip(start).take(visible_height).collect();
        let p = Paragraph::new(windowed).block(block).style(styles.text());
        f.render_widget(p, area);
    }

    fn render_achievements_tab(&self, f: &mut Frame, area: Rect, styles: &RommStyles) {
        let block = styles.panel_block("Achievements");
        let inner = block.inner(area);
        let visible_height = inner.height as usize;
        let lines = achievement_lines(&self.achievements_state);
        let start = self
            .achievement_scroll_offset
            .min(lines.len().saturating_sub(1));
        let windowed: Vec<_> = lines.into_iter().skip(start).take(visible_height).collect();
        let p = Paragraph::new(windowed).block(block).style(styles.text());
        f.render_widget(p, area);
    }

    fn render_footer_panel(&mut self, f: &mut Frame, footer_area: Rect, styles: &RommStyles) {
        self.tick_message();
        // Footer: show progress bar if downloading, otherwise help text.
        if let Some(job) = self.active_download() {
            let (label, style) = match &job.status {
                DownloadStatus::Downloading => {
                    (format!("Downloading… {}%", job.percent()), styles.label())
                }
                DownloadStatus::Done => ("Download complete".to_string(), styles.success()),
                DownloadStatus::SkippedAlreadyExists => {
                    ("Already present (skipped)".to_string(), styles.warning())
                }
                DownloadStatus::Cancelled => ("Download cancelled".to_string(), styles.warning()),
                DownloadStatus::FinalizeFailed(msg) => (
                    format!("Finalize failed: {}", truncate(msg, 40)),
                    styles.error(),
                ),
                DownloadStatus::Error(msg) => {
                    (format!("Error: {}", truncate(msg, 50)), styles.error())
                }
            };
            let gauge = Gauge::default()
                .block(styles.panel_block_untitled())
                .gauge_style(style)
                .percent(job.percent())
                .label(label);
            f.render_widget(gauge, footer_area);
        } else if let Some(msg) = self.message.as_deref() {
            let footer = Paragraph::new(msg)
                .style(styles.footer_hint())
                .block(styles.panel_block_untitled());
            f.render_widget(footer, footer_area);
        } else {
            render_footer_panel(f, footer_area, styles, self.footer_help_entries(), None);
        }
    }
}
