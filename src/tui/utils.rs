use std::process::Command;

/// Truncate a string to `max` chars, appending "…" if trimmed.
pub fn truncate(s: &str, max: usize) -> String {
    crate::core::utils::truncate(s, max)
}

// ---------------------------------------------------------------------------
// OS helpers
// ---------------------------------------------------------------------------

/// Open a URL in the system default browser.
pub fn open_in_browser(url: &str) -> std::io::Result<std::process::Child> {
    let mut cmd = if cfg!(target_os = "windows") {
        let mut c = Command::new("cmd");
        c.args(["/C", "start", "", url]);
        c
    } else if cfg!(target_os = "macos") {
        Command::new("open")
    } else {
        Command::new("xdg-open")
    };

    if !cfg!(target_os = "windows") {
        cmd.arg(url);
    }
    cmd.spawn()
}
