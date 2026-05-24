# Custom Console Paths — Design

**Date:** 2026-05-23  
**Status:** Approved  
**Replaces:** Auto/Manual `RomsLayoutMode` toggle

## Problem

The current ROM layout model exposes an **Auto / Manual** mode toggle. Manual mode promises per-console absolute paths (for multi-drive collections) but defers all mapping to Settings after setup. Users on scenario A (Switch on `D:`, NES on `E:`) hit:

- A two-step setup trap (pick Manual → configure later)
- Console picker blocked until library is browsed (snapshot dependency)
- Opaque hybrid behavior (Manual + unmapped → silent auto fallback)
- No way to clear a per-console override
- Confusing "Manual" label (sounds like one folder for everything)

## Goal

One mental model: **every console defaults to `{base}/{platform-slug}/` unless it has a custom path.** Map only the consoles that live on other drives.

## Decision

**Approach 2 — overrides only.** Remove the global mode toggle. Keep absolute paths in `platform_dirs` for multi-drive layouts.

User-facing terminology: **custom** (never "manual").

## Config & data model

### Persisted shape

```json
{
  "download_dir": "C:\\Games\\romm-cli",
  "roms_layout": {
    "platform_dirs": {
      "7": "D:\\Roms\\Switch",
      "3": "E:\\Roms\\NES"
    }
  }
}
```

### Path resolution

1. If `platform_dirs[platform_id]` is non-empty → use that absolute path (validate writable).
2. Else → `{download_dir}/{platform-slug}/`.

### Backward compatibility

- Deserialize legacy `"mode": "auto" | "manual"` silently; ignore on read.
- `platform_dirs` always honored regardless of legacy `mode`.
- Next save omits `mode` from written JSON.
- Remove `ROMM_ROMS_LAYOUT` env var (document in CHANGELOG).

### Removed types

- `RomsLayoutMode` enum
- `RomsLayoutConfig.mode` field (read-only compat via serde `default` + skip on serialize)

## Settings → ROMs tab

**Rows:**

| Row | Label | Action |
|-----|-------|--------|
| Roms Dir | `Roms Dir: {path}` | Enter → directory picker |
| Console paths | `Console paths: {N} custom · Enter to edit` | Enter → console picker |

**Console picker:**

- Load platforms via `ListPlatforms` API (not library snapshot)
- Loading / error states (same pattern as device picker)
- List: `{name}  [custom path | base default]  {resolved path}`
- Enter → absolute directory path picker
- Delete → remove override for selected console
- Esc → back

## Setup flows

### TUI wizard

Replace **Step 4 — ROM layout** with **Map consoles on other drives?**

- Default: Not now
- Map now → inline console picker (API-backed); user maps only needed consoles
- Remove Auto/Manual toggle

### `romm-cli init`

- Remove layout `Select` prompt
- After ROMs directory: optional "Map custom paths for consoles on other drives now?" (default No)
- If Yes: fetch platforms, loop select platform → input path until user skips

### Non-interactive init

Unchanged — empty `platform_dirs`; configure later in TUI Settings.

## Runtime

- Downloads, extras, saves: same resolver, no mode branch
- Verbose-only log on unmapped console: `Using base default path: {path}`
- README: replace auto/manual section with Custom console paths

## Testing

- Resolver: override present / absent / cleared
- Config: legacy JSON with `mode` loads; save drops `mode`
- Settings: console paths always visible; clear custom action
- Wizard: optional map-now step replaces layout toggle
- Init: remove layout select test

## Out of scope

- Relative path overrides
- Platform slug aliases
- CLI flags for platform_dirs in non-interactive init
- Extracting console picker into a shared crate module (only if wizard reuse requires it)
