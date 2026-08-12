# CLI ⊇ TUI parity — design spec

**Date:** 2026-08-12  
**Status:** Approved  
**Scope:** Phase 1 — config registry (approach D), `saves`, `collections` CLI commands  

## Problem

`romm-cli` and `romm-tui` share `romm-api`, but feature parity is uneven:

- TUI can list/upload/download saves per game, browse collections, and edit full config via Settings — CLI lacks direct equivalents.
- CLI exposes `roms props`, notes, delete, find, filters, manifest sync, and batch download — TUI lacks these (Phase 2).
- Config fields like `save_sync`, `extras_defaults`, and `roms_layout.platform_dirs` exist only in `config.json`; env override coverage is incomplete, and there is no source-aware introspection.

**Out of scope (all phases):** emulator launch, in-browser play, netplay hosting, QR pairing UI.

## Goals

1. **CLI ⊇ TUI (Phase 1):** Every TUI library-management action has a scriptable CLI equivalent.
2. **Config approach D:** Full field registry, env mapping for every persistent field, source-aware `config show`, file-only `config set`.
3. **Shared core:** New behavior lives in `romm-api` (`config/registry`, `core/saves`, `core/collections`); CLI commands stay thin.
4. **Automation-first:** All new commands support `--json` with stable shapes documented in `json-output.md`.

## Non-goals (Phase 1)

- TUI UI for CLI-only features (props, notes, delete) — Phase 2.
- Global one-shot `--base-url` / `--token-file` on every subcommand — Phase 1c (optional follow-up).
- Firmware, states, play-sessions, platform PUT, collection create — Phase 3+.
- Bulk metadata rescrape (WebSocket) — remains web UI / future.

---

## Architecture

```text
romm-api/
├── config/
│   ├── registry.rs      # ConfigKey, env names, get/set, source tracking
│   └── ... (existing config.rs refactored to call registry)
└── core/
    ├── saves.rs         # list/get/download/upload helpers
    └── collections.rs   # list/get/delete by type

romm-cli/
└── commands/
    ├── config.rs
    ├── saves.rs
    └── collections.rs
```

### Config precedence (unchanged layers, completed coverage)

1. Built-in defaults  
2. `config.json`  
3. Environment variables (**every** `Config` field mapped; legacy names kept)  
4. OS keyring (secret resolution)  
5. Command-specific runtime flags (existing: `download --output`, `sync run --download-dir`)  
6. *(Phase 1c)* Global CLI overrides: `--base-url`, `--token-file`, etc.

`config set` writes layer 2 only. It never mutates the process environment.

### Config introspection

| Command | Output |
|---------|--------|
| `config show` | Effective merged config (secrets redacted) |
| `config show --file` | On-disk JSON only |
| `config show --sources --json` | Per-field `{ value, source }` where `source` is `default`, `file`, `env:VAR`, or `keyring` |
| `config env-map [key]` | Env var name(s) for a dotted key |
| `config set <key> <value>` | Patch one field in `config.json` via registry |
| `config path` | Path to `config.json` |
| `config reset --yes` | Delete config file (+ clear keyring entries, matching TUI reset) |

### Config key paths (Phase 1)

| Key path | Env var(s) | Notes |
|----------|------------|-------|
| `base_url` | `API_BASE_URL` | legacy name kept |
| `download_dir` | `ROMM_ROMS_DIR`, `ROMM_DOWNLOAD_DIR` | legacy alias |
| `use_https` | `API_USE_HTTPS` | |
| `theme` | `ROMM_THEME` | |
| `extras_defaults.include_related_roms` | `ROMM_EXTRAS_INCLUDE_RELATED_ROMS` | bool `true`/`false`/`1`/`0` |
| `extras_defaults.include_cover` | `ROMM_EXTRAS_INCLUDE_COVER` | |
| `extras_defaults.include_manual` | `ROMM_EXTRAS_INCLUDE_MANUAL` | |
| `save_sync.save_dir` | `ROMM_SAVE_SYNC_SAVE_DIR` | |
| `save_sync.device_id` | `ROMM_SAVE_SYNC_DEVICE_ID` | |
| `save_sync.platform_dirs.<id>` | `ROMM_SAVE_SYNC_PLATFORM_DIR_<id>` | per platform |
| `roms_layout.platform_dirs.<id>` | `ROMM_ROMS_PLATFORM_DIR_<id>` | per platform |
| `tui_layout.*` | *(file only in Phase 1)* | TUI-specific; shown in `config show`, set via `config set` |
| `auth.*` | existing `API_*` vars | `config set` for auth discouraged; use `auth login` |

Bulk override for maps (optional): `ROMM_ROMS_LAYOUT_JSON`, `ROMM_SAVE_SYNC_PLATFORM_DIRS_JSON` as JSON objects merging into the respective maps (env wins over file for keys present).

Secrets in `config show`: always `<redacted>` unless `--reveal-secrets` on a TTY with confirmation prompt.

---

## CLI: `saves`

Mirrors TUI game-detail Saves tab (`u` upload, `D` download, list on tab open).

| Subcommand | API |
|------------|-----|
| `saves list [--rom-id] [--device-id] [--slot]` | `GET /api/saves` |
| `saves get <id>` | `GET /api/saves/{id}` |
| `saves download <id> [--output]` | `GET /api/saves/{id}/content` |
| `saves upload --rom-id <id> <file> [--emulator] [--slot] [--device-id] [--overwrite]` | `POST /api/saves` |

Default download path: resolved save dir from config (`resolved_save_dir` + platform slug when `--rom-id` known).

OpenAPI gate: none required beyond authenticated saves endpoints (distinct from full save-sync negotiate flow).

---

## CLI: `collections`

Mirrors TUI library collections pane.

| Subcommand | API |
|------------|-----|
| `collections list [--type manual\|smart\|virtual\|all]` | GET manual/smart/virtual |
| `collections get <id> [--type manual\|smart\|virtual]` | GET by type |
| `collections delete <id> [--type manual\|smart] [--yes]` | DELETE (virtual: not deletable; error with clear message) |

Default `--type all` for list merges via existing `merge_all_collection_sources`.

---

## Error handling & exit codes

- Use existing `RommError` / `exit_code()` mapping.
- Unknown config keys → usage error (exit 2).
- Invalid bool/path values → config error (exit 3).
- API failures → exit 4.

---

## Testing strategy

- **Unit (`romm-api`):** registry parse/set, env merge precedence, source attribution.
- **Integration (`romm-cli`):** `assert_cmd` + `httpmock` for saves/collections; config tests with `ROMM_TEST_CONFIG_DIR`.
- **Docs:** `cli.md`, `api.md` env table, `json-output.md` schemas.

---

## Phased roadmap

| Phase | Deliverable |
|-------|-------------|
| **1 (this spec)** | Config D + `saves` + `collections` CLI |
| **1c** | Global CLI connection overrides |
| **2** | TUI: props, notes, delete, find, filters, manifest sync UX |
| **3** | Firmware, states, play-sessions, stats |
| **4** | Admin: users, server config, feeds, exports (CLI-only) |

---

## Acceptance criteria (Phase 1)

- [ ] `romm-cli config show --sources --json` reports correct source for env-overridden and file-only fields.
- [ ] Setting `save_sync.device_id` via env overrides file; `config set save_sync.device_id x` persists to file.
- [ ] `saves upload/download/list` works against httpmock fixtures; matches TUI upload/download behavior for the same ROM.
- [ ] `collections list --type all` returns merged manual + smart + virtual JSON.
- [ ] All new subcommands documented with `--json` examples.
- [ ] `cargo test`, `cargo clippy -D warnings`, `cargo fmt --check` pass.
