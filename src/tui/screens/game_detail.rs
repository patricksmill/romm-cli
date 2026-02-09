use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::{Color, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph};
use ratatui::Frame;

use crate::tui::utils;
use crate::types::Rom;

use super::{LibraryBrowseScreen, SearchScreen};

/// Previous screen when opening game detail (so Esc can return).
pub enum GameDetailPrevious {
    Library(LibraryBrowseScreen),
    Search(SearchScreen),
}

pub struct GameDetailScreen {
    pub rom: Rom,
    /// Other files for the same game (updates, DLC).
    pub other_files: Vec<Rom>,
    pub previous: GameDetailPrevious,
    pub show_technical: bool,
    pub message: Option<String>,
}

impl GameDetailScreen {
    pub fn new(rom: Rom, other_files: Vec<Rom>, previous: GameDetailPrevious) -> Self {
        Self {
            rom,
            other_files,
            previous,
            show_technical: false,
            message: None,
        }
    }

    pub fn toggle_technical(&mut self) {
        self.show_technical = !self.show_technical;
    }

    pub fn open_cover(&mut self) {
        self.message = None;
        let url = self
            .rom
            .url_cover
            .as_deref()
            .filter(|s| !s.is_empty());
        match url {
            Some(u) => {
                match utils::open_in_browser(u) {
                    Ok(_) => self.message = Some("Opened in browser".to_string()),
                    Err(e) => self.message = Some(format!("Failed: {}", e)),
                }
            }
            None => self.message = Some("No cover URL".to_string()),
        }
    }

    pub fn clear_message(&mut self) {
        self.message = None;
    }

    fn format_size(&self, bytes: u64) -> String {
        const KB: u64 = 1024;
        const MB: u64 = KB * 1024;
        const GB: u64 = MB * 1024;
        if bytes >= GB {
            format!("{:.2} GB", bytes as f64 / GB as f64)
        } else if bytes >= MB {
            format!("{:.2} MB", bytes as f64 / MB as f64)
        } else if bytes >= KB {
            format!("{:.2} KB", bytes as f64 / KB as f64)
        } else {
            format!("{} B", bytes)
        }
    }

    pub fn render(&self, f: &mut Frame, area: Rect) {
        let chunks = Layout::default()
            .constraints([Constraint::Min(10), Constraint::Length(3)])
            .direction(ratatui::layout::Direction::Vertical)
            .split(area);

        let title = self.rom.name.as_str();
        let platform = self
            .rom
            .platform_display_name
            .as_deref()
            .or(self.rom.platform_custom_name.as_deref())
            .unwrap_or("—");
        let summary = self.rom.summary.as_deref().unwrap_or("").trim();
        let path = self.rom.fs_path.as_str();
        let size = self.format_size(self.rom.fs_size_bytes);
        let cover_text = if self.rom.url_cover.is_some() {
            "[Cover] (o: open in browser)"
        } else {
            "No cover"
        };

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
            Line::from(vec![
                Span::styled("Cover: ", Style::default().fg(Color::Cyan)),
                Span::raw(cover_text),
            ]),
            Line::from(""),
            Line::from(vec![
                Span::styled("Summary: ", Style::default().fg(Color::Cyan)),
            ]),
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
            lines.push(Line::from(format!("  Platform ID: {}", self.rom.platform_id)));
            if let Some(s) = &self.rom.slug {
                lines.push(Line::from(format!("  Slug: {}", s)));
            }
            lines.push(Line::from(format!(
                "  Identified: {}",
                self.rom.is_identified
            )));
        }

        let block = Block::default()
            .title("Game detail")
            .borders(Borders::ALL);
        let p = Paragraph::new(lines)
            .block(block)
            .wrap(ratatui::widgets::Wrap { trim: true });
        f.render_widget(p, chunks[0]);

        let help = if self.show_technical {
            "Enter: Download | o: Open cover | m: Hide technical | Esc: Back"
        } else {
            "Enter: Download | o: Open cover | m: More technical details | Esc: Back"
        };
        let msg = self.message.as_deref().unwrap_or(help);
        let footer = Paragraph::new(msg).block(Block::default().borders(Borders::ALL));
        f.render_widget(footer, chunks[1]);
    }
}
