use crate::core::interrupt::InterruptContext;
use crate::update::{self, ApplyUpdateOptions, ApplyUpdateOutcome};
use anyhow::Result;

/// Handle the `update` command.
pub async fn handle(interrupt: Option<InterruptContext>) -> Result<()> {
    let options = ApplyUpdateOptions {
        show_progress: true,
        show_output: true,
        no_confirm: true,
        target_version_tag: None,
    };
    match update::apply_update(interrupt, options).await? {
        ApplyUpdateOutcome::Updated(version) => {
            println!("Updated successfully to `{version}`.");
            println!("Restart romm-cli to use the new version.");
        }
        ApplyUpdateOutcome::UpToDate(version) => {
            println!("Already up to date (`{version}`).");
        }
    }
    Ok(())
}
