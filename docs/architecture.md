# Architecture overview

This document gives a deeper view of how the workspace is structured. Read it alongside generated Rustdoc (`cargo doc --workspace --open`) and the per-crate guides: [api.md](api.md), [cli.md](cli.md), [tui.md](tui.md).

## High-level layers

The project is a **Cargo workspace** with three members:

| Crate | Role |
|-------|------|
| `romm-api` | HTTP client (`RommClient`), endpoints, types, `core/`, config, errors — shared by all frontends |
| `romm-cli` | CLI commands and `romm-cli` binary; re-exports `romm_api` for backward-compatible library use |
| `romm-tui` | TUI screens and `romm-tui` binary |

An Android client (Kotlin/Compose + UniFFI) is developed in [**romm-rust-android**](https://github.com/patricksmill/romm-rust-android) on top of published `romm-api`. The workspace split that enabled this is documented in [workspace-split ADR](plans/2026-06-06-workspace-split-adr.md).

Configuration is layered per field: built-in defaults → `config.json` → environment variables → OS keyring (secret sentinels) → command-specific CLI runtime overrides. See [api.md — configuration precedence](api.md#configuration-precedence) and [`romm-api/src/config.rs`](../romm-api/src/config.rs). Secrets may be stored in the OS keyring via `keyring::Entry` with a `<stored-in-keyring>` sentinel in JSON only after successful read-back. `Commands::Init` is handled in `romm-cli/src/main.rs` *before* `load_config` so `init` can run when no configuration exists yet.

From bottom to top:

- **Types & endpoints** (`romm-api`)
  - `types.rs` — data models used throughout the app.
  - `endpoints/*` — `Endpoint` trait implementations (method, path, query, body) per ROMM API route. Grouped by area: `platforms`, `roms`, `collections`, `client_tokens`, `device`, `saves`, `sync`, `system`, `tasks`.
- **Core services** (`romm-api`)
  - `config` — `Config` / `AuthConfig`, layered merge, keyring.
  - `client` — `RommClient` wraps `reqwest::Client` and performs typed HTTP calls.
  - `core` — `RomCache`, `DownloadManager`, `library_scan`, resolve helpers.
- **Frontends**
  - **CLI** (`romm-cli/src/commands/*`, `frontend/cli.rs`) — one-shot commands; `frontend::cli` routes parsed arguments to handlers.
  - **TUI** (`romm-tui/src/tui/*`) — event loop and screens.

There are no TUI/CLI dependencies inside `romm-api` core services, which keeps new frontends straightforward.

### CLI structure

- `commands::mod` — top-level `Cli` and `Commands` enum plus `OutputFormat`.
- `commands::{platforms,roms,api,auth,download,scan,sync,cache,init,update}` — parse args, call services, print results. Upload-triggered and standalone scans share `romm-api::core::library_scan`; CLI presentation wrappers live in `commands::library_scan`.
- `commands::print` — tabular text output helpers.
- `cli_presentation` — color, progress, JSON vs text output.

### TUI event loop

The TUI follows an **Event → Action → update → render** pipeline (see [Gap 5 in rust-guidelines.md](rust-guidelines.md#gap-5-tui-event--action-separation)):

```text
poll_frame_events()     # drain_background_events + crossterm input
  → map_event()       # AppEvent → Action
  → App::update()     # single state-mutation entry point
  → render()          # ratatui draw (screens are render-only)
```

Key files under `romm-tui/src/tui/`:

- `app/event.rs` — `AppEvent`, `Action`, global key mapping
- `app/update.rs` — applies actions (navigation, spawns, background completions)
- `app/run.rs` — thin loop using `runtime.rs` (`TuiSession`)
- `app/handlers/screen_keys.rs` — per-screen key → action dispatch
- `screens/setup_wizard/event.rs` — first-run setup wizard

See [tui.md](tui.md) for screen list and theming.

## Data flow

```text
Config + env + OS Keyring
    ↓
RommClient (HTTP + auth)
    ↓
Endpoint implementations
    ↓
typed responses (types.rs)
```

The TUI and CLI both operate on the same `RommClient` and model types from `romm-api`.

## Why an enum-based state machine?

The TUI uses:

- `AppScreen` — enum variants for each high-level screen (`MainMenu`, `LibraryBrowse`, `Search`, `Settings`, `GameDetail`, `Download`, `SetupWizard`).
- `App` — owns shared services (`RommClient`, `RomCache`, `DownloadManager`) and the current `AppScreen`, plus `save_sync_compat`, `server_version`, `startup_splash`, and `deferred_load_roms`.

Each key press dispatches to handlers that match on `self.screen`, mutate it, and transition variants. The compiler forces exhaustive `match` handling; ownership stays explicit when moving screens in and out of the enum.
