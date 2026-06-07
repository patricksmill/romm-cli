# romm-cli

[![Crates.io](https://img.shields.io/crates/v/romm-cli.svg)](https://crates.io/crates/romm-cli)
[![Docs.rs](https://docs.rs/romm-cli/badge.svg)](https://docs.rs/romm-cli)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![CI](https://github.com/patricksmill/romm-cli/actions/workflows/ci.yml/badge.svg)](https://github.com/patricksmill/romm-cli/actions/workflows/ci.yml)

Rust clients for managing a game library through the [RomM API](https://github.com/romm-retro/romm). This repository is a **Cargo workspace** with a shared API library and two frontends: a scriptable CLI and an interactive TUI.

---

## Documentation

| Crate | Guide | crates.io / docs.rs |
|-------|-------|---------------------|
| **`romm-api`** | [API & configuration](docs/api.md) | [crates.io](https://crates.io/crates/romm-api) · [docs.rs](https://docs.rs/romm-api) |
| **`romm-cli`** | [CLI commands & scripting](docs/cli.md) | [crates.io](https://crates.io/crates/romm-cli) · [docs.rs](https://docs.rs/romm-cli) |
| **`romm-tui`** | [Terminal UI](docs/tui.md) | [crates.io](https://crates.io/crates/romm-tui) · [docs.rs](https://docs.rs/romm-tui) |

### Shared topics

- [Releases](docs/releases.md) — per-crate versioning, tags, and maintainer runbook
- [Architecture](docs/architecture.md) — workspace layout, layers, TUI state machine
- [HTTP client](docs/http-client.md) — `RommClient`, endpoints, streaming downloads
- [Save sync](docs/save-sync.md) — manifest format, `sync` subcommands
- [Post-upload library scan](docs/scan-after-upload.md) — `--scan`, `scan --wait`, cache
- [JSON output](docs/json-output.md) — `--json` field reference
- [Troubleshooting authentication](docs/troubleshooting-auth.md) — keyring, Docker, CI, Windows

---

## Quick start

**CLI** (scripting and automation):

```bash
cargo install romm-cli
romm-cli init
romm-cli platforms
```

**TUI** (interactive browsing):

```bash
cargo install romm-tui
romm-tui
```

**Library** (embed `romm-api` in your own Rust project):

```toml
[dependencies]
romm-api = "1.0"
```

See [docs/api.md](docs/api.md) for a minimal `RommClient` example.

Prebuilt binaries: [GitHub Releases](https://github.com/patricksmill/romm-cli/releases) (`romm-cli-v*` ships the CLI; `romm-tui-v*` ships the TUI). See [docs/releases.md](docs/releases.md).

---

## What's in each crate

| Crate | Role | Highlights |
|-------|------|------------|
| **[romm-api](docs/api.md)** | Shared library | `RommClient`, config, caching, downloads, save sync, typed errors |
| **[romm-cli](docs/cli.md)** | Scripting CLI | Resource–action commands, `--json`, shell completions, `auth`, `sync`, self-update |
| **[romm-tui](docs/tui.md)** | Terminal UI | Library browse, search, game detail with covers, background downloads, theming |

Each crate has its own [changelog](CHANGELOG.md) and release tags. User-facing features, screenshots, and release notes live on the matching crate page—not duplicated here.

---

## Workspace layout

```text
romm-cli/          # workspace root (this repo)
├── romm-api/      # RommClient, endpoints, core, config, errors
├── romm-cli/      # commands, CLI binary, shell completions
└── romm-tui/      # TUI screens, event loop, romm-tui binary
```

- New embedders: depend on **`romm-api`** directly.
- Existing library consumers: **`romm-cli`** re-exports `romm_api` for backward compatibility.
- Android (Kotlin/Compose + UniFFI) lives in the separate [**romm-rust-android**](https://github.com/patricksmill/romm-rust-android) repo and depends on `romm-api` from crates.io.

---

## Contributing

Issues and pull requests are welcome. To build from source:

```bash
git clone https://github.com/patricksmill/romm-cli
cd romm-cli
cargo build --release
```

Contributor notes: [rust-guidelines.md](docs/rust-guidelines.md).

---

## License

This project is licensed under the [MIT License](LICENSE).

---

*Creation assisted with AI; content reviewed by the maintainers.*
