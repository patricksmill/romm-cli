use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::widgets::{Gauge, Paragraph};
use ratatui::Frame;
use std::sync::{Arc, Mutex};

use crate::core::download::{DownloadJob, DownloadStatus, ExtrasJob, ExtrasJobStatus};
use crate::tui::theme::RommStyles;
use crate::tui::utils::truncate;

/// Overlay screen listing active and completed downloads.
///
/// This screen is read-only; it observes `DownloadJob`s and composite `ExtrasJob`s from
/// [`DownloadManager`](crate::core::download::DownloadManager).
pub struct DownloadScreen {
    pub downloads: Arc<Mutex<Vec<DownloadJob>>>,
    pub extras_jobs: Arc<Mutex<Vec<ExtrasJob>>>,
}

impl DownloadScreen {
    pub fn new(
        downloads: Arc<Mutex<Vec<DownloadJob>>>,
        extras_jobs: Arc<Mutex<Vec<ExtrasJob>>>,
    ) -> Self {
        Self {
            downloads,
            extras_jobs,
        }
    }

    pub fn render(&self, f: &mut Frame, area: Rect, styles: &RommStyles) {
        let chunks = Layout::default()
            .constraints([Constraint::Min(3), Constraint::Length(3)])
            .direction(ratatui::layout::Direction::Vertical)
            .split(area);

        let jobs = match self.downloads.lock() {
            Ok(guard) => guard.clone(),
            Err(err) => {
                eprintln!("warning: download list lock poisoned: {}", err);
                Vec::new()
            }
        };
        let extras = match self.extras_jobs.lock() {
            Ok(guard) => guard.clone(),
            Err(err) => {
                eprintln!("warning: extras job list lock poisoned: {}", err);
                Vec::new()
            }
        };

        let block = styles.panel_block("Downloads (d: close)");

        if jobs.is_empty() && extras.is_empty() {
            let p = Paragraph::new(
                "No downloads. Press Enter on a game detail for the ROM, or e for extras.",
            )
            .block(block);
            f.render_widget(p, chunks[0]);
        } else {
            let inner = block.inner(chunks[0]);
            let max_rows = inner.height as usize;
            let rows = Layout::default()
                .constraints(
                    (0..max_rows.max(1))
                        .map(|_| Constraint::Length(1))
                        .collect::<Vec<_>>(),
                )
                .direction(ratatui::layout::Direction::Vertical)
                .split(inner);

            f.render_widget(block, chunks[0]);

            let mut row_i = 0usize;

            for job in jobs.iter() {
                if row_i >= max_rows {
                    break;
                }
                if let Some(row_area) = rows.get(row_i) {
                    let percent = job.percent();
                    let (label, gauge_style) = match &job.status {
                        DownloadStatus::Downloading => (format!("{}%", percent), styles.label()),
                        DownloadStatus::Done => ("Done".into(), styles.success()),
                        DownloadStatus::SkippedAlreadyExists => {
                            ("Skipped (already exists)".into(), styles.warning())
                        }
                        DownloadStatus::Cancelled => ("Cancelled".into(), styles.warning()),
                        DownloadStatus::FinalizeFailed(msg) => (
                            format!("Finalize failed: {}", truncate(msg, 40)),
                            styles.error(),
                        ),
                        DownloadStatus::Error(msg) => {
                            (format!("Error: {}", truncate(msg, 50)), styles.error())
                        }
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
                    f.render_widget(
                        Paragraph::new(line.as_str()).style(styles.text()),
                        line_area,
                    );
                    if gauge_width > 0 {
                        f.render_widget(gauge, gauge_area);
                    }
                }
                row_i += 1;
            }

            for job in extras.iter() {
                if row_i >= max_rows {
                    break;
                }
                if let Some(row_area) = rows.get(row_i) {
                    let percent = job.percent();
                    let (label, gauge_style) = match &job.status {
                        ExtrasJobStatus::Running => (
                            format!("{}% {}/{}", percent, job.completed_items, job.total_items),
                            styles.label(),
                        ),
                        ExtrasJobStatus::Done => ("Done".into(), styles.success()),
                        ExtrasJobStatus::PartialFailure(n) => {
                            (format!("Partial ({n} failed)"), styles.warning())
                        }
                        ExtrasJobStatus::AllFailed => ("All failed".into(), styles.error()),
                    };
                    let gauge = Gauge::default()
                        .gauge_style(gauge_style)
                        .percent(percent)
                        .label(label);

                    let line = format!("Extras | {} | ", truncate(&job.name, 36),);
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
                    f.render_widget(
                        Paragraph::new(line.as_str()).style(styles.text()),
                        line_area,
                    );
                    if gauge_width > 0 {
                        f.render_widget(gauge, gauge_area);
                    }
                }
                row_i += 1;
            }
        }

        let help = "d or Esc: Back to previous screen";
        let footer = Paragraph::new(help)
            .style(styles.footer_hint())
            .block(styles.panel_block_untitled());
        f.render_widget(footer, chunks[1]);
    }
}
