# Tabbed game detail — design

**Date:** 2026-07-03
**Status:** Approved

## Problem

The game detail metadata panel is a single non-scrolling `Paragraph` containing title, platform, summary, file info, DLC list, technical section, saves list, and achievements list. As features accumulate, content overflows the visible area, sections are hard to navigate, and unrelated concerns are jammed together.

## Solution

Split the metadata panel (left side) into three tabs: **Info**, **Saves**, **Achievements**. The cover panel (right side) and footer stay unchanged. Tabs control only the left panel content.

## Layout

```
┌─ Game detail ─────────────────────────────────┬─ Cover ──────┐
│ [1:Info]  [2:Saves]  [3:Achievements]         │              │
│───────────────────────────────────────────────│   (cover     │
│                                               │    image)    │
│   (active tab content here)                   │              │
│                                               │              │
├───────────────────────────────────────────────┴──────────────┤
│  Esc Back  d Download  e Extras  1/2/3 Tabs                  │
└──────────────────────────────────────────────────────────────┘
```

## Tab contents

### Tab 1: Info (default)

Static, no scrolling. Same content currently rendered before the Saves section:

- Title, Platform
- Overview (download status, cover URL)
- Summary
- File path, size
- Other files / DLC list (up to 10 + overflow)
- Technical section (toggled with `t`)

### Tab 2: Saves

- Full save list with `>` selection marker
- `j`/`k` to navigate saves (same as today)
- `u` to upload, `D` to download (only active on this tab)
- Scrollable — remove the 8-item truncation, window by scroll offset + visible height

### Tab 3: Achievements

- Summary header: `earned/total (pct%)`
- Full achievement list with `[✓]`/`[ ]` markers
- `j`/`k` to scroll
- Remove 8-item truncation, window by scroll offset + visible height

## Navigation

- `1`, `2`, `3` jump directly to each tab
- `j`/`k` scroll within the active tab (Saves or Achievements)
- Tab-specific keys (`u`, `D`) only work on the relevant tab
- Global keys (`Esc`, `d`, `e`, `m`, `t`, `o`, `q`, `Shift+U`, `Ctrl+←/→`) work from any tab

## Footer hints

Context-sensitive per active tab:

- **Info:** `e Extras` `m Match` `t Technical` `Shift+U Unmatch` `Ctrl+←/→ Resize`
- **Saves:** `u Upload` `D Download` `j/k Navigate`
- **Achievements:** `j/k Scroll`
- **Common (all tabs):** `1/2/3 Tabs` `Esc Back` `Enter Download`

## State changes

- `GameDetailScreen` gains `active_tab: DetailTab` enum and `achievement_scroll_offset: usize`
- Existing `selected_save_index` already handles save scroll
- `render_metadata_panel` replaced by `render_tab_bar` + `render_active_tab` dispatch
- `save_lines()` and `achievement_lines()` drop their hardcoded `.take(8)` truncation — the tab renderer handles windowing via `area.height` and scroll offset

## Precedent

Mirrors the existing `SettingsScreen` tab pattern: `SettingsTab` enum with `ALL`/`index()`/`title()`, ratatui `Tabs` widget, `selected_tab` field on the screen struct.

## Out of scope

- Horizontal tab cycling with Tab/Shift+Tab (number keys are sufficient)
- Tab for extras/DLC (stays on Info tab)
- Any new data fetching or API changes
