# Android prep + workspace split implementation plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Prepare for an Android browse-only client by splitting the monolith into `romm-api` / `romm-cli` / `romm-tui`, documenting the Android design, and updating ADR/Gap 4 to record Android as the third-frontend trigger.

**Architecture:** Kotlin/Compose UI (future) calls UniFFI-generated bindings (future) into `romm-api`. This phase delivers the workspace split only; UniFFI and the Gradle project are deferred.

**Tech Stack:** Rust workspace, existing `reqwest`/`tokio` stack; future: UniFFI, cargo-ndk, Jetpack Compose.

**Design doc:** [2026-06-06-android-frontend-design.md](./2026-06-06-android-frontend-design.md)

---

### Task 1: Workspace scaffold

**Files:**
- Create: `Cargo.toml` (workspace root)
- Create: `romm-api/Cargo.toml`, `romm-cli/Cargo.toml`, `romm-tui/Cargo.toml`
- Create: stub `lib.rs` in each crate

**Verify:** `cargo check` compiles empty workspace members.

---

### Task 2: Move `romm-api` modules

**Files:**
- Move: `src/{client,endpoints,core,update}/` → `romm-api/src/`
- Move: `src/{config,error,types,openapi,feature_compat,log_redact}.rs` → `romm-api/src/`
- Move: `tools/openapi_gen.rs` → `romm-api/src/bin/openapi_gen.rs`
- Create: `romm-api/src/lib.rs` with public module tree
- Move: `tests/openapi_registry.rs` → `romm-api/tests/`

**Verify:** `cargo test -p romm-api`

---

### Task 3: Extract `library_scan` core to `romm-api`

**Files:**
- Create: `romm-api/src/core/library_scan.rs` (types + `start_scan_library` + `wait_for_task_terminal`)
- Modify: `romm-cli/src/commands/library_scan.rs` — CLI-only presentation wrappers
- Modify: TUI imports → `romm_api::core::library_scan`

**Verify:** `cargo test -p romm-api`; TUI scan paths compile.

---

### Task 4: Move `romm-cli` crate

**Files:**
- Move: `src/commands/`, `src/cli_presentation.rs`, `src/frontend/cli.rs`, `src/main.rs`
- Move: `build.rs`, `completions/`, `tools/generate_completions.rs`
- Move: `tests/cli_*.rs`, `tests/release_check.rs` → `romm-cli/tests/`
- Create: `romm-cli/src/lib.rs` — commands + `pub use romm_api::…` for backward compat
- Add: optional `romm-tui` dependency with `tui` feature for `romm-cli tui` subcommand

**Verify:** `cargo test -p romm-cli --no-default-features`

---

### Task 5: Move `romm-tui` crate

**Files:**
- Move: `src/tui/` → `romm-tui/src/tui/`
- Move: `src/frontend/tui.rs`, `src/bin/romm_tui.rs`
- Move: `tests/tui_app.rs` → `romm-tui/tests/`
- Create: `romm-tui/src/lib.rs`

**Verify:** `cargo test -p romm-tui`

---

### Task 6: CI, release, and docs

**Files:**
- Modify: `.github/workflows/ci.yml` — workspace-aware jobs + `cargo test -p romm-api`
- Modify: `release-please-config.json`, `.release-please-manifest.json`
- Modify: `docs/plans/2026-06-06-workspace-split-adr.md` — Android trigger active
- Modify: `docs/rust-guidelines.md` Gap 4 — split complete
- Modify: `docs/architecture.md`, `README.md`

**Verify:** full pre-commit sequence from repo root.

---

### Task 7: Post-split verification checklist

- [x] `cargo test -p romm-api` passes without ratatui in dep graph
- [x] `cargo clippy --all-targets -p romm-cli --no-default-features -- -D warnings`
- [x] `cargo clippy --all-targets -p romm-tui -- -D warnings`
- [x] `cargo test --workspace --all-targets --all-features`
- [x] Release workflow `cargo build --workspace --release --bins` still produces `romm-cli` + `romm-tui`

---

## Next phase (out of scope here)

1. `SecretStore` + `AppPaths` traits in `romm-api`
2. UniFFI scaffold on `romm-api` (browse-only surface)
3. `android/` Gradle project + cargo-ndk CI job
4. Browse screens: setup → platforms → library → detail
