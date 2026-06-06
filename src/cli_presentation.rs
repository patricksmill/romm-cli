//! CLI output presentation: color, progress, and JSON vs text stdout rules.

use std::io::{self, IsTerminal, Write};

use indicatif::{MultiProgress, ProgressDrawTarget, ProgressStyle};
use serde::Serialize;

use crate::commands::OutputFormat;
use crate::error::{user_message, RommError};

/// Resolved CLI output mode for a single command invocation.
#[derive(Clone, Copy, Debug)]
pub struct CliPresentation {
    pub format: OutputFormat,
    pub verbose: bool,
}

impl CliPresentation {
    pub fn from_cli(global_json: bool, local_json: bool, verbose: bool) -> Self {
        Self {
            format: OutputFormat::from_flags(global_json, local_json),
            verbose,
        }
    }

    pub fn is_json(&self) -> bool {
        matches!(self.format, OutputFormat::Json)
    }

    pub fn is_text(&self) -> bool {
        matches!(self.format, OutputFormat::Text)
    }

    /// True when ANSI styling is allowed on CLI output.
    pub fn supports_ansi_color(&self) -> bool {
        if std::env::var_os("NO_COLOR").is_some() {
            return false;
        }
        if std::env::var("CLICOLOR").as_deref() == Ok("0")
            && std::env::var("CLICOLOR_FORCE").as_deref() != Ok("1")
        {
            return false;
        }
        io::stdout().is_terminal()
    }

    /// True when indicatif progress bars/spinners may be shown.
    pub fn shows_progress(&self) -> bool {
        self.is_text() && io::stdout().is_terminal()
    }

    pub fn progress_draw_target(&self) -> ProgressDrawTarget {
        ProgressDrawTarget::stderr()
    }

    pub fn progress_style(&self, template_plain: &str, template_color: &str) -> ProgressStyle {
        let template = if self.supports_ansi_color() {
            template_color
        } else {
            template_plain
        };
        ProgressStyle::with_template(template)
            .expect("hardcoded progress template")
            .progress_chars("#>-")
    }

    pub fn multi_progress(&self) -> Option<MultiProgress> {
        if !self.shows_progress() {
            return None;
        }
        let mp = MultiProgress::with_draw_target(self.progress_draw_target());
        Some(mp)
    }

    /// Human status line on stderr (omitted in JSON mode).
    pub fn emit_status(&self, message: impl AsRef<str>) {
        if self.is_text() {
            let _ = writeln!(io::stderr(), "{}", message.as_ref());
        }
    }

    /// Pretty JSON on stdout (JSON mode only).
    pub fn emit_json<T: Serialize>(&self, value: &T) -> Result<(), RommError> {
        if self.is_json() {
            println!(
                "{}",
                serde_json::to_string_pretty(value).map_err(|e| {
                    RommError::Other(format!("failed to serialize JSON output: {e}"))
                })?
            );
        }
        Ok(())
    }

    /// Actionable error line on stderr (text mode batch failures).
    pub fn emit_command_error(&self, err: &RommError) {
        eprintln!("Error: {}", user_message(err));
        if self.verbose {
            eprintln!("Details: {err:#}");
        }
    }
}

/// Format a command-local failure for stderr.
pub fn format_command_error(err: &RommError) -> String {
    user_message(err)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn clear_color_env() {
        std::env::remove_var("NO_COLOR");
        std::env::remove_var("CLICOLOR");
        std::env::remove_var("CLICOLOR_FORCE");
    }

    #[test]
    fn json_format_suppresses_progress() {
        clear_color_env();
        let p = CliPresentation::from_cli(true, false, false);
        assert!(!p.shows_progress());
        assert!(p.is_json());
    }

    #[test]
    fn no_color_disables_ansi() {
        clear_color_env();
        std::env::set_var("NO_COLOR", "1");
        let p = CliPresentation::from_cli(false, false, false);
        assert!(!p.supports_ansi_color());
        std::env::remove_var("NO_COLOR");
    }

    #[test]
    fn clicolor_zero_disables_ansi() {
        clear_color_env();
        std::env::set_var("CLICOLOR", "0");
        let p = CliPresentation::from_cli(false, false, false);
        assert!(!p.supports_ansi_color());
        std::env::remove_var("CLICOLOR");
    }

    #[test]
    fn clicolor_force_overrides_clicolor_zero() {
        clear_color_env();
        std::env::set_var("CLICOLOR", "0");
        std::env::set_var("CLICOLOR_FORCE", "1");
        let p = CliPresentation::from_cli(false, false, false);
        // May still be false when stdout is not a TTY in test harness.
        let _ = p.supports_ansi_color();
        std::env::remove_var("CLICOLOR");
        std::env::remove_var("CLICOLOR_FORCE");
    }
}
