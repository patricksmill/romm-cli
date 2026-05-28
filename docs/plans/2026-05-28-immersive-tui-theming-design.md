# Immersive TUI Theming — Design

**Date:** 2026-05-28  
**Status:** Implemented

## Problem

Built-in themes only changed accent-colored highlights. Background, borders, body text, and selection blocks stayed at terminal defaults, so switching presets (e.g. Rosé Pine → Dracula) felt nearly identical.

## Goal

Full immersive theming: each preset paints the entire UI with its palette — background, surfaces, borders, text, and block-style selections.

## Decision

Expand `RommStyles` to expose the full ratatui-themekit contract and apply it consistently across all TUI render paths. No new config fields; immersive mode activates when a theme defines a real `background()` color (`no-color` / `NO_COLOR` keeps terminal-native behavior).

## Architecture

```
App.theme → RommStyles
  ├── fill_background()     — full frame at render start
  ├── panel_block()         — bordered panels with surface fill
  ├── header_block()        — info strips
  ├── text / muted / label  — semantic foreground
  ├── border / border_accent
  ├── selection()           — accent fg + stripe bg + bold
  └── row()                   — zebra stripes in tables
```

## Semantic mapping (new / changed)

| Role | Source | Use |
|------|--------|-----|
| Canvas | `background()` | Full-screen fill |
| Panels | `surface()` + `text()` | All `Block` interiors |
| Borders | `border()` | Default box borders |
| Focus border | `accent()` | Path picker list focus |
| Selection | `accent()` + `stripe()` bg | Lists, tabs, tables |
| Zebra rows | `stripe()` | Alternating table rows |
| Body text | `text()` | Default list/paragraph content |

## Scope

All TUI screens and overlays: main menu, library, search, game detail, downloads, settings, setup wizard, path picker, keyboard help, splash, global popups.

## Testing

- `dracula_has_immersive_background_and_selection_contrast` unit test
- Full `cargo test --features tui --lib` suite (203 tests)
- Manual: cycle presets in Settings → Appearance on each major screen
