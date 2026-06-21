//! Launches the ROMM TUI only (no `tui` subcommand). Same config as `romm-cli`.

use anyhow::Result;

#[tokio::main]
async fn main() -> Result<()> {
    let verbose = std::env::var("ROMM_VERBOSE")
        .map(|v| v == "1" || v.eq_ignore_ascii_case("true"))
        .unwrap_or(false);

    romm_tui::tui::run_interactive(verbose, false).await
}
