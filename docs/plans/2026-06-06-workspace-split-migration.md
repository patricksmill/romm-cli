# Workspace split migration playbook

**Status:** **Completed** (2026-06-06) — Android frontend prep fired [ADR trigger #3](./2026-06-06-workspace-split-adr.md).  
**Date:** 2026-06-06

## Target layout

```text
romm-cli/                    # workspace root (Cargo.toml workspace.members)
├── romm-api/                # lib: client, endpoints, core, config, error, types, openapi, feature_compat, update
│   └── bins: romm-openapi-gen
├── romm-cli/                # lib: commands, frontend/cli; bin: romm-cli; build.rs + completions
└── romm-tui/                # lib: tui, frontend/tui; bin: romm-tui
```

## Dependency rules

```text
romm-cli ──depends on──> romm-api
romm-tui ──depends on──> romm-api
romm-cli -X- romm-tui     (no direct dependency)
```

| Crate | Allowed deps | Forbidden deps |
|-------|--------------|----------------|
| `romm-api` | `reqwest`, `serde`, `tokio`, `thiserror`, `keyring`, `tracing`, `self_update`, … | `clap`, `ratatui`, `crossterm`, `dialoguer`, `indicatif` |
| `romm-cli` | `romm-api`, `clap`, `dialoguer`, `indicatif`, … | `ratatui`, `crossterm` |
| `romm-tui` | `romm-api`, `ratatui`, `crossterm`, … | `clap` (optional: thin args only) |

### Module placement

| Current path | Target crate |
|--------------|--------------|
| `src/client/` | `romm-api` |
| `src/endpoints/` | `romm-api` |
| `src/core/` | `romm-api` |
| `src/config.rs` | `romm-api` |
| `src/error.rs` | `romm-api` |
| `src/types/` | `romm-api` |
| `src/openapi/` | `romm-api` |
| `src/feature_compat.rs` | `romm-api` |
| `src/update/` | `romm-api` (both frontends use it) |
| `src/commands/` | `romm-cli` |
| `src/frontend/cli.rs` | `romm-cli` |
| `src/main.rs` | `romm-cli` bin |
| `src/tui/` | `romm-tui` |
| `src/frontend/tui.rs` | `romm-tui` |
| `src/bin/romm_tui.rs` | `romm-tui` bin |
| `tools/openapi_gen.rs` | `romm-api` bin |
| `tools/generate_completions.rs` | `romm-cli` bin |
| `build.rs`, `completions/` | `romm-cli` |

### Known risks

- `commands/download.rs` uses `dialoguer` for interactive extras prompts — stays in `romm-cli`, not `romm-api`.
- `~40` `use crate::tui` imports are isolated under `src/tui/` today; no `core/` → `tui` coupling exists.
- `frontend/mod.rs` splits into `romm-cli` and `romm-tui` entry modules.

## Migration order

1. **Scaffold workspace** — Root `Cargo.toml` with `workspace.members = ["romm-api", "romm-cli", "romm-tui"]`; empty crate stubs that compile.
2. **Move `romm-api`** — Relocate shared modules; fix `use romm_api::…` paths; `cargo test -p romm-api`.
3. **Move `romm-cli`** — Depend on `romm-api`; relocate `commands/`, `main.rs`, `build.rs`; update integration tests to use workspace paths.
4. **Move `romm-tui`** — Depend on `romm-api`; relocate `tui/`, `romm_tui.rs`; `cargo test -p romm-tui`.
5. **Tool binaries** — `romm-openapi-gen` in `romm-api`; `romm-complete-gen` in `romm-cli`.
6. **CI updates** — Replace single-crate jobs with per-crate matrix:
   - `cargo test -p romm-api`
   - `cargo test -p romm-cli` (no TUI features needed)
   - `cargo test -p romm-tui`
   - Keep cross-target `cargo build --release` for release artifacts.
7. **Publish strategy** — Publish `romm-api` + `romm-cli` to crates.io; `romm-tui` optional separate crate or bundled in releases only.
8. **Docs** — Update README install paths, `rust-guidelines.md` Gap 4 second checkbox, and `docs.rs` crate split.

## Verification checklist (post-split)

- [x] `cargo test -p romm-api` passes without `ratatui` in the dependency graph
- [x] `cargo clippy --all-targets -p romm-cli` passes
- [x] `cargo clippy --all-targets -p romm-tui` passes
- [x] `romm-cli/tests/cli_*.rs` integration tests still pass via `romm-cli` binary
- [x] `cargo publish --dry-run -p romm-api` succeeds (`romm-cli` publish follows first `romm-api` crates.io release)
- [x] Release workflow produces `romm-cli` + `romm-tui` binaries from workspace

## Rollback

Keep the pre-split tag. If migration stalls, revert the workspace commit and continue using the `tui` feature flag until triggers are re-evaluated.
