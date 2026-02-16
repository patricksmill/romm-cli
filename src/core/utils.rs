use crate::types::Rom;

/// One game entry for list display: same `name` (base + updates/DLC) shown once.
#[derive(Debug, Clone)]
pub struct RomGroup {
    pub name: String,
    pub primary: Rom,
    pub others: Vec<Rom>,
}

/// Group ROMs by game name; primary is the "base" file (prefer over
/// `"[Update]"` / `"[DLC]"` tags in `fs_name` when present).
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

/// Human-readable file size.
pub fn format_size(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;
    if bytes >= GB {
        format!("{:.2} GB", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.2} MB", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.2} KB", bytes as f64 / KB as f64)
    } else {
        format!("{} B", bytes)
    }
}

/// Make a filename safe for the local filesystem.
pub fn sanitize_filename(name: &str) -> String {
    name.chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || c == '.' || c == '-' || c == '_' || c == ' ' {
                c
            } else {
                '_'
            }
        })
        .collect()
}
