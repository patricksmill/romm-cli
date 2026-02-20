use crate::core::utils::truncate;
use crate::types::{Platform, RomList};

/// Print a simple table of platforms with aligned columns.
pub fn print_platforms_table(platforms: &[Platform]) {
    if platforms.is_empty() {
        println!("No platforms returned.");
        return;
    }

    let id_w = platforms
        .iter()
        .map(|p| p.id.to_string().len())
        .max()
        .unwrap_or(2)
        .max(2);
    let slug_w = platforms
        .iter()
        .map(|p| p.slug.len())
        .max()
        .unwrap_or(4)
        .max(4);
    let name_w = platforms
        .iter()
        .map(|p| {
            p.display_name
                .as_ref()
                .filter(|s| !s.is_empty())
                .unwrap_or(&p.name)
                .len()
        })
        .max()
        .unwrap_or(4)
        .max(4)
        .min(40); // keep names from getting too wide

    println!(
        "{:>id_w$}  {:<slug_w$}  {:<name_w$}  {:>6}  {:>8}",
        "ID",
        "SLUG",
        "NAME",
        "ROMS",
        "FIRMWARE",
        id_w = id_w,
        slug_w = slug_w,
        name_w = name_w
    );

    for p in platforms {
        let display_name = p
            .display_name
            .as_ref()
            .filter(|s| !s.is_empty())
            .unwrap_or(&p.name);
        let name = truncate(display_name, name_w);
        println!(
            "{:>id_w$}  {:<slug_w$}  {:<name_w$}  {:>6}  {:>8}",
            p.id,
            p.slug,
            name,
            p.rom_count,
            p.firmware.len(),
            id_w = id_w,
            slug_w = slug_w,
            name_w = name_w
        );
    }
}

/// Print a simple table of ROMs for the `roms` command.
pub fn print_roms_table(results: &RomList) {
    if results.items.is_empty() {
        println!("No ROMs returned.");
        return;
    }

    let id_w = results
        .items
        .iter()
        .map(|r| r.id.to_string().len())
        .max()
        .unwrap_or(2)
        .max(2);
    let pid_w = results
        .items
        .iter()
        .map(|r| r.platform_id.to_string().len())
        .max()
        .unwrap_or(2)
        .max(2);
    let name_w = results
        .items
        .iter()
        .map(|r| r.name.len())
        .max()
        .unwrap_or(4)
        .max(4)
        .min(60);

    println!(
        "{:>id_w$}  {:>pid_w$}  {:<name_w$}",
        "ID",
        "PLAT",
        "NAME",
        id_w = id_w,
        pid_w = pid_w,
        name_w = name_w
    );

    for r in &results.items {
        let name = truncate(&r.name, name_w);
        println!(
            "{:>id_w$}  {:>pid_w$}  {:<name_w$}",
            r.id,
            r.platform_id,
            name,
            id_w = id_w,
            pid_w = pid_w,
            name_w = name_w
        );
    }
}
