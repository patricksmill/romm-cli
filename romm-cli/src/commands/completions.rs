//! Runtime shell completion generation via `clap_complete`.

use clap::{Args, CommandFactory};
use clap_complete::aot::{generate, Shell};

use crate::commands::Cli;
use romm_api::error::RommError;

#[derive(Args, Debug)]
pub struct CompletionsCommand {
    /// Shell to generate completions for.
    #[arg(value_enum)]
    pub shell: Shell,
}

pub fn handle(cmd: CompletionsCommand) -> Result<(), RommError> {
    let mut cli = Cli::command();
    generate(cmd.shell, &mut cli, "romm-cli", &mut std::io::stdout());
    Ok(())
}
