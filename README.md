# 🎮 romm-cli

[![Crates.io](https://img.shields.io/crates/v/romm-cli.svg)](https://crates.io/crates/romm-cli)
[![Docs.rs](https://docs.rs/romm-cli/badge.svg)](https://docs.rs/romm-cli)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![CI](https://github.com/patricksmill/romm-cli/actions/workflows/ci.yml/badge.svg)](https://github.com/patricksmill/romm-cli/actions/workflows/ci.yml)

**A powerful, blazing-fast Rust CLI and TUI for managing your game library via the [ROMM API](https://github.com/romm-retro/romm).**

Whether you're a terminal power user who likes to script their downloads or an enthusiast who wants a beautiful, lightweight interface to browse their collection, `romm-cli` has you covered.

---

## ✨ Features

- 🚀 **Dual Frontends**: A full-featured CLI for scripting and a rich, interactive TUI for browsing.
- 📂 **Library Browser**: Search, filter, and view detailed metadata for your games.
- 📥 **Background Downloads**: Start downloads in the TUI and keep browsing while they finish.
- 🔐 **Secure Auth**: Support for Basic Auth, Bearer tokens, and Bearer-only API keys.
- 💾 **Disk Caching**: Efficient caching of your game list for near-instant loading.
- 🛠️ **Expert API Browser**: Explore your ROMM server's OpenAPI spec and execute any endpoint directly from the terminal.
- 🥧 **Cross-Platform**: Built for Windows, Linux, and macOS (including ARM).

---

## 🚀 Getting Started

### Quick Install (Rust Users)
If you have Rust installed, the easiest way to get started is via `cargo`:

```bash
cargo install romm-cli
```

*Note: The TUI feature is enabled by default. For a CLI-only build, use `--no-default-features`.*

### Binary Downloads
Prebuilt binaries for Windows, Linux, and macOS are available on the [Releases page](https://github.com/patricksmill/romm-cli/releases).

---

## 🛠️ Configuration

Run the interactive setup wizard to get connected:

```bash
romm-cli init
```

This will guide you through setting your `API_BASE_URL` and authentication. Config is stored in your OS-standard config directory (e.g., `~/.config/romm-cli/.env`).

### Environment Variables
For advanced usage or development, you can set these in your shell or a local `.env`:

| Variable | Description |
|----------|-------------|
| `API_BASE_URL` | Your ROMM server URL (e.g., `http://my-server:1738`) |
| `API_USERNAME` / `API_PASSWORD` | Basic Auth credentials |
| `API_TOKEN` / `API_KEY` | Bearer token or API Key |
| `ROMM_VERBOSE` | Set to `1` to enable request logging |

---

## 📖 Usage

### Launch the TUI
```bash
romm-cli tui
# OR just:
romm-tui
```

### CLI Commands
The CLI is designed to be pipe-friendly and supports JSON output:

```bash
# List all platforms
romm-cli platforms

# Search for a game and output as JSON
romm-cli roms --search-term "zelda" --json

# Trigger a self-update
romm-cli update
```

---

## 🏗️ Project Architecture

Built with a focus on modularity and "teaching-quality" Rust patterns:

- **client**: Generic, trait-based HTTP client for API interaction.
- **tui**: Interactive layer built using `ratatui` and `crossterm`.
- **frontend**: Routing layer that separates presentation from core logic.
- **core**: Shared services for caching and download management.

---

## 🤝 Contributing

Contributions are welcome! Please feel free to open issues or submit pull requests. If you're building from source:

```bash
git clone https://github.com/patricksmill/romm-cli
cd romm-cli
cargo build --release
```

---

## 📜 License

This project is licensed under the [MIT License](LICENSE).