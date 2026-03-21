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

### Install from release (recommended for end users)

Prebuilt binaries are available on [GitHub Releases](https://github.com/patricksmill/romm-cli/releases). Download the archive for your platform:

| Platform        | File |
|-----------------|------|
| Windows x86_64  | `romm-cli-windows-x86_64.zip` |
| Linux x86_64    | `romm-cli-linux-x86_64.tar.gz` |
| Linux aarch64   | `romm-cli-linux-aarch64.tar.gz` |
| macOS x86_64    | `romm-cli-macos-x86_64.tar.gz` |
| macOS arm64     | `romm-cli-macos-aarch64.tar.gz` |

**Windows:** Extract the zip, then run `romm-cli.exe` from a terminal.

**Linux / macOS:** Extract the archive, make the binary executable, and run it:

```bash
tar -xzf romm-cli-linux-x86_64.tar.gz   # or macos variant
chmod +x romm-cli
./romm-cli --help
```

Optional: move `romm-cli` to a directory in your `PATH` (e.g. `~/.local/bin`).

SHA256 checksums for all assets are included as `checksums.txt` in each release.

### Build from source

**Prerequisites:**

- Rust toolchain (stable) via `rustup`
- A running ROMM server, with a reachable API base URL
  (for example `http://mill-server:1738`)

### Environment variables

Configuration is read from the environment (optionally via a `.env` file
in the repo root, using `dotenvy` in development):

- `API_BASE_URL` (required) – e.g. `http://mill-server:1738`
- **Authentication (first match wins):**
  1. **Basic auth** – if both `API_USERNAME` and `API_PASSWORD` are set, they are used (other auth env vars are ignored for the request).
  2. **Custom header** – if both `API_KEY` and `API_KEY_HEADER` are set and the key is not treated as a placeholder, the key is sent in that header.
  3. **Bearer** – otherwise, if `API_TOKEN` or `API_KEY` is set and not a placeholder (`your-…`, `placeholder`, empty), it is sent as `Authorization: Bearer …`.

  Placeholder-like bearer/API key values are skipped so a template `.env` does not accidentally authenticate.

  Note: `API_KEY` alone is ambiguous: with `API_KEY_HEADER` it is the header secret; without it, the same value is used as a Bearer token.

**Optional (TUI / cache / downloads):**

- `ROMM_OPENAPI_PATH` – path to OpenAPI JSON for the expert API browser (default `openapi.json` in the working directory).
- `ROMM_CACHE_PATH` – path to the on-disk ROM list cache file (default `romm-cache.json`).
- `ROMM_DOWNLOAD_DIR` – directory for TUI background downloads (default `./downloads`). If a file name already exists, the client uses `name__2.zip`, `name__3.zip`, etc.

**CLI:**

- `-v` / `--verbose` – log each HTTP request’s method, path, query **parameter names** (not values), status code, and duration on stderr (no secrets).

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
        ├── RommClient::new(Config, verbose)
        └── frontend router
              ├── frontend::cli::run(...)  -> commands::{api,platforms,roms}
              └── frontend::tui::run(...)  -> tui::run -> App::new(...).run()
```

### Modules

- `src/lib.rs`
  - Library crate root (`romm_cli`); same modules as below for use by tests and `romm_openapi_gen`.
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