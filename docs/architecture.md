# Architecture overview

This document gives a slightly deeper view of how the project is
structured internally. It is meant to be read alongside the generated
Rustdoc (`cargo doc --open`).

## High-level layers

The crate exposes a library root (`src/lib.rs`, `romm_cli`) alongside the
`romm-cli` binary so integration tests and helper binaries can reuse the same
modules. A second binary, `romm-tui`, only launches the TUI. 

Configuration is loaded from the process environment, then a cwd `.env`, then the user config file written by `romm-cli init`. Secrets (like passwords and tokens) are stored in the OS keyring via `keyring::Entry` and loaded transparently. Note that `Commands::Init` is handled in `main.rs` *before* `load_config` so that the setup wizard can run even if no configuration exists yet.

From bottom to top:

- **Types & endpoints**
  - `types.rs` – data models used throughout the app.
  - `endpoints/*` – implementations of the `Endpoint` trait describing
    the HTTP method, path, query params, and optional body for each
    ROMM API endpoint.
- **Core services** (`src/core/`, `src/client.rs`, `src/config.rs`)
  - `Config` / `AuthConfig` – decide how to talk to ROMM (base URL and
    authentication mode).
  - `RommClient` – wraps `reqwest::Client` and uses `Endpoint` values to
    perform typed HTTP calls.
  - `RomCache` – small disk-backed cache for ROM lists, keyed by
    platform/collection.
  - `DownloadManager` – orchestrates background downloads and exposes a
    shared list of `DownloadJob`s.
- **Frontends** (`src/frontend/`)
  - **CLI** (`src/commands/*`) – one-shot commands for platforms/ROMs/API. The `frontend::cli` module routes parsed arguments to these handlers.
  - **TUI** (`src/tui/*`) – an event loop and a set of screens that
    present and manipulate the underlying data.

The CLI layer itself is split into:

- `commands::mod` – top-level `Cli` and `Commands` enum plus `OutputFormat`.
- `commands::platforms` / `commands::roms` / `commands::api` / `commands::download` / `commands::init` / `commands::update` – small modules that parse arguments, call into services, and print results.
- `commands::print` – helpers for tabular text output.
- `services` – `PlatformService` and `RomService` wrappers around endpoint calls.

There are no TUI/CLI dependencies inside the core services, which makes
it straightforward to add more frontends later.

## Data flow

Roughly:

```text
Config + env + OS Keyring
    ↓
RommClient (HTTP + auth)
    ↓
Endpoint implementations
    ↓
typed responses (types.rs)
```

The TUI and CLI both operate on the same `RommClient` and model types.

## Why an enum-based state machine?

The TUI uses:

- `AppScreen` – an enum with variants for each high-level screen (`MainMenu`, `LibraryBrowse`, `Search`, `Settings`, `Browse`, `Execute`, `Result`, `ResultDetail`, `GameDetail`, `Download`, `SetupWizard`).
- `App` – a struct that owns shared services (`RommClient`, `RomCache`, `DownloadManager`) and the current `AppScreen`. It also holds shared state like the `EndpointRegistry` (for the API browser), `server_version`, `startup_splash`, and `deferred_load_roms`.

Each key press is dispatched to a method like `handle_main_menu` or
`handle_library_browse`, which matches on `self.screen`, mutates it,
and possibly transitions to another variant.

This pattern works well in Rust because:

- The compiler forces you to handle all variants in `match` statements.
- Ownership is explicit (you often move a screen out of the enum,
  mutate it, then put it back).

You could also model screens as trait objects, but the enum-based
approach keeps everything static and easy to follow for learners.

