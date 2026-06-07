//! Regenerate shell completion scripts under `completions/`.
//!
//! Invoked by `build.rs` (with `ROMM_SKIP_COMPLETION_GEN=1`) and manually via:
//! `cargo run --bin romm-complete-gen`

use clap::CommandFactory;
use clap_complete::aot::{generate_to, Shell};
use romm_cli::commands::Cli;
use std::fs;
use std::io;
use std::path::Path;

fn main() -> io::Result<()> {
    let outdir = Path::new("completions");
    fs::create_dir_all(outdir)?;

    let mut cmd = Cli::command();
    for shell in [
        Shell::Bash,
        Shell::Zsh,
        Shell::Fish,
        Shell::PowerShell,
        Shell::Elvish,
    ] {
        generate_to(shell, &mut cmd, "romm-cli", outdir)?;
    }

    eprintln!("Generated completion scripts in {}", outdir.display());
    Ok(())
}
