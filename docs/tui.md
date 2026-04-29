# TUI internals (ratatui + crossterm)

This document explains how the terminal UI is wired together.

## Event loop

The heart of the TUI lives in `tui::app::App::run`:

- Enable raw mode
- Enter the alternate screen
- In a loop:
  - drain background tasks via `App::poll_background_tasks` (library metadata refresh completions)
  - draw the current screen via `App::render`
  - poll for key events with a short timeout
  - dispatch keys to the appropriate `handle_*` method
  - perform deferred work (like ROM loading)

## Library startup (metadata snapshot)

Choosing **Library** from the main menu loads a compact on-disk snapshot of platforms and merged collections (if present) so the list can render without waiting for the network. A background task then refetches the same endpoints, updates the UI when complete, and writes a fresh snapshot. Full ROM lists are still loaded on demand (and use the ROM list cache). Snapshot path defaults next to `ROMM_CACHE_PATH`; override with `ROMM_LIBRARY_METADATA_SNAPSHOT_PATH`.

The TUI uses `crossterm` to manage the terminal and `ratatui` to build widgets.

## Screens

Each screen is its own struct under `src/tui/screens/`:

- `MainMenuScreen` – entry menu
- `LibraryBrowseScreen` – consoles/collections + ROM list
- `SearchScreen` – text input + results table
- `GameDetailScreen` – detail view for a single ROM
- `DownloadScreen` – overlay showing downloads
- `SettingsScreen` – current config summary
- `BrowseScreen` / `ExecuteScreen` / `ResultScreen` / `ResultDetailScreen` – API browser flow
- `SetupWizard` – first-run / reconnect configuration flow

The `AppScreen` enum in `tui::app` wraps these screen structs so that `App` only ever has one active screen at a time. During startup, a `StartupSplash` overlay (`screens/connected_splash`) may render before the main menu appears.

## Layout and scrolling

`ratatui::layout::Layout` is used extensively to divide the terminal into smaller `Rect`s:

- A typical pattern is a vertical split into `main area + footer`.
- Complex screens (like the library browser) then split the main area
  horizontally into left/right panes.

Scrolling is done manually:

- For library/search results:
  - a `scroll_offset` index tracks which row is at the top
  - `visible` rows are computed dynamically from the available height
  - helper methods ensure the selected row stays inside the viewport

