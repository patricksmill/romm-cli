use ratatui::text::Line;

use crate::tui::utils::truncate;
use romm_api::core::utils::format_size;

use super::types::SaveListState;

pub fn save_lines(state: &SaveListState, selected_index: usize) -> Vec<Line<'static>> {
    match state {
        SaveListState::Idle => vec![Line::from("  Loading soon...")],
        SaveListState::Loading => vec![Line::from("  Loading remote saves...")],
        SaveListState::Failed(e) => vec![Line::from(format!("  Error: {}", truncate(e, 90)))],
        SaveListState::Loaded(rows) if rows.is_empty() => vec![Line::from("  No remote saves")],
        SaveListState::Loaded(rows) => rows
            .iter()
            .enumerate()
            .take(8)
            .map(|(i, save)| {
                let marker = if i == selected_index { "> " } else { "  " };
                let mut parts = vec![save.file_name.clone()];
                if let Some(emulator) = save.emulator.as_deref().filter(|s| !s.is_empty()) {
                    parts.push(format!("emu={emulator}"));
                }
                if let Some(slot) = save.slot.as_deref().filter(|s| !s.is_empty()) {
                    parts.push(format!("slot={slot}"));
                }
                if let Some(size) = save.size_bytes {
                    parts.push(format_size(size));
                }
                if let Some(updated) = save.updated_at.as_deref().filter(|s| !s.is_empty()) {
                    parts.push(updated.to_string());
                }
                if let Some(hash) = save.hash.as_deref().filter(|s| !s.is_empty()) {
                    parts.push(format!("hash={}", truncate(hash, 12)));
                }
                if let Some(device) = save.device_name.as_deref().filter(|s| !s.is_empty()) {
                    parts.push(format!("device={device}"));
                } else if let Some(device) = save.device_id.as_deref().filter(|s| !s.is_empty()) {
                    parts.push(format!("device={device}"));
                }
                Line::from(format!("{marker}{}", truncate(&parts.join(" | "), 120)))
            })
            .collect(),
    }
}
