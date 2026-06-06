use std::sync::{Arc, Mutex};
use std::time::Instant;

use ratatui_image::picker::{Picker, ProtocolType};

use crate::core::download::{DownloadJob, DownloadStatus};
use crate::core::extras::collect_update_dlc_files;
use crate::tui::path_picker::{PathPicker, PathPickerMode};
use crate::tui::utils::open_in_browser;
use crate::types::{Rom, SaveMetadata};

use super::cover::detect_cover_protocol;
use super::types::{
    CoverRenderMode, CoverState, GameDetailPrevious, GameDetailScreen, SaveListState,
    COVER_PANEL_WIDTH_MAX, COVER_PANEL_WIDTH_MIN,
};

impl GameDetailScreen {
    pub fn new(
        rom: Rom,
        other_files: Vec<Rom>,
        previous: GameDetailPrevious,
        downloads: Arc<Mutex<Vec<DownloadJob>>>,
        cover_panel_width: u16,
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
            message_clear_at: None,
            downloads,
            has_started_download: false,
            download_completion_acknowledged: false,
            cover_render_mode: CoverRenderMode::Auto,
            cover_state,
            cover_last_url,
            cover_protocol,
            cover_image: None,
            saves_state: SaveListState::Idle,
            selected_save_index: 0,
            save_upload_picker: None,
            cover_panel_width: cover_panel_width
                .clamp(COVER_PANEL_WIDTH_MIN, COVER_PANEL_WIDTH_MAX),
        }
    }

    pub fn adjust_cover_panel_width(&mut self, delta: i16) {
        let next = (self.cover_panel_width as i16 + delta)
            .clamp(COVER_PANEL_WIDTH_MIN as i16, COVER_PANEL_WIDTH_MAX as i16);
        self.cover_panel_width = next as u16;
    }

    pub fn toggle_technical(&mut self) {
        self.show_technical = !self.show_technical;
    }

    pub fn open_cover(&mut self) {
        self.message = None;
        self.message_clear_at = None;
        let url = self.rom.url_cover.as_deref().filter(|s| !s.is_empty());
        match url {
            Some(u) => match open_in_browser(u) {
                Ok(_) => {
                    self.message = Some("Opened in browser".to_string());
                    self.message_clear_at =
                        Some(Instant::now() + std::time::Duration::from_secs(3));
                }
                Err(e) => {
                    self.message = Some(format!("Failed: {}", e));
                    self.message_clear_at =
                        Some(Instant::now() + std::time::Duration::from_secs(5));
                }
            },
            None => {
                self.message = Some("No cover URL".to_string());
                self.message_clear_at = Some(Instant::now() + std::time::Duration::from_secs(3));
            }
        }
    }

    pub fn clear_message(&mut self) {
        self.message = None;
        self.message_clear_at = None;
    }

    pub fn tick_message(&mut self) {
        if let Some(clear_at) = self.message_clear_at {
            if Instant::now() >= clear_at {
                self.message = None;
                self.message_clear_at = None;
            }
        }
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
        let picker = match self.cover_protocol {
            None => Picker::halfblocks(),
            Some(env_protocol) => match Picker::from_query_stdio() {
                Ok(mut p) => {
                    // ratatui-image ties pixel budget to protocol + font_size together. After
                    // `from_query_stdio`, protocol and cell size both come from the terminal when
                    // the query succeeds. Replacing the queried protocol with a different env-only
                    // guess (e.g. host vs PTY layer disagree) mis-maps pixels to cells and clips or
                    // gaps the image even when `Resize::Fit` is correct.
                    if matches!(env_protocol, ProtocolType::Kitty) {
                        p.set_protocol_type(ProtocolType::Kitty);
                    } else if p.protocol_type() == ProtocolType::Halfblocks {
                        p.set_protocol_type(env_protocol);
                    }
                    p
                }
                Err(_) => {
                    let mut p = Picker::halfblocks();
                    p.set_protocol_type(env_protocol);
                    p
                }
            },
        };
        self.cover_image = Some(picker.new_resize_protocol(image));
        self.cover_state = CoverState::Ready;
    }

    pub fn apply_cover_error(&mut self, message: String) {
        self.cover_image = None;
        self.cover_state = CoverState::Failed(message);
    }

    pub(crate) fn footer_help_text(&self) -> &'static str {
        if self.show_technical {
            "Enter: Download ROM | e: Extras | u: Upload save | D: Download save | m: Hide technical | Ctrl+←/→: Resize cover | Esc: Back"
        } else {
            "Enter: Download ROM | e: Extras | u: Upload save | D: Download save | m: More technical details | Ctrl+←/→: Resize cover | Esc: Back"
        }
    }

    pub fn set_saves_loading(&mut self) {
        self.saves_state = SaveListState::Loading;
    }

    pub fn apply_saves(&mut self, saves: Vec<SaveMetadata>) {
        self.selected_save_index = self.selected_save_index.min(saves.len().saturating_sub(1));
        self.saves_state = SaveListState::Loaded(saves);
    }

    pub fn apply_saves_error(&mut self, error: String) {
        self.saves_state = SaveListState::Failed(error);
    }

    pub fn selected_save(&self) -> Option<&SaveMetadata> {
        match &self.saves_state {
            SaveListState::Loaded(rows) => rows.get(self.selected_save_index),
            _ => None,
        }
    }

    pub fn save_selection_next(&mut self) {
        if let SaveListState::Loaded(rows) = &self.saves_state {
            if !rows.is_empty() {
                self.selected_save_index = (self.selected_save_index + 1).min(rows.len() - 1);
            }
        }
    }

    pub fn save_selection_previous(&mut self) {
        self.selected_save_index = self.selected_save_index.saturating_sub(1);
    }

    pub fn open_save_upload_picker(&mut self) {
        self.save_upload_picker = Some(PathPicker::new(PathPickerMode::File, ""));
        self.message = Some("Choose a save file to upload".to_string());
        self.message_clear_at = None;
    }

    /// True when the extras picker can offer at least one row.
    pub fn has_any_extras(&self) -> bool {
        !self.other_files.is_empty()
            || !collect_update_dlc_files(&self.rom).is_empty()
            || self
                .rom
                .url_cover
                .as_deref()
                .map(str::trim)
                .filter(|s| !s.is_empty())
                .is_some()
            || self
                .rom
                .url_manual
                .as_deref()
                .map(str::trim)
                .filter(|s| !s.is_empty())
                .is_some()
    }

    pub(crate) fn cover_pipeline_label(&self) -> &'static str {
        if self.cover_protocol.is_some() {
            "Advanced terminal protocol"
        } else {
            "Halfblocks fallback mode"
        }
    }

    /// Find the most recent download job for this ROM (if any).
    /// Returns downloading jobs always, or completed/errored jobs if not yet acknowledged.
    pub(crate) fn active_download(&self) -> Option<DownloadJob> {
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
                                        | DownloadStatus::Cancelled
                                        | DownloadStatus::FinalizeFailed(_)
                                        | DownloadStatus::Error(_)
                                )))
                })
                .cloned()
        })
    }
}
