# romm-cli (Rust)

Rust-based CLI + TUI client for the ROMM API, written to be a
**teaching-quality Rust project**.

It is a good fit if you want to learn how to:

- Structure a small Rust application into \"core\" services and frontends
- Build a TUI with `ratatui` and `crossterm`
- Call HTTP APIs with `reqwest`
- Use async (`tokio`), `Arc<Mutex<_>>`, and disk-backed caching

---

## Overview

At a high level:

- The **core** of the app is:
  - `Config` / `AuthConfig` – load base URL and auth from the environment
  - `RommClient` – HTTP client for the ROMM API
  - `types` – strongly-typed models for platforms, ROMs, etc.
  - `endpoints` – small `Endpoint` trait impls for each API endpoint
  - `core/*` – shared cache/download/util modules reused by all frontends
- The **frontends** are:
  - The **CLI** (`commands/*` + `frontend/cli.rs`) – subcommands for platforms/ROMs/API
  - The **TUI** (`tui/*` + `frontend/tui.rs`) – interactive browser and game library UI

Because the core is UI-agnostic, you can add another frontend later by
reusing `RommClient`, `RomCache`, and `DownloadManager`.

---

## Getting started

### Prerequisites

- Rust toolchain (stable) via `rustup`
- A running ROMM server, with a reachable API base URL
  (for example `http://mill-server:1738`)

### Environment variables

Configuration is read from the environment (optionally via a `.env` file
in the repo root, using `dotenvy` in development):

- `API_BASE_URL` (required) – e.g. `http://mill-server:1738`
- **Authentication (pick one):**
  - Basic auth:
    - `API_USERNAME`
    - `API_PASSWORD`
  - Bearer token:
    - `API_TOKEN` (or `API_KEY`), must not be a placeholder like
      `your-bearer-token-here`
  - API key in a custom header:
    - `API_KEY`
    - `API_KEY_HEADER` (e.g. `X-API-Key`)

Example `.env`:

```env
API_BASE_URL=http://mill-server:1738
API_USERNAME=patrick
API_PASSWORD=your-password
```

### Build

From the `romm-cli` directory:

```bash
cargo build --release
```

The compiled binary will be at:

- `target/release/romm-cli`

---

## Running the app

### Launch the TUI

```bash
cargo run --bin romm-cli -- tui
```

This starts the interactive terminal UI:

- Browse platforms/collections and ROMs
- View game details and metadata
- Start background downloads and watch their progress

### CLI examples

List platforms:

```bash
cargo run --bin romm-cli -- platforms              # text table
cargo run --bin romm-cli -- --json platforms       # JSON (global flag)
cargo run --bin romm-cli -- platforms --json       # JSON (per-command flag)
```

Output as JSON:

```bash
cargo run --bin romm-cli -- roms --search-term "zelda" --json
```

Get help:

```bash
cargo run --bin romm-cli -- --help
cargo run --bin romm-cli -- platforms --help
```

---

## Architecture tour

At a high level the control flow looks like this:

```text
main.rs
  └── commands::run
        ├── RommClient::new(Config)
        └── frontend router
              ├── frontend::cli::run(...)  -> commands::{api,platforms,roms}
              └── frontend::tui::run(...)  -> tui::run -> App::new(...).run()
```

### Modules

- `src/main.rs`
  - Minimal binary entrypoint. Loads config, parses CLI, and calls
    `commands::run`.
- `src/config.rs`
  - `Config` + `AuthConfig` read environment variables and decide which
    auth mode to use (Basic, Bearer, or API key).
- `src/client.rs`
  - `RommClient` wraps `reqwest::Client`, a base URL, and auth headers.
  - Exposes `call<E: Endpoint>` for typed endpoints and `download_rom`
    for streaming downloads with progress callbacks.
- `src/types.rs`
  - Strongly-typed models for ROMM entities (platforms, ROMs, etc.).
- `src/endpoints/*`
  - Implements a small `Endpoint` trait per HTTP endpoint, describing
    method, path, query, and body.
- `src/core/*`
  - Frontend-agnostic shared modules:
    - `core/cache.rs` for persistent ROM cache
    - `core/download.rs` for background download job management
    - `core/utils.rs` for reusable grouping/formatting helpers
- `src/commands/*`
  - The `Cli` struct and subcommands, built with `clap`.
  - Each subcommand uses `RommClient` and the endpoint types to talk to
    the API.
- `src/frontend/*`
  - Runtime frontend routing layer.
  - `frontend/cli.rs` runs non-interactive CLI commands.
  - `frontend/tui.rs` runs the interactive TUI frontend.
- `src/tui/mod.rs` and `src/tui/app.rs`
  - The TUI \"frontend\". `App` owns shared services and the active
    `AppScreen`, and runs the event loop.
- `src/tui/screens/*`
  - Individual screens (main menu, library browser, downloads, etc.).

For a deeper dive, see the Markdown guides under `docs/` (if present).

---

## Rust concepts illustrated

This project is intentionally written to demonstrate several core Rust
ideas in a realistic but approachable codebase:

- **Ownership and borrowing**
  - `AppScreen` and `App` show how to move whole screens in and out of
    `self.screen` when changing views.
  - `Option` and enums are used to model state that is \"sometimes there\"
    (e.g., `deferred_load_roms`).
- **Enums as state machines**
  - `AppScreen` is an enum over all high-level states (screens).
  - Each `handle_*` method in `App` matches on `self.screen` and
    delegates to the correct screen type.
- **Async and concurrency**
  - `tokio::main` is used in `main.rs`.
  - `DownloadManager::start_download` uses `tokio::spawn` to run a
    background task while updating progress through `Arc<Mutex<_>>`.
- **Error handling**
  - `anyhow::Result` is used for ergonomic error propagation with
    context messages.
- **Traits and generics**
  - The `Endpoint` trait plus `RommClient::call<E: Endpoint>` show a
    simple form of typed HTTP endpoints.

You can browse generated docs with:

```bash
cargo doc --open
```

---

## Walkthrough examples

### 1. Starting the TUI and browsing ROMs

1. `main` loads `Config` and parses CLI args.
2. `commands::run` sees the `tui` subcommand and calls `tui::run`.
3. `tui::run` loads the OpenAPI spec, builds an `EndpointRegistry`, and
   constructs `App`.
4. `App::run` enters the event loop:
   - draws the current `AppScreen` (initially `MainMenuScreen`)
   - waits for key events
   - dispatches to `handle_main_menu`, `handle_library_browse`, etc.

### 2. Opening game details and starting a download

1. In the library browse screen, use ↑/↓ and Enter to select a game.
2. `handle_library_browse` calls `GameDetailScreen::new` and switches
   `self.screen` to `AppScreen::GameDetail`.
3. Press Enter on the game detail screen:
   - `handle_game_detail` calls `downloads.start_download(...)`.
   - `DownloadManager::start_download` spawns a `tokio` task that:
     - streams bytes from the `/api/roms/download` endpoint,
     - writes them to `./downloads/<rom>.zip`,
     - updates the shared job list with progress.
4. The TUI event loop keeps redrawing, so both the game detail footer
   and the Downloads screen can show a live progress bar.

### 3. Using the API browser

1. From the main menu, pick **API (Expert)**.
2. The browse screen groups endpoints by tag/path.
3. Selecting an endpoint and pressing Enter opens the execute screen,
   where you can edit query parameters and an optional JSON body.
4. Press Enter again to send the request and view a JSON/table result.

---

## Extending the app

Here are some ideas for extending the project as exercises:

- **Add a new TUI screen**
  - Create `src/tui/screens/your_screen.rs`.
  - Export it from `src/tui/screens/mod.rs`.
  - Add a new variant to `AppScreen` and update `App::render` and
    `App::handle_key` accordingly.
- **Add a new endpoint**
  - Add a new `Endpoint` implementation under `src/endpoints`.
  - Call it via `RommClient::call` from either the CLI or TUI.
- **Add a new CLI subcommand**
  - Extend the `Commands` enum in `src/commands/mod.rs`.
  - Create a corresponding `handle` function in a new module.