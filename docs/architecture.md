# Architecture overview

This document gives a slightly deeper view of how the project is
structured internally. It is meant to be read alongside the generated
Rustdoc (`cargo doc --open`).

## High-level layers

From bottom to top:

- **Types & endpoints**
  - `types.rs` – data models used throughout the app.
  - `endpoints/*` – implementations of the `Endpoint` trait describing
    the HTTP method, path, query params, and optional body for each
    ROMM API endpoint.
- **Core services**
  - `Config` / `AuthConfig` – decide how to talk to ROMM (base URL and
    authentication mode).
  - `RommClient` – wraps `reqwest::Client` and uses `Endpoint` values to
    perform typed HTTP calls.
  - `RomCache` – small disk-backed cache for ROM lists, keyed by
    platform/collection.
  - `DownloadManager` – orchestrates background downloads and exposes a
    shared list of `DownloadJob`s.
- **Frontends**
  - **CLI** (`commands/*`) – one-shot commands for platforms/ROMs/API.
  - **TUI** (`tui/*`) – an event loop and a set of screens that
    present and manipulate the underlying data.

The CLI layer itself is split into:

- `commands::mod` – top-level `Cli` and `Commands` enum plus
  `OutputFormat`.
- `commands::platforms` / `commands::roms` / `commands::api` – small
  modules that parse arguments, call into services, and print results.
- `commands::print` – helpers for tabular text output.
- `services` – `PlatformService` and `RomService` plus traits
  `PlatformApi` / `RomApi` for testability.

There are no TUI/CLI dependencies inside the core services, which makes
it straightforward to add more frontends later.

## Data flow

Roughly:

```text
Config + env
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

- `AppScreen` – an enum with variants for each high-level screen
- `App` – a struct that owns shared services and the current `AppScreen`

Each key press is dispatched to a method like `handle_main_menu` or
`handle_library_browse`, which matches on `self.screen`, mutates it,
and possibly transitions to another variant.

This pattern works well in Rust because:

- The compiler forces you to handle all variants in `match` statements.
- Ownership is explicit (you often move a screen out of the enum,
  mutate it, then put it back).

You could also model screens as trait objects, but the enum-based
approach keeps everything static and easy to follow for learners.

