use ratatui::layout::{Alignment, Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Gauge, Paragraph};
use ratatui::Frame;
use ratatui_image::picker::{Picker, ProtocolType};
use ratatui_image::protocol::StatefulProtocol;
use ratatui_image::{Resize, StatefulImage};
use std::sync::{Arc, Mutex};

use crate::core::download::{DownloadJob, DownloadStatus};
use crate::core::utils::format_size;
use crate::tui::utils::{open_in_browser, truncate};
use crate::types::Rom;

use super::{LibraryBrowseScreen, SearchScreen};

/// Previous screen when opening game detail (so Esc can return).
pub enum GameDetailPrevious {
    Library(LibraryBrowseScreen),
    Search(SearchScreen),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CoverRenderMode {
    Auto,
    InlineImage,
    TextFallback,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CoverState {
    Idle,
    Loading,
    Ready,
    Failed(String),
}

/// Detailed view for a single ROM (and its related files).
pub struct GameDetailScreen {
    pub rom: Rom,
    /// Other files for the same game (updates, DLC).
    pub other_files: Vec<Rom>,
    pub previous: GameDetailPrevious,
    pub show_technical: bool,
    pub message: Option<String>,
    /// Shared download list — used to show inline progress for this ROM.
    pub downloads: Arc<Mutex<Vec<DownloadJob>>>,
    /// Whether a download has been started from this detail view.
    pub has_started_download: bool,
    /// Whether the user has acknowledged the download completion message.
    pub download_completion_acknowledged: bool,
    pub cover_render_mode: CoverRenderMode,
    pub cover_state: CoverState,
    pub cover_last_url: Option<String>,
    pub cover_protocol: Option<ProtocolType>,
    pub cover_image: Option<StatefulProtocol>,
}

impl GameDetailScreen {
    pub fn new(
        rom: Rom,
        other_files: Vec<Rom>,
        previous: GameDetailPrevious,
        downloads: Arc<Mutex<Vec<DownloadJob>>>,
    ) -> Self {
        let cover_last_url = rom.url_cover.clone();
        let cover_protocol = detect_cover_protocol();
        let cover_state = if cover_last_url.is_none() {
            CoverState::Idle
        } else {
            CoverState::Loading
        };
        Self {
            rom,
            other_files,
            previous,
            show_technical: false,
            message: None,
            downloads,
            has_started_download: false,
            download_completion_acknowledged: false,
            cover_render_mode: CoverRenderMode::Auto,
            cover_state,
            cover_last_url,
            cover_protocol,
            cover_image: None,
        }
    }

    pub fn toggle_technical(&mut self) {
        self.show_technical = !self.show_technical;
    }

    pub fn open_cover(&mut self) {
        self.message = None;
        let url = self.rom.url_cover.as_deref().filter(|s| !s.is_empty());
        match url {
            Some(u) => match open_in_browser(u) {
                Ok(_) => self.message = Some("Opened in browser".to_string()),
                Err(e) => self.message = Some(format!("Failed: {}", e)),
            },
            None => self.message = Some("No cover URL".to_string()),
        }
    }

    pub fn clear_message(&mut self) {
        self.message = None;
    }

    pub fn should_request_cover_load(&self) -> bool {
        self.cover_last_url.is_some()
    }

    pub fn set_cover_loading(&mut self) {
        if self.should_request_cover_load() {
            self.cover_state = CoverState::Loading;
        }
    }

    pub fn apply_cover_image(&mut self, image: image::DynamicImage) {
        let mut picker = Picker::halfblocks();
        if let Some(protocol) = self.cover_protocol {
            picker.set_protocol_type(protocol);
        }
        self.cover_image = Some(picker.new_resize_protocol(image));
        self.cover_state = CoverState::Ready;
    }

    pub fn apply_cover_error(&mut self, message: String) {
        self.cover_image = None;
        self.cover_state = CoverState::Failed(message);
    }

    fn footer_help_text(&self) -> &'static str {
        if self.show_technical {
            "Enter: Download | o: Open cover | m: Hide technical | Esc: Back"
        } else {
            "Enter: Download | o: Open cover | m: More technical details | Esc: Back"
        }
    }

    fn cover_pipeline_label(&self) -> &'static str {
        if self.cover_protocol.is_some() {
            "Advanced terminal protocol"
        } else {
            "Halfblocks fallback mode"
        }
    }

    /// Find the most recent download job for this ROM (if any).
    /// Returns downloading jobs always, or completed/errored jobs if not yet acknowledged.
    fn active_download(&self) -> Option<DownloadJob> {
        self.downloads.lock().ok().and_then(|list| {
            list.iter()
                .rev()
                .find(|j| {
                    j.rom_id == self.rom.id
                        && (matches!(j.status, DownloadStatus::Downloading)
                            || (!self.download_completion_acknowledged
                                && matches!(
                                    j.status,
                                    DownloadStatus::Done
                                        | DownloadStatus::SkippedAlreadyExists
                                        | DownloadStatus::FinalizeFailed(_)
                                        | DownloadStatus::Error(_)
                                )))
                })
                .cloned()
        })
    }

    pub fn render(&mut self, f: &mut Frame, area: Rect) {
        let chunks = Layout::default()
            .constraints([Constraint::Min(10), Constraint::Length(3)])
            .direction(Direction::Vertical)
            .split(area);
        let body = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Min(10), Constraint::Length(42)])
            .split(chunks[0]);

        self.render_metadata_panel(f, body[0]);
        self.render_cover_panel(f, body[1]);
        self.render_footer_panel(f, chunks[1]);
    }

    fn render_cover_panel(&mut self, f: &mut Frame, area: Rect) {
        let platform = self
            .rom
            .platform_display_name
            .as_deref()
            .or(self.rom.platform_custom_name.as_deref())
            .unwrap_or("—");
        let name = truncate(&self.rom.name, 28);
        if matches!(self.cover_state, CoverState::Ready) {
            if let Some(image_state) = self.cover_image.as_mut() {
                let block = Block::default().title("Cover").borders(Borders::ALL);
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
                Line::from(Span::styled(
                    "Inline cover ready",
                    Style::default().fg(Color::Green),
                )),
                Line::from(""),
                Line::from(self.cover_pipeline_label()),
                Line::from("Press o for browser view"),
            ],
            CoverState::Loading => vec![
                Line::from(""),
                Line::from(Span::styled(
                    "Loading cover...",
                    Style::default().fg(Color::Yellow),
                )),
                Line::from(""),
                Line::from("Fetching image"),
                Line::from("in background"),
            ],
            CoverState::Failed(message) => vec![
                Line::from(""),
                Line::from(Span::styled("Cover unavailable", Style::default().fg(Color::Red))),
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
            Line::from(Span::styled(
                format!("[{}]", platform),
                Style::default().fg(Color::Cyan),
            )),
            Line::from(Span::styled(name, Style::default().fg(Color::White))),
            Line::from(""),
        ]
        .into_iter()
        .chain(content)
        .collect::<Vec<_>>();
        let widget = Paragraph::new(lines)
            .alignment(Alignment::Center)
            .block(Block::default().title("Cover").borders(Borders::ALL))
            .wrap(ratatui::widgets::Wrap { trim: true });
        f.render_widget(widget, area);
    }

    fn render_metadata_panel(&self, f: &mut Frame, area: Rect) {
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
                Span::styled("Title: ", Style::default().fg(Color::Cyan)),
                Span::raw(title),
            ]),
            Line::from(vec![
                Span::styled("Platform: ", Style::default().fg(Color::Cyan)),
                Span::raw(platform),
            ]),
            Line::from(""),
            Line::from(Span::styled(
                "Overview:",
                Style::default().fg(Color::Cyan),
            )),
            Line::from(vec![
                Span::styled("Download: ", Style::default().fg(Color::Gray)),
                Span::raw(if self.has_started_download {
                    "Started"
                } else {
                    "Not started"
                }),
            ]),
            Line::from(vec![
                Span::styled("Cover URL: ", Style::default().fg(Color::Gray)),
                Span::raw(if self.rom.url_cover.is_some() {
                    "Available (o to open)"
                } else {
                    "Missing"
                }),
            ]),
            Line::from(""),
            Line::from(vec![Span::styled(
                "Summary: ",
                Style::default().fg(Color::Cyan),
            )]),
            Line::from(if summary.is_empty() { "—" } else { summary }),
            Line::from(""),
            Line::from(vec![
                Span::styled("File: ", Style::default().fg(Color::Cyan)),
                Span::raw(path),
            ]),
            Line::from(vec![
                Span::styled("Size: ", Style::default().fg(Color::Cyan)),
                Span::raw(size),
            ]),
        ];

        if !self.other_files.is_empty() {
            lines.push(Line::from(""));
            lines.push(Line::from(vec![
                Span::styled(
                    "Other files (updates/DLC): ",
                    Style::default().fg(Color::Cyan),
                ),
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
            lines.push(Line::from(Span::styled(
                "Technical:",
                Style::default().fg(Color::Yellow),
            )));
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

        let block = Block::default().title("Game detail").borders(Borders::ALL);
        let p = Paragraph::new(lines)
            .block(block)
            .wrap(ratatui::widgets::Wrap { trim: true });
        f.render_widget(p, area);
    }

    fn render_footer_panel(&self, f: &mut Frame, footer_area: Rect) {
        // Footer: show progress bar if downloading, otherwise help text.
        if let Some(job) = self.active_download() {
            let (label, style) = match &job.status {
                DownloadStatus::Downloading => (
                    format!("Downloading… {}%", job.percent()),
                    Style::default().fg(Color::Cyan),
                ),
                DownloadStatus::Done => (
                    "Download complete".to_string(),
                    Style::default().fg(Color::Green),
                ),
                DownloadStatus::SkippedAlreadyExists => (
                    "Already present (skipped)".to_string(),
                    Style::default().fg(Color::Yellow),
                ),
                DownloadStatus::FinalizeFailed(msg) => (
                    format!("Finalize failed: {}", truncate(msg, 40)),
                    Style::default().fg(Color::Red),
                ),
                DownloadStatus::Error(msg) => (
                    format!("Error: {}", truncate(msg, 50)),
                    Style::default().fg(Color::Red),
                ),
            };
            let gauge = Gauge::default()
                .block(Block::default().borders(Borders::ALL))
                .gauge_style(style)
                .percent(job.percent())
                .label(label);
            f.render_widget(gauge, footer_area);
        } else {
            let msg = self.message.as_deref().unwrap_or(self.footer_help_text());
            let footer = Paragraph::new(msg).block(Block::default().borders(Borders::ALL));
            f.render_widget(footer, footer_area);
        }
    }
}

fn detect_cover_protocol() -> Option<ProtocolType> {
    detect_cover_protocol_from_env(
        std::env::var("TERM_PROGRAM").ok(),
        std::env::var("TERM").ok(),
        std::env::var("KITTY_WINDOW_ID").ok(),
    )
}

fn detect_cover_protocol_from_env(
    term_program: Option<String>,
    term: Option<String>,
    kitty_window_id: Option<String>,
) -> Option<ProtocolType> {
    let term_program = term_program.unwrap_or_default();
    if kitty_window_id.is_some_and(|v| !v.is_empty()) {
        return Some(ProtocolType::Kitty);
    }
    if term_program.contains("iTerm")
        || term_program.contains("WezTerm")
        || term_program.contains("mintty")
        || term_program.contains("vscode")
        || term_program.contains("Tabby")
        || term_program.contains("Hyper")
        || term_program.contains("rio")
        || term_program.contains("WarpTerminal")
    {
        return Some(ProtocolType::Iterm2);
    }
    if term.is_some_and(|v| v.to_ascii_lowercase().contains("sixel"))
    {
        return Some(ProtocolType::Sixel);
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detect_cover_protocol_prefers_kitty_hint() {
        let protocol = detect_cover_protocol_from_env(
            Some("iTerm.app".into()),
            Some("xterm-256color".into()),
            Some("123".into()),
        );
        assert_eq!(protocol, Some(ProtocolType::Kitty));
    }

    #[test]
    fn detect_cover_protocol_supports_sixel_term() {
        let protocol =
            detect_cover_protocol_from_env(None, Some("xterm+sixel".into()), Some(String::new()));
        assert_eq!(protocol, Some(ProtocolType::Sixel));
    }

    #[test]
    fn missing_protocol_still_requests_cover_load() {
        let rom = crate::types::Rom {
            id: 5,
            platform_id: 1,
            platform_slug: None,
            platform_fs_slug: None,
            platform_custom_name: None,
            platform_display_name: None,
            fs_name: "game.zip".to_string(),
            fs_name_no_tags: "game".to_string(),
            fs_name_no_ext: "game".to_string(),
            fs_extension: "zip".to_string(),
            fs_path: "/game.zip".to_string(),
            fs_size_bytes: 10,
            name: "game".to_string(),
            slug: None,
            summary: None,
            path_cover_small: None,
            path_cover_large: None,
            url_cover: Some("http://example.com/cover.png".to_string()),
            is_unidentified: false,
            is_identified: true,
        };
        let previous = GameDetailPrevious::Search(SearchScreen::new());
        let downloads = Arc::new(Mutex::new(Vec::new()));
        let mut detail = GameDetailScreen::new(rom, Vec::new(), previous, downloads);
        detail.cover_protocol = None;
        assert!(detail.should_request_cover_load());
        detail.set_cover_loading();
        assert_eq!(detail.cover_state, CoverState::Loading);
    }

    #[test]
    fn footer_help_text_tracks_technical_mode() {
        let rom = crate::types::Rom {
            id: 1,
            platform_id: 1,
            platform_slug: None,
            platform_fs_slug: None,
            platform_custom_name: None,
            platform_display_name: None,
            fs_name: "game.zip".to_string(),
            fs_name_no_tags: "game".to_string(),
            fs_name_no_ext: "game".to_string(),
            fs_extension: "zip".to_string(),
            fs_path: "/game.zip".to_string(),
            fs_size_bytes: 10,
            name: "game".to_string(),
            slug: None,
            summary: None,
            path_cover_small: None,
            path_cover_large: None,
            url_cover: None,
            is_unidentified: false,
            is_identified: true,
        };
        let previous = GameDetailPrevious::Search(SearchScreen::new());
        let downloads = Arc::new(Mutex::new(Vec::new()));
        let mut detail = GameDetailScreen::new(rom, Vec::new(), previous, downloads);
        let non_technical = detail.footer_help_text();
        assert!(non_technical.contains("More technical details"));

        detail.show_technical = true;
        let technical = detail.footer_help_text();
        assert!(technical.contains("Hide technical"));
    }

    #[test]
    fn cover_state_transitions_to_ready_and_error() {
        let rom = crate::types::Rom {
            id: 1,
            platform_id: 1,
            platform_slug: None,
            platform_fs_slug: None,
            platform_custom_name: None,
            platform_display_name: None,
            fs_name: "game.zip".to_string(),
            fs_name_no_tags: "game".to_string(),
            fs_name_no_ext: "game".to_string(),
            fs_extension: "zip".to_string(),
            fs_path: "/game.zip".to_string(),
            fs_size_bytes: 10,
            name: "game".to_string(),
            slug: None,
            summary: None,
            path_cover_small: None,
            path_cover_large: None,
            url_cover: Some("http://example.com/cover.png".to_string()),
            is_unidentified: false,
            is_identified: true,
        };
        let previous = GameDetailPrevious::Search(SearchScreen::new());
        let downloads = Arc::new(Mutex::new(Vec::new()));
        let mut detail = GameDetailScreen::new(rom, Vec::new(), previous, downloads);
        detail.cover_protocol = Some(ProtocolType::Iterm2);
        detail.set_cover_loading();
        assert_eq!(detail.cover_state, CoverState::Loading);

        detail.apply_cover_image(image::DynamicImage::new_rgba8(4, 4));
        assert_eq!(detail.cover_state, CoverState::Ready);
        assert!(detail.cover_image.is_some());

        detail.apply_cover_error("oops".to_string());
        assert_eq!(detail.cover_state, CoverState::Failed("oops".to_string()));
    }
}
