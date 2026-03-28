# romm-cli

[![Crates.io](https://img.shields.io/crates/v/romm-cli.svg)](https://crates.io/crates/romm-cli)
[![Docs.rs](https://docs.rs/romm-cli/badge.svg)](https://docs.rs/romm-cli)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![CI](https://github.com/patricksmill/romm-cli/actions/workflows/ci.yml/badge.svg)](https://github.com/patricksmill/romm-cli/actions/workflows/ci.yml)

Rust CLI and TUI for managing a game library through the [ROMM API](https://github.com/romm-retro/romm). Use the CLI for scripting and automation, or the TUI for interactive browsing.

---

## Features

- **CLI and TUI**: Command-line interface for scripts plus an interactive terminal UI.
- **Library browsing**: Search, filter, and inspect game metadata.
- **Background downloads**: Start downloads in the TUI and keep browsing while they run.
- **Authentication**: Basic Auth, Bearer tokens, and Bearer-only API keys.
- **Caching**: Game list caching for faster repeat loads.
- **API browser**: Inspect the ROMM server OpenAPI spec and call endpoints from the terminal.
- **Cross-platform**: Windows, Linux, and macOS (including ARM).

---

## Getting started

### Install with Cargo

If you have Rust installed:

```bash
cargo install romm-cli
```

The TUI is enabled by default. For a CLI-only build, use `--no-default-features`.

### Binary releases

Prebuilt binaries for Windows, Linux, and macOS are on the [Releases page](https://github.com/patricksmill/romm-cli/releases).

---

## Configuration

Run the setup wizard:

```bash
romm-cli init
```

This sets `API_BASE_URL` and authentication. Configuration lives in your OS config directory (for example `~/.config/romm-cli/.env` on Unix).

### Environment variables

Set these in your shell or a local `.env` for advanced use:

| Variable | Description |
|----------|-------------|
| `API_BASE_URL` | ROMM server URL (e.g. `http://my-server:1738`) |
| `API_USERNAME` / `API_PASSWORD` | Basic Auth credentials |
| `API_TOKEN` / `API_KEY` | Bearer token or API key |
| `ROMM_OPENAPI_BASE_URL` | Optional. Web origin for `/openapi.json` if it differs from `API_BASE_URL` (same host/scheme as the RomM UI). |
| `ROMM_OPENAPI_PATH` | Optional. Path to a local `openapi.json` instead of the default cache file. |
| `ROMM_VERBOSE` | Set to `1` to log HTTP requests |

---

## Usage

### TUI

```bash
romm-cli tui
# or:
romm-tui
```

### CLI

The CLI supports JSON output where applicable:

```bash
# List platforms
romm-cli platforms

# Search and print JSON
romm-cli roms --search-term "zelda" --json

# Self-update
romm-cli update
```

---

## Project layout

- **client**: HTTP client for the API.
- **tui**: Terminal UI (`ratatui`, `crossterm`).
- **frontend**: Routing between CLI and shared logic.
- **core**: Caching and download handling.

---

## Contributing

Issues and pull requests are welcome. To build from source:

```bash
git clone https://github.com/patricksmill/romm-cli
cd romm-cli
cargo build --release
```

---

## License

This project is licensed under the [MIT License](LICENSE).

---

*Creation assisted with AI; content reviewed by the maintainers.*
