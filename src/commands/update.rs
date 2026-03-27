use anyhow::Result;
use self_update::cargo_crate_version;

/// Handle the `update` command.
pub fn handle() -> Result<()> {
    let status = self_update::backends::github::Update::configure()
        .repo_owner("patricksmill")
        .repo_name("romm-cli")
        .bin_name("romm-cli")
        .show_download_progress(true)
        .current_version(cargo_crate_version!())
        .build()?
        .update()?;

    println!("Update status: `{}`!", status.version());
    Ok(())
}
