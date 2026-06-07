# ADR: Workspace split (Android prep)

**Date:** 2026-06-06  
**Status:** Accepted — workspace split **completed** (trigger #3: Android frontend)

## Maintainer decision

The project is a **Cargo workspace** with three members: `romm-api`, `romm-cli`, and `romm-tui`. The split was executed to prepare for an Android browse-only client (Kotlin/Compose + UniFFI over `romm-api`). Android development lives in [**romm-rust-android**](https://github.com/patricksmill/romm-rust-android).

## Trigger

**#3 — Third frontend:** Android app needs shared `RommClient` / core logic without pulling `clap` or `ratatui`.

## Context

Previously a single package hosted the library plus four binaries. TUI was feature-gated; CI validated `--no-default-features` builds. Compile times were acceptable, but a clean `romm-api` boundary is required for `cargo-ndk` / UniFFI in the next phase.

## Decision

Split into workspace members per [migration playbook](./2026-06-06-workspace-split-migration.md):

```text
romm-cli/          # workspace root
├── romm-api/      # client, endpoints, core, config, error, types, …
├── romm-cli/      # commands, CLI binary, completions
└── romm-tui/      # TUI binary (depends on romm-api only)
```

- `romm-cli` re-exports `romm_api` for crates.io backward compatibility.
- `library_scan` core logic lives in `romm-api::core::library_scan` so TUI does not depend on `romm-cli`.
- Android (Gradle + UniFFI) is **not in this repo** — see [romm-rust-android](https://github.com/patricksmill/romm-rust-android).

## Consequences

### Positive

- `cargo test -p romm-api` validates API client without TUI deps.
- Android and future frontends depend on a small, publishable `romm-api` crate.
- CLI and TUI compile boundaries are enforced.

### Negative (accepted)

- Cross-crate path updates and workspace CI coordination.
- Library consumers may use `romm-api` directly or `romm-cli` re-exports during transition.

## Revisit

- UniFFI for Android: implement in `romm-rust-android/ffi/`; optional `uniffi` feature on `romm-api` if shared types are needed.
- Separate `romm-api` crates.io publish: optional; evaluate when Android ships.

## References

- [rust-guidelines.md — Gap 4](../rust-guidelines.md)
- [romm-rust-android](https://github.com/patricksmill/romm-rust-android)
- [CI workflow](../../.github/workflows/ci.yml)
