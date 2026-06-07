# romm-api

[![Docs.rs](https://docs.rs/romm-api/badge.svg)](https://docs.rs/romm-api)

Shared HTTP client, API types, configuration, and domain logic for RomM frontends (CLI, TUI, and future Android via UniFFI).

**Crate path:** `romm-api/` in the workspace.

---

## When to use this crate

| Use case | Dependency |
|----------|------------|
| New embedders, Android prep, minimal deps | `romm-api` directly |
| Existing library consumers on crates.io | `romm-cli` (re-exports `romm_api`) |

```toml
[dependencies]
romm-api = "1.0"
tokio = { version = "1", features = ["macros", "rt-multi-thread"] }
```

---

## Quick start

```rust,no_run
use romm_api::client::RommClient;
use romm_api::config::load_config;
use romm_api::error::RommError;

#[tokio::main]
async fn main() -> Result<(), RommError> {
    let config = load_config()?;
    let client = RommClient::new(&config, false)?;

    let version = client.rom_server_version_from_heartbeat().await;
    println!("Connected to RomM server version: {:?}", version);

    Ok(())
}
```

Run `cargo doc -p romm-api --open` for the full API reference.

---

## Features

- **HTTP client** — typed `RommClient` with one struct per API route ([http-client.md](http-client.md))
- **Configuration** — layered merge of defaults, `config.json`, env vars, and OS keyring (below)
- **Caching** — ROM list and library metadata snapshots for fast frontend startup
- **Downloads** — authenticated streaming downloads, zip safety, concurrent download manager
- **Library scan** — helpers to trigger and poll server `scan_library` jobs
- **Save sync** — device registration, manifest planning, and sync execution types
- **OpenAPI** — fetch/cache server OpenAPI for capability checks (`feature_compat`)
- **Errors** — typed `RommError` hierarchy with exit-code mapping for frontends
- **Self-update helpers** — shared GitHub release lookup used by CLI and TUI binaries

Changelog: [romm-api/CHANGELOG.md](../romm-api/CHANGELOG.md).

---

## Module overview

| Module | Purpose |
|--------|---------|
| `client` | `RommClient` — typed HTTP calls, downloads, uploads |
| `endpoints` | One type per API route (`Endpoint` trait) |
| `types` | Shared serde models |
| `config` | `load_config`, auth, paths, keyring integration |
| `error` | `ApiError`, `ConfigError`, `DownloadError`, `RommError`, `exit` codes |
| `core` | Cache, downloads, library scan helpers, resolve utilities |
| `openapi` | OpenAPI fetch/cache and endpoint lookup |
| `feature_compat` | Server capability checks from OpenAPI |
| `update` | Self-update helpers (used by CLI/TUI binaries) |

See [HTTP client and endpoints](http-client.md) for request/response patterns and streaming downloads.

---

## Configuration

All frontends share the same configuration layer in `romm-api::config`.

### Setup wizard (CLI)

The easiest way to create `config.json` is through the CLI:

```bash
romm-cli init
```

### Config file location

`config.json` lives under the OS config directory (for example `~/.config/romm-cli/config.json` on Unix, or `%APPDATA%\romm-cli\config.json` on Windows).

`API_BASE_URL` should match the RomM **website** address from your browser (scheme, host, port only), for example `https://romm.example.com` or `http://my-server:1738`.

The library does **not** auto-load a `.env` file. Set environment variables in your shell or process manager.

**Auth problems (keyring, Docker, CI, Windows credentials):** see [troubleshooting-auth.md](troubleshooting-auth.md).

### API token (recommended)

```bash
romm-cli init --url https://romm.example.com --token-file ~/.romm-token --check
```

Prefer `--token-file` over `--token` to keep secrets out of shell history. The CLI stores the token in the OS keyring when available.

**Non-interactive `init` flags:**
- `--url <URL>` — RomM origin (browser-style).
- `--token <TOKEN>` — Bearer token string.
- `--token-file <PATH>` — Read token from UTF-8 file. Use `-` for stdin.
- `--download-dir <PATH>` — Override the default ROMs directory.
- `--no-https` — Disable HTTPS (use HTTP instead).
- `--check` — Verify URL and token after saving.
- `--force` — Overwrite existing configuration without asking.
- `--print-path` — Print the path to `config.json` and exit.

### Environment variables

| Variable | Description |
|----------|-------------|
| `API_BASE_URL` | RomM site URL (browser address, no `/api`) |
| `ROMM_ROMS_DIR` | Base directory for stored ROMs (preferred) |
| `ROMM_DOWNLOAD_DIR` | Legacy alias for `ROMM_ROMS_DIR` |
| `API_USE_HTTPS` | Set to `false` to disable automatic upgrade to HTTPS |
| `API_USERNAME` / `API_PASSWORD` | Basic Auth credentials |
| `API_TOKEN` | Bearer token |
| `ROMM_TOKEN_FILE` | Path to bearer token file (alias: `API_TOKEN_FILE`). Max 64 KiB. |
| `API_KEY_HEADER` / `API_KEY` | Custom API key header and value |
| `ROMM_CACHE_PATH` | Override ROM list cache path |
| `ROMM_LIBRARY_METADATA_SNAPSHOT_PATH` | Override TUI library metadata snapshot path |
| `ROMM_OPENAPI_BASE_URL` | OpenAPI origin if different from `API_BASE_URL` |
| `ROMM_OPENAPI_PATH` | Override downloaded OpenAPI cache path |
| `ROMM_USER_AGENT` | Override HTTP `User-Agent` |
| `ROMM_VERBOSE` | Verbose HTTP logging (`1`/`true`) |
| `ROMM_CHECK_UPDATES` | Set to `false`/`0`/`no`/`off` to disable update checks |
| `ROMM_THEME` | TUI color theme ID (see [TUI documentation](tui.md)) |
| `ROMM_GITHUB_API_BASE` | Override GitHub API base for self-update |
| `ROMM_GITHUB_RELEASES_API` | Override GitHub releases list URL (component tag filtering) |
| `ROMM_GITHUB_LATEST_RELEASE_API` | Legacy override; redirects to releases list when it points at `/releases/latest` |

Test-only variables (`ROMM_TEST_*`) are omitted. For auth precedence and keyring behavior, see [troubleshooting-auth.md](troubleshooting-auth.md).

### Configuration precedence

Settings merge **per field** from lowest to highest precedence:

1. **Built-in defaults** — HTTPS enabled, default theme, OS download folder.
2. **`config.json`** — written by `romm-cli init`, TUI setup, Settings, or `auth login`.
3. **Environment variables** — override `config.json` for matching fields.
4. **OS keyring** — resolves `<stored-in-keyring>` placeholders after env + file merge.
5. **Command-specific CLI runtime** — only where documented (e.g. `download --output`).

**Examples:**

- `config.json` has `base_url` `https://romm.local`, but `API_BASE_URL=https://romm.prod` is set → **`https://romm.prod`** is used.
- `romm-cli download --output /tmp/roms` uses **`/tmp/roms`** for that run.

There are no global `--url` / `--token` flags on normal commands. Connection settings come from env + file + keyring.

### Custom console paths

Default download layout: `{base Roms Dir}/{platform-slug}/`.

Map individual consoles in `config.json` or via TUI **Settings → ROMs → Console paths**:

```json
{
  "download_dir": "C:\\Games\\romm-cli",
  "roms_layout": {
    "platform_dirs": {
      "7": "D:\\Roms\\Switch",
      "3": "E:\\Roms\\NES"
    }
  }
}
```

Keys in `platform_dirs` are RomM platform IDs. Unmapped consoles use the base subfolder.

### Custom console save paths

Default save layout: `{Save Dir}/{platform-slug}/{game}/`.

Map consoles in **Settings → Saves → Save console paths** or `config.json`:

```json
{
  "save_sync": {
    "save_dir": "C:\\Games\\romm-cli\\saves",
    "platform_dirs": {
      "7": "D:\\Saves\\Switch",
      "3": "E:\\Saves\\NES"
    }
  }
}
```

---

## Errors and exit codes

Library methods return typed errors from `romm_api::error`. The CLI binary maps them to process exit codes via `romm_api::exit`:

| Code | Meaning |
|------|---------|
| 0 | Success |
| 1 | General failure |
| 2 | Usage error (invalid flags) |
| 3 | Config / auth |
| 4 | API / network / download |

See [CLI documentation](cli.md#exit-codes) for scripting examples.

---

## Workspace utilities

| Binary | Crate | Purpose |
|--------|-------|---------|
| `romm-openapi-gen` | `romm-api` | Regenerate OpenAPI-derived helpers |

---

## Related documentation

- [Changelog](../romm-api/CHANGELOG.md)
- [HTTP client and endpoints](http-client.md)
- [Architecture overview](architecture.md)
- [Troubleshooting authentication](troubleshooting-auth.md)
- [Workspace split ADR](plans/2026-06-06-workspace-split-adr.md)
- [Android frontend design](plans/2026-06-06-android-frontend-design.md)
