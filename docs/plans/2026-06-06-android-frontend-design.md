# Android frontend design

**Date:** 2026-06-06  
**Status:** Approved — preparatory phase in progress

## Summary

Add an Android client for RomM using **Kotlin/Compose** for UI and **Rust `romm-api`** (via UniFFI/JNI) for HTTP, types, and domain logic. The first shipped milestone is **browse-only**: connect to a server, list platforms and ROMs, and view ROM metadata. Downloads, saves, scan, and full settings parity are deferred.

This design triggers [Gap 4 workspace split](2026-06-06-workspace-split-adr.md) trigger #3 (third frontend).

## Goals

- Reuse existing `RommClient`, endpoints, and core logic — no duplicated HTTP layer in Kotlin.
- Keep Android UI idiomatic (Material/Compose, Android lifecycle, Keystore).
- Ship a minimal browse experience before investing in downloads or background sync.

## Non-goals (this phase and v1)

- UniFFI scaffold or Kotlin project (next phase).
- `SecretStore` / `AppPaths` platform traits (next phase; keyring stays desktop-only in `romm-api` for now).
- ROM downloads to device storage.
- Play Store release pipeline.
- Feature parity with CLI/TUI.

## Architecture

```text
┌─────────────────────────────────────┐
│  Android app (Kotlin / Compose)     │
│  - Activities / Navigation          │
│  - ViewModels                       │
│  - Setup wizard UI                  │
└──────────────┬──────────────────────┘
               │ UniFFI / JNI (next phase)
┌──────────────▼──────────────────────┐
│  romm-api (Rust lib, .so via NDK)   │
│  - RommClient, endpoints, types     │
│  - core (roms, cache, resolve)      │
│  - config + error                   │
└──────────────┬──────────────────────┘
               │ HTTPS
┌──────────────▼──────────────────────┐
│  RomM server                        │
└─────────────────────────────────────┘
```

### Workspace layout (after split)

```text
romm-cli/                 # workspace root
├── romm-api/             # shared library (future UniFFI target)
├── romm-cli/             # CLI binary + commands
└── romm-tui/             # TUI binary
```

Future addition (not in this phase):

```text
├── android/              # Gradle project (Kotlin/Compose)
└── romm-api/             # + uniffi.toml / generated Kotlin bindings
```

### Dependency rules

| Crate | Depends on | Must not depend on |
|-------|------------|-------------------|
| `romm-api` | reqwest, serde, tokio, keyring, … | clap, ratatui, dialoguer, indicatif |
| `romm-cli` | `romm-api` | `romm-tui` (optional feature for `romm-cli tui` subcommand only) |
| `romm-tui` | `romm-api` | `romm-cli` |
| Android app | generated UniFFI bindings | — |

**Note:** `library_scan` core logic moves to `romm-api::core::library_scan` so TUI and CLI share scan helpers without `romm-tui` → `romm-cli` coupling.

## Browse-only v1 FFI surface (planned)

UniFFI types and methods to expose from `romm-api` in the next phase:

| Method | Purpose |
|--------|---------|
| `RommSession::connect(config)` | Build `RommClient` from base URL + auth |
| `RommSession::server_version()` | Heartbeat / version check |
| `RommSession::list_platforms()` | Platform list |
| `RommSession::list_roms(platform_id, page, search)` | Paginated ROM list |
| `RommSession::get_rom(rom_id)` | ROM metadata for detail screen |

Auth in v1: bearer token or API key passed from Kotlin setup UI (no OS keyring on Android until `SecretStore` trait exists).

Errors: map `RommError` / `ApiError` to UniFFI-friendly enums with user-facing messages.

## Android app structure (planned)

```text
android/
├── app/src/main/kotlin/.../
│   ├── MainActivity.kt
│   ├── navigation/          # NavHost: Setup → Platforms → Library → Detail
│   ├── setup/               # Server URL + token
│   ├── platforms/           # Platform grid/list
│   ├── library/             # ROM list per platform
│   └── detail/              # ROM metadata
└── app/src/main/jniLibs/    # per-ABI libromm_api.so (built via cargo-ndk)
```

UI stack: Jetpack Compose, ViewModel + coroutines, Material 3.

## Platform adaptations (deferred)

| Desktop (`romm-api` today) | Android (future) |
|----------------------------|------------------|
| `keyring` crate | Android Keystore via `SecretStore` trait |
| `dirs` / user home paths | App-scoped storage via `AppPaths` trait |
| `config.json` in XDG-style dir | `filesDir` / DataStore |
| `self_update` | Play Store / sideload updates |

## Testing strategy

| Layer | Tests |
|-------|-------|
| `romm-api` | Existing unit + integration tests; `wiremock` HTTP mocks |
| UniFFI (next phase) | Rust round-trip tests for exposed types |
| Android (next phase) | JVM instrumented tests against mock server; manual device testing |

## Preparatory phase (this work)

1. Write this design doc and [implementation plan](./2026-06-06-android-prep-implementation.md).
2. Execute workspace split per [migration playbook](./2026-06-06-workspace-split-migration.md).
3. Update ADR and `rust-guidelines.md` Gap 4 — Android fires trigger #3.
4. Re-export `romm_api` from `romm-cli` lib where needed for crates.io backward compatibility.

## References

- [Workspace split ADR](./2026-06-06-workspace-split-adr.md)
- [Workspace split migration playbook](./2026-06-06-workspace-split-migration.md)
- [rust-guidelines.md — Gap 4](../rust-guidelines.md)
- [architecture.md](../architecture.md)
