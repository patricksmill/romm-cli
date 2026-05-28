//! Semantic TUI styling backed by ratatui-themekit presets.

use ratatui::style::{Color, Modifier, Style};
use ratatui_themekit::{available_theme_ids, resolve_theme, Theme};

use crate::config::{default_theme_id, DEFAULT_THEME_ID};

/// Resolve a theme ID, falling back to [`DEFAULT_THEME_ID`] for unknown values.
pub fn resolve_theme_or_default(id: &str) -> Box<dyn Theme> {
    if ratatui_themekit::no_color_active() {
        return resolve_theme(id);
    }
    let known = id == "no-color" || available_theme_ids().iter().any(|known_id| *known_id == id);
    if !known {
        tracing::warn!(theme = id, "unknown theme ID, using {DEFAULT_THEME_ID}");
        return resolve_theme(DEFAULT_THEME_ID);
    }
    resolve_theme(id)
}

/// Human-readable name for a theme ID (uses fallback resolution).
pub fn theme_display_name(id: &str) -> String {
    resolve_theme_or_default(id).name().to_string()
}

/// Cycle to the next built-in theme ID (wraps).
pub fn next_theme_id(current: &str) -> String {
    let ids: Vec<&str> = available_theme_ids();
    if ids.is_empty() {
        return default_theme_id();
    }
    let idx = ids.iter().position(|id| *id == current).unwrap_or(0);
    ids[(idx + 1) % ids.len()].to_string()
}

/// Cycle to the previous built-in theme ID (wraps).
pub fn prev_theme_id(current: &str) -> String {
    let ids: Vec<&str> = available_theme_ids();
    if ids.is_empty() {
        return default_theme_id();
    }
    let idx = ids.iter().position(|id| *id == current).unwrap_or(0);
    let len = ids.len();
    ids[(idx + len - 1) % len].to_string()
}

/// App-level semantic styles mapped onto a ratatui-themekit theme.
pub struct RommStyles<'a> {
    theme: &'a dyn Theme,
}

impl<'a> RommStyles<'a> {
    pub fn new(theme: &'a dyn Theme) -> Self {
        Self { theme }
    }

    pub fn theme(&self) -> &dyn Theme {
        self.theme
    }

    pub fn selection(&self) -> Style {
        Style::default()
            .fg(self.theme.accent())
            .add_modifier(Modifier::BOLD)
    }

    pub fn label(&self) -> Style {
        Style::default().fg(self.theme.info())
    }

    pub fn success(&self) -> Style {
        Style::default().fg(self.theme.success())
    }

    pub fn error(&self) -> Style {
        Style::default().fg(self.theme.error())
    }

    pub fn warning(&self) -> Style {
        Style::default().fg(self.theme.warning())
    }

    pub fn muted(&self) -> Style {
        Style::default().fg(self.theme.text_dim())
    }

    pub fn primary_text(&self) -> Style {
        Style::default().fg(self.theme.text_bright())
    }

    pub fn border_focus(&self) -> Style {
        Style::default().fg(self.theme.accent())
    }

    pub fn footer_hint(&self) -> Style {
        Style::default().fg(self.theme.text_dim())
    }

    pub fn color_success(&self) -> Color {
        self.theme.success()
    }

    pub fn color_error(&self) -> Color {
        self.theme.error()
    }

    pub fn color_warning(&self) -> Color {
        self.theme.warning()
    }

    pub fn color_info(&self) -> Color {
        self.theme.info()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resolve_unknown_falls_back_to_terminal() {
        std::env::remove_var("NO_COLOR");
        let theme = resolve_theme_or_default("not-a-theme");
        assert_eq!(theme.id(), "terminal");
    }
}
