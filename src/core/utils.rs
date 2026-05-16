use crate::types::{Rom, RomFile, RomFileCategory};

/// One game entry for list display: same `name` on the same platform (base + updates/DLC) shown once.
#[derive(Debug, Clone)]
pub struct RomGroup {
    pub name: String,
    pub primary: Rom,
    pub others: Vec<Rom>,
}

/// Group ROMs by game name and platform; primary is the "base" file (prefer over
/// `"[Update]"` / `"[DLC]"` tags in `fs_name` when present).
pub fn group_roms_by_name(items: &[Rom]) -> Vec<RomGroup> {
    use std::collections::HashMap;
    let mut by_group: HashMap<(String, u64), Vec<Rom>> = HashMap::new();
    for rom in items {
        by_group
            .entry((rom.name.clone(), rom.platform_id))
            .or_default()
            .push(rom.clone());
    }
    let mut groups = Vec::with_capacity(by_group.len());
    for ((name, _platform_id), mut roms) in by_group {
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
    groups.sort_by(|a, b| {
        a.name
            .cmp(&b.name)
            .then_with(|| a.primary.platform_id.cmp(&b.primary.platform_id))
    });
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

fn category_bucket_index(cat: Option<&RomFileCategory>) -> usize {
    match cat {
        Some(RomFileCategory::Game) => 0,
        Some(RomFileCategory::Update) => 1,
        Some(RomFileCategory::Dlc) => 2,
        Some(RomFileCategory::Patch) => 3,
        Some(RomFileCategory::Hack) => 4,
        Some(RomFileCategory::Mod) => 5,
        Some(RomFileCategory::Translation) => 6,
        Some(RomFileCategory::Demo) => 7,
        Some(RomFileCategory::Prototype) => 8,
        Some(RomFileCategory::Cheat) => 9,
        Some(RomFileCategory::Manual) => 10,
        None => 11,
    }
}

const CATEGORY_BUCKET_LABELS: [&str; 12] = [
    "game",
    "update",
    "dlc",
    "patch",
    "hack",
    "mod",
    "translation",
    "demo",
    "prototype",
    "cheat",
    "manual",
    "other",
];

/// Group a Rom's files by category and return ordered `(label, total_bytes)` pairs.
///
/// Order: game, update, dlc, patch, hack, mod, translation, demo, prototype, cheat, manual, other.
/// Returns an empty vector when `files` is empty. Categories with zero total bytes are omitted.
pub fn size_breakdown_by_category(files: &[RomFile]) -> Vec<(&'static str, u64)> {
    if files.is_empty() {
        return Vec::new();
    }
    let mut sums = [0u64; 12];
    for f in files {
        let i = category_bucket_index(f.category.as_ref());
        sums[i] = sums[i].saturating_add(f.file_size_bytes);
    }
    let mut out = Vec::new();
    for (i, label) in CATEGORY_BUCKET_LABELS.iter().enumerate() {
        if sums[i] > 0 {
            out.push((*label, sums[i]));
        }
    }
    out
}

/// Human-readable total size, optionally with a per-category breakdown from `Rom::files`.
///
/// When `files` is empty, returns [`format_size`] of `total` only. When there is exactly one
/// non-empty bucket and it is `game`, returns the total without a parenthetical (single base file).
pub fn format_size_with_breakdown(total: u64, files: &[RomFile]) -> String {
    let breakdown = size_breakdown_by_category(files);
    if breakdown.is_empty() {
        return format_size(total);
    }
    if breakdown.len() == 1 && breakdown[0].0 == "game" {
        return format_size(total);
    }
    let parts: Vec<String> = breakdown
        .iter()
        .map(|(label, bytes)| format!("{} {}", label, format_size(*bytes)))
        .collect();
    format!("{} ({})", format_size(total), parts.join(" + "))
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

/// Truncate a string to `max` chars, appending "…" if trimmed.
pub fn truncate(s: &str, max: usize) -> String {
    let s = s.trim();
    if s.chars().count() <= max {
        s.to_string()
    } else {
        format!(
            "{}…",
            s.chars().take(max.saturating_sub(1)).collect::<String>()
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::Rom;

    fn rom(id: u64, platform_id: u64, name: &str, fs_name: &str) -> Rom {
        Rom {
            id,
            platform_id,
            platform_slug: None,
            platform_fs_slug: None,
            platform_custom_name: Some("NES".to_string()),
            platform_display_name: Some("NES".to_string()),
            fs_name: fs_name.to_string(),
            fs_name_no_tags: name.to_string(),
            fs_name_no_ext: name.to_string(),
            fs_extension: "zip".to_string(),
            fs_path: format!("/roms/{}.zip", id),
            fs_size_bytes: 1,
            name: name.to_string(),
            slug: None,
            summary: None,
            path_cover_small: None,
            path_cover_large: None,
            url_cover: None,
            has_manual: false,
            path_manual: None,
            url_manual: None,
            is_unidentified: false,
            is_identified: true,
            files: Vec::new(),
        }
    }

    #[test]
    fn group_roms_prefers_base_file_as_primary() {
        let input = vec![
            rom(1, 1, "Game A", "Game A [Update].zip"),
            rom(2, 1, "Game A", "Game A [DLC].zip"),
            rom(3, 1, "Game A", "Game A.zip"),
            rom(4, 1, "Game B", "Game B.zip"),
        ];

        let groups = group_roms_by_name(&input);
        assert_eq!(groups.len(), 2);

        let game_a = groups.iter().find(|g| g.name == "Game A").expect("group");
        assert_eq!(game_a.primary.fs_name, "Game A.zip");
        assert_eq!(game_a.others.len(), 2);
    }

    #[test]
    fn group_roms_separates_same_title_by_platform() {
        let input = vec![
            rom(
                1,
                1,
                "Paper Mario: The Thousand-Year Door",
                "Paper Mario.zip",
            ),
            rom(
                2,
                2,
                "Paper Mario: The Thousand-Year Door",
                "Paper Mario (Switch).zip",
            ),
        ];

        let groups = group_roms_by_name(&input);
        assert_eq!(groups.len(), 2);
        assert_eq!(groups[0].primary.platform_id, 1);
        assert_eq!(groups[1].primary.platform_id, 2);
    }
}
