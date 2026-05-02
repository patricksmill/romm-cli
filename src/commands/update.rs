use crate::core::interrupt::InterruptContext;
use crate::update;
use anyhow::Result;

/// Handle the `update` command.
pub async fn handle(interrupt: Option<InterruptContext>) -> Result<()> {
    let version = update::apply_update(interrupt, true).await?;
    println!("Update status: `{}`!", version);
    Ok(())
}
