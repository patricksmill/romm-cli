# Release coordination design

**Date:** 2026-06-06  
**Status:** Implemented — Approach A (Release Please + `cargo-workspace`)

## Summary

Coordinate independent semver releases for three workspace crates (`romm-api`, `romm-cli`, `romm-tui`), using component-prefixed GitHub tags, per-crate changelogs, and hardened CI publish guards.

## Industry benchmarks

| Project | Model | Takeaway |
|---------|-------|----------|
| [starship](https://github.com/starship/starship) | Unified `v*`, Release Please, binary matrix | Binary CI + checksum patterns |
| [uv](https://github.com/astral-sh/uv) | Lockstep workspace crate versions | Tight coupling avoids coordination overhead |
| [zellij](https://github.com/zellij-org/zellij) | Lockstep multi-crate, `cargo xtask publish` | TUI apps often ship one user version |
| [clap](https://github.com/clap-rs/clap) | Independent subcrate versions + exact pins | Model for workspace library releases |
| [ratatui](https://github.com/ratatui/ratatui) | `release-plz` + unified tags | Rust-native alternative (not chosen) |
| [ripgrep](https://github.com/BurntSushi/ripgrep) | Tag must match `Cargo.toml` | CI version consistency guard |

Romm is a hybrid: shared `romm-api` library plus two desktop frontends. Independent crate versions are valid but require explicit cascade rules and a compatibility matrix.

## Chosen approach

**Release Please + `cargo-workspace` plugin** (`merge: false`):

- Component tags: `romm-api-vA.B.C`, `romm-cli-vX.Y.Z`, `romm-tui-vP.Q.R`
- Per-crate changelogs under each crate directory
- Combined release PR bumps only components with qualifying commits
- crates.io publish: `romm-api` first; `romm-cli` and `romm-tui` depend on `romm-api` only

Rejected alternatives: full `release-plz` migration (tooling churn), `cargo-workspaces` manual releases (weak GitHub automation).

## Publish orchestration

crates.io publish runs in a single job chained off Release Please: `romm-api` first, then `romm-tui` and `romm-cli` in parallel (no cross-dependency between frontends). This avoids races when multiple component tags are created from one release PR merge. GitHub binary builds remain in `release-artifacts.yml` (per-tag, binaries only).

Shared scripts: `tools/publish-workspace.sh`, `tools/publish-crate.sh`, `tools/wait-for-crates-io.sh`. Diverged version combinations are tracked in `docs/compatibility.toml` and validated by `tools/release-check.sh`.

## Version ownership

| Crate | Tag | Changelog | GitHub binaries | crates.io |
|-------|-----|-----------|-----------------|-----------|
| `romm-api` | `romm-api-v*` | `romm-api/CHANGELOG.md` | None | Yes |
| `romm-cli` | `romm-cli-v*` | `romm-cli/CHANGELOG.md` | Single-binary archives | Yes |
| `romm-tui` | `romm-tui-v*` | `romm-tui/CHANGELOG.md` | Single-binary archives | Yes |

Root `CHANGELOG.md` is an index linking to per-crate changelogs.

## Commit scopes

| Scope / path | Bumps |
|--------------|-------|
| `feat(api):` / `romm-api/` | `romm-api` |
| `feat(cli):` / `romm-cli/` | `romm-cli` |
| `feat(tui):` / `romm-tui/` | `romm-tui` |
| Breaking `romm-api` change | `romm-api` + verify frontend releases same window |

## Binary distribution

- `romm-cli-v*`: archives contain `romm-cli` only
- `romm-tui-v*`: archives contain `romm-tui` only
- Self-update resolves component from running binary name; version from frontend crate (`env!("CARGO_PKG_VERSION")`); each archive updates one binary

## Migration

Bootstrap component tags at `0.40.0` from the last unified release SHA. New releases use component tags only (`romm-<crate>-v*`).

## References

- [docs/releases.md](../releases.md) — maintainer runbook
- [release-please manifest](../../.release-please-manifest.json)
- [Workspace split ADR](./2026-06-06-workspace-split-adr.md)
