use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::{Color, Style};
use ratatui::widgets::{Block, Borders, Gauge, Paragraph};
use ratatui::Frame;
use std::sync::{Arc, Mutex};

use crate::tui::download::{DownloadJob, DownloadStatus};

pub struct DownloadScreen {
    pub downloads: Arc<Mutex<Vec<DownloadJob>>>,
}

impl DownloadScreen {
    pub fn new(downloads: Arc<Mutex<Vec<DownloadJob>>>) -> Self {
        Self { downloads }
    }

    pub fn render(&self, f: &mut Frame, area: Rect) {
        let chunks = Layout::default()
            .constraints([Constraint::Min(3), Constraint::Length(3)])
            .direction(ratatui::layout::Direction::Vertical)
            .split(area);

        let jobs = self.downloads.lock().unwrap().clone();
        let block = Block::default()
            .title("Downloads (d: close)")
            .borders(Borders::ALL);

        if jobs.is_empty() {
            let p = Paragraph::new("No downloads. Press Enter on a game detail to start a download.")
                .block(block);
            f.render_widget(p, chunks[0]);
        } else {
            let inner = block.inner(chunks[0]);
            let max_rows = inner.height as usize;
            let visible: Vec<_> = jobs.iter().take(max_rows).collect();
            let n = visible.len().max(1);
            let rows = Layout::default()
                .constraints(
                    (0..n).map(|_| Constraint::Length(1)).collect::<Vec<_>>(),
                )
                .direction(ratatui::layout::Direction::Vertical)
                .split(inner);

            f.render_widget(block, chunks[0]);

            for (i, job) in visible.iter().enumerate() {
                if let Some(row_area) = rows.get(i) {
                    let percent = job.percent();
                    let (label, gauge_style) = match &job.status {
                        DownloadStatus::Downloading => {
                            (format!("{}%", percent), Style::default().fg(Color::Cyan))
                        }
                        DownloadStatus::Done => ("Done".into(), Style::default().fg(Color::Green)),
                        DownloadStatus::Error(msg) => (
                            format!("Error: {}", truncate(msg, 50)),
                            Style::default().fg(Color::Red),
                        ),
                    };
                    let gauge = Gauge::default()
                        .gauge_style(gauge_style)
                        .percent(percent)
                        .label(label);

                    let line = format!(
                        "{} | {} | ",
                        truncate(&job.name, 30),
                        truncate(&job.platform, 15)
                    );
                    let line_len = line.chars().count().min(row_area.width as usize) as u16;
                    let line_area = Rect {
                        x: row_area.x,
                        y: row_area.y,
                        width: line_len,
                        height: 1,
                    };
                    let gauge_width = row_area.width.saturating_sub(line_len);
                    let gauge_area = Rect {
                        x: row_area.x + line_len,
                        y: row_area.y,
                        width: gauge_width,
                        height: 1,
                    };
                    f.render_widget(Paragraph::new(line.as_str()), line_area);
                    if gauge_width > 0 {
                        f.render_widget(gauge, gauge_area);
                    }
                }
            }
        }

        let help = "d or Esc: Back to previous screen";
        let footer = Paragraph::new(help).block(Block::default().borders(Borders::ALL));
        f.render_widget(footer, chunks[1]);
    }
}

fn truncate(s: &str, max: usize) -> String {
    let s = s.trim();
    if s.chars().count() <= max {
        s.to_string()
    } else {
        format!("{}…", s.chars().take(max.saturating_sub(1)).collect::<String>())
    }
}
