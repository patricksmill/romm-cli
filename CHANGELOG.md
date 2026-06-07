# Changelog index

This repository uses **independent per-crate changelogs** and component release tags (`romm-<crate>-v*`).

| Crate | Changelog | User guide | crates.io |
|-------|-----------|------------|-----------|
| **romm-api** | [romm-api/CHANGELOG.md](romm-api/CHANGELOG.md) | [docs/api.md](docs/api.md) | [crates.io/crates/romm-api](https://crates.io/crates/romm-api) |
| **romm-cli** | [romm-cli/CHANGELOG.md](romm-cli/CHANGELOG.md) | [docs/cli.md](docs/cli.md) | [crates.io/crates/romm-cli](https://crates.io/crates/romm-cli) |
| **romm-tui** | [romm-tui/CHANGELOG.md](romm-tui/CHANGELOG.md) | [docs/tui.md](docs/tui.md) | [crates.io/crates/romm-tui](https://crates.io/crates/romm-tui) |

**Scope rules:** Release Please writes entries to the changelog for the crate that owns the change (`feat(cli):` → `romm-cli`, `feat(tui):` → `romm-tui`, `feat(api):` / shared config & HTTP → `romm-api`). See [docs/releases.md — Changelog scopes](docs/releases.md#changelog-scopes).

Releases prior to the workspace split used a single unified history. Pre-1.0.0 entries in each crate changelog are filtered from that monolith by conventional-commit scope. The current supported line starts at **1.0.0** (post-split fresh start); `0.40.0` component tags were a bootstrap only.

Maintainers: see [docs/releases.md](docs/releases.md) for the full release runbook.
