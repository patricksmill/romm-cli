# ADR: Workspace split

**Date:** 2026-06-06  
**Status:** Accepted — workspace split **completed** (trigger #3: third frontend)

## Maintainer decision

The project is a **Cargo workspace** with three members: `romm-api`, `romm-cli`, and `romm-tui`. The split was executed so additional frontends can depend on shared `RommClient` / core logic without pulling `clap` or `ratatui`.

## Trigger

**#3 — Third frontend:** A new frontend needs shared `RommClient` / core logic without pulling `clap` or `ratatui`.

## Context

Previously a single package hosted the library plus four binaries. TUI was feature-gated; CI validated `--no-default-features` builds. Compile times were acceptable, but a clean `romm-api` boundary is required for embedders that need a small, publishable library crate.

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

## Consequences

### Positive

- `cargo test -p romm-api` validates API client without TUI deps.
- Future frontends depend on a small, publishable `romm-api` crate.
- CLI and TUI compile boundaries are enforced.

### Negative (accepted)

- Cross-crate path updates and workspace CI coordination.
- Library consumers may use `romm-api` directly or `romm-cli` re-exports during transition.

## Revisit

- Optional `uniffi` feature on `romm-api` if a future FFI frontend needs shared types.

## References

- [rust-guidelines.md — Gap 4](../rust-guidelines.md)
- [CI workflow](../../.github/workflows/ci.yml)
