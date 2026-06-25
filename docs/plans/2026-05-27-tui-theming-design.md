# TUI Theming — Design

**Date:** 2026-05-27  
**Status:** Implemented

## Problem

The TUI hardcodes ANSI colors (`Color::Yellow`, `Color::Cyan`, etc.) across ~15 files (~100 usages). There is no central palette, no user choice, and visual consistency depends on each screen author picking the same colors.

## Goal

Let users pick from **built-in preset themes** only (no custom color editor). Theme choice persists in `config.json`, applies across all TUI screens, and can be changed live from Settings.

## Decision

Use **[ratatui-themekit](https://docs.rs/ratatui-themekit)** (v0.6.1) for preset palettes and semantic color slots. Wrap it in a thin `RommStyles` module that maps app roles to theme semantics.

**Default theme:** `terminal` (Terminal Native) — closest to today's ANSI-based look so existing users are not surprised on upgrade.

## Architecture

```
config.json  →  theme: "dracula"
       ↓
load_config / env ROMM_THEME (optional override)
       ↓
App.theme: Box<dyn Theme>   (resolve_theme at startup / on settings change)
       ↓
RommStyles { theme: &dyn Theme }   app-role helpers → Theme trait slots
       ↓
All render() methods receive &RommStyles
```

### Semantic mapping

| App role | ratatui-themekit slot | Typical use today |
|----------|----------------------|-------------------|
| `selection()` | `accent` + bold | Yellow highlights, list focus |
| `label()` | `info` | Cyan field labels, hints |
| `success()` | `success` | Green status, "Done", saved |
| `error()` | `error` | Red errors, confirmations |
| `warning()` | `warning` | Yellow warnings, skipped |
| `muted()` | `text_dim` | DarkGray / Gray secondary text |
| `primary_text()` | `text_bright` | White primary content |
| `border_focus()` | `accent` | Focused widget borders |
| `footer_hint()` | `text_dim` or `info` | Help footers |

### Available presets

All themes from `ratatui_themekit::available_theme_ids()`:

`catppuccin`, `dracula`, `gruvbox`, `nord`, `one-dark`, `rose-pine`, `solarized`, `tailwind`, `tokyo-night`, `terminal`, `no-color`

When `NO_COLOR` is set in the environment, `resolve_theme` returns the no-color preset automatically (themekit built-in behavior).

## Config

### Persisted shape

```json
{
  "theme": "terminal"
}
```

- New optional field on `Config` with serde default `"terminal"`.
- Optional env override: `ROMM_THEME=dracula` (same precedence pattern as other config: env wins over JSON).

### Invalid values

Unknown theme ID → log a warning, fall back to `terminal`.

## Settings → Appearance tab

New **Appearance** tab (between Extras and Auth/Maint, or after Connection — implementer picks tab order that fits existing Tab widget width).

| Row | Label | Action |
|-----|-------|--------|
| Theme | `Theme: {display name}` | ← / → cycle through `available_theme_ids()` |

- Change applies **immediately** in memory (live preview across the TUI).
- **S** saves `theme` to `config.json` with other settings.
- `SettingsTab::COUNT` and tab indices update; tests updated accordingly.

## Migration scope

Replace hardcoded `Color::` in TUI render/handler code with `styles.*()` calls. Files (~15):

- `src/tui/app/render.rs`
- `src/tui/path_picker.rs`
- `src/tui/screens/*/render.rs` (and related)
- `src/tui/app/handlers/settings.rs`, `setup_wizard.rs`
- `src/tui/app/background/tasks.rs`
- `src/tui/screens/settings/console.rs`, `state.rs`

Screens that currently use default terminal colors (e.g. main menu) gain themed list highlights once migrated.

**Out of scope:** CLI output (`indicatif`, `dialoguer`), non-TUI code paths.

## Dependencies

```toml
ratatui-themekit = { version = "0.6", optional = true, features = ["serde"] }
```

Add to the existing `tui` feature group. Requires Rust 1.86+ (ratatui-themekit MSRV; CI uses stable).

## Testing

- Config: JSON round-trip for `theme` field; default when absent; env override
- `RommStyles` / theme resolution: unknown ID falls back
- Settings: Appearance tab visible; theme row cycles; save persists
- `cargo test` full suite; manual smoke: switch themes on each major screen

## Out of scope

- Custom per-slot color overrides (TOML/JSON editor)
- Light-theme-only presets beyond what themekit ships
- Syntax highlighting or image widget theming
- Theme hotkey outside Settings
