use std::process::Command;

use crate::types::Rom;

/// One game entry for list display: same `name` (base + updates/DLC) shown once.
#[derive(Debug, Clone)]
pub struct RomGroup {
    pub name: String,
    pub primary: Rom,
    pub others: Vec<Rom>,
}

/// Group ROMs by game name; primary is the "base" file (prefer over [Update]/[DLC] in fs_name).
pub fn group_roms_by_name(items: &[Rom]) -> Vec<RomGroup> {
    use std::collections::HashMap;
    let mut by_name: HashMap<String, Vec<Rom>> = HashMap::new();
    for rom in items {
        by_name
            .entry(rom.name.clone())
            .or_default()
            .push(rom.clone());
    }
    let mut groups = Vec::with_capacity(by_name.len());
    for (name, mut roms) in by_name {
        roms.sort_by(|a, b| {
            let a_extra = a.fs_name.to_lowercase().contains("[update]")
                || a.fs_name.to_lowercase().contains("[dlc]");
            let b_extra = b.fs_name.to_lowercase().contains("[update]")
                || b.fs_name.to_lowercase().contains("[dlc]");
            match (a_extra, b_extra) {
                (false, true) => std::cmp::Ordering::Less,
                (true, false) => std::cmp::Ordering::Greater,
                _ => std::cmp::Ordering::Equal,
            }
        });
        let primary = roms.remove(0);
        groups.push(RomGroup {
            name,
            primary,
            others: roms,
        });
    }
    groups.sort_by(|a, b| a.name.cmp(&b.name));
    groups
}

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
