# ADR: Workspace split deferred

**Date:** 2026-06-06  
**Status:** Accepted — split deferred

## Context

`romm-cli` is a single Cargo package with four binaries (`romm-cli`, `romm-tui`, `romm-openapi-gen`, `romm-complete-gen`) sharing one library (`src/lib.rs`). The TUI is feature-gated (`default = ["tui"]`); CI already validates `--no-default-features` builds and tests on Linux, Windows, and macOS.

The [rust-guidelines](../rust-guidelines.md) target layout splits the project into `romm-api`, `romm-cli`, and `romm-tui` workspace members. At current size (~40 TUI modules, one external consumer via crates.io), compile times and dependency boundaries do not yet justify the migration cost.

## Decision

**Remain a single crate** until at least one trigger below is met. Continue feature-gating TUI dependencies and reserving the `romm-api` crate name in documentation and migration plans.

## Triggers (any one)

1. **External consumer** — Another crate needs `RommClient` / core logic without pulling `clap`, `ratatui`, or `dialoguer`.
2. **Compile-time pain** — `cargo test --no-default-features` or incremental rebuilds routinely exceed ~3 minutes on a typical dev machine (measure baseline before splitting).
3. **Third frontend** — A new binary (e.g. GUI, language bindings) needs shared API client code with a clean dependency boundary.
4. **Separate crates.io publish** — `romm-api` must be published independently from the `romm-cli` binary crate.

## Consequences

### Positive (staying monolithic)

- No cross-crate versioning or workspace publish coordination.
- Simpler contributor onboarding (one `cargo test`).
- Feature flag already isolates TUI from headless CI.

### Negative (accepted)

- Library consumers depend on the full `romm-cli` crate graph unless they use `--no-default-features`.
- TUI and CLI share one release version even when only one frontend changes.

## Revisit

- Quarterly during maintenance, or immediately when a trigger fires.
- Migration steps: [2026-06-06-workspace-split-migration.md](./2026-06-06-workspace-split-migration.md).

## References

- [rust-guidelines.md — Gap 4](../rust-guidelines.md)
- [CI workflow](../../.github/workflows/ci.yml) (`test-cli-only`, `clippy-cli-only`)
