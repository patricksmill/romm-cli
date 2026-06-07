use crate::cli_presentation::CliPresentation;
use crate::core::interrupt::InterruptContext;
use crate::update::{self, ApplyUpdateOptions, ApplyUpdateOutcome};
use anyhow::Result;
use serde::Serialize;

#[derive(Serialize)]
struct UpdateJsonOutcome {
    status: &'static str,
    version: String,
}

/// Handle the `update` command.
pub async fn handle(
    presentation: CliPresentation,
    interrupt: Option<InterruptContext>,
) -> Result<()> {
    let options = ApplyUpdateOptions {
        show_progress: presentation.shows_progress(),
        show_output: presentation.is_text(),
        no_confirm: true,
        target_version_tag: None,
    };
    match update::apply_update(interrupt, options).await? {
        ApplyUpdateOutcome::Updated(version) => {
            if presentation.is_json() {
                let out = UpdateJsonOutcome {
                    status: "updated",
                    version: version.clone(),
                };
                println!("{}", serde_json::to_string_pretty(&out)?);
            } else {
                println!("Updated successfully to `{version}`.");
                println!("Restart romm-cli to use the new version.");
            }
        }
        ApplyUpdateOutcome::UpToDate(version) => {
            if presentation.is_json() {
                let out = UpdateJsonOutcome {
                    status: "up_to_date",
                    version: version.clone(),
                };
                println!("{}", serde_json::to_string_pretty(&out)?);
            } else {
                println!("Already up to date (`{version}`).");
            }
        }
    }
    Ok(())
}
