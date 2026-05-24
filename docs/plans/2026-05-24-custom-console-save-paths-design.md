# Custom Console Save Paths — Design

**Date:** 2026-05-24  
**Status:** Approved  
**Mirrors:** [Custom Console Paths (ROMs)](2026-05-23-custom-console-paths-design.md)

## Problem

Save downloads use a single global `save_dir` with per-game subfolders (`{save_dir}/{game}/`). Users with multi-drive layouts (Switch saves on `D:`, NES on `E:`) cannot map consoles to custom absolute paths the way ROM downloads already support via `roms_layout.platform_dirs`.

## Goal

One mental model aligned with ROMs: **every console defaults to `{save_base}/{platform-slug}/` unless it has a custom path.** Game saves land in `{console_save_dir}/{game-name}/`. Map only consoles that live on other drives.

User-facing terminology: **custom** (same as ROMs).

## Decision

**Overrides only.** Extend `save_sync.platform_dirs` with the same semantics as `roms_layout.platform_dirs`.

## Config & data model

### Persisted shape

```json
{
  "download_dir": "C:\\Games\\romm-cli",
  "save_sync": {
    "save_dir": "C:\\Games\\romm-cli\\saves",
    "device_id": "...",
    "platform_dirs": {
      "7": "D:\\Saves\\Switch",
      "3": "E:\\Saves\\NES"
    }
  }
}
```

### Path resolution

1. Base save root: `save_sync.save_dir` or `{download_dir}/saves`.
2. Per-console: if `platform_dirs[platform_id]` is non-empty → use that absolute path (validate writable).
3. Else → `{save_base}/{platform-slug}/`.
4. Per-game download: `{console_save_dir}/{sanitized-game-name}/`.

### Breaking change

Previously unmapped consoles used flat `{save_base}/{game}/`. They now use `{save_base}/{platform-slug}/{game}/` to match the ROM layout model. Document in CHANGELOG; no auto-migration.

## Settings → Saves tab

**Rows:**

| Row | Label | Action |
|-----|-------|--------|
| Save Dir | `Save Dir: {path}` | Enter → directory picker |
| Save console paths | `Save console paths: {N} custom · Enter to edit` | Enter → console picker |

**Console picker:** Same UX as ROMs tab (shared picker with `ConsolePathKind::Saves`).

- Load platforms via `ListPlatforms` API
- Enter → absolute directory path picker
- Delete → remove override for selected console
- Esc → back

## Runtime

- TUI save download (`D` in game detail): uses `resolve_game_save_dir`
- CLI `sync run` manifest paths: unchanged (explicit paths in manifest)
- TUI Sync Now (push-pull): server-side; no local path change

## Testing

- Resolver: override present / absent / cleared / empty string ignored
- Config: JSON round-trip with `save_sync.platform_dirs`
- Settings: Save console paths row visible; clear custom action
- Save download wiring uses resolver

## Out of scope

- Setup wizard / `init` prompts for save paths
- Relative path overrides
- “Follow ROM directory” inheritance
- CLI manifest path generation from config
