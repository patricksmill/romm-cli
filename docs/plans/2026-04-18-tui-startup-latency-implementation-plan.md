# TUI startup latency implementation plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Ship metadata-first TUI library entry with on-disk snapshot and background refresh so first paint is fast on repeat opens.

**Architecture:** `LibraryBrowseScreen` hydrates from `startup_library_snapshot::load_snapshot()` immediately; `App` spawns `fetch_merged_library_metadata` via `tokio::spawn` and applies results through a generation-guarded channel; ROM lists use existing `deferred_load_roms` + `RomCache`.

**Tech Stack:** Rust, `tokio` (spawn, `time::timeout`, `mpsc`), `serde_json`, existing `RommClient` / endpoints.

---

### Task 1: Snapshot persistence module

**Files:**

- Create: `src/core/startup_library_snapshot.rs`
- Modify: `src/core/mod.rs` (export module)
- Modify: `Cargo.toml` (enable `tokio` `time` for virtual-collection timeout)

**Steps:**

1. Add versioned JSON file (`version`, `saved_at_secs`, `platforms`, `collections`) with `load_snapshot`, `save_snapshot`, path helpers (`ROMM_LIBRARY_METADATA_SNAPSHOT_PATH`, test env `ROMM_TEST_LIBRARY_SNAPSHOT_DIR`).
2. Add `fetch_merged_library_metadata` mirroring prior synchronous main-menu logic (including 3s timeout on virtual collections).
3. Unit tests: round-trip save/load, corrupt file, wrong version.

**Verify:** `cargo test core::startup_library_snapshot`

---

### Task 2: Library screen metadata footer + list replacement

**Files:**

- Modify: `src/tui/screens/library_browse.rs`

**Steps:**

1. Add `metadata_footer: Option<String>`, `set_metadata_footer`, `replace_metadata` (reset indices, clear ROM pane).
2. Extend `render_help` to show footer text above key hints.

**Verify:** `cargo test tui::screens::library_browse`

---

### Task 3: App wiring — immediate library + background refresh

**Files:**

- Modify: `src/tui/app.rs`

**Steps:**

1. Add refresh generation + `UnboundedReceiver` for `LibraryMetadataRefreshDone`.
2. Implement `spawn_library_metadata_refresh`, `poll_library_metadata_refresh` (batch recv to satisfy borrow checker), `apply_library_metadata_refresh` (stale gen, total-failure guard, `save_snapshot`, footer, `deferred_load_roms`).
3. Replace main-menu Library branch: load snapshot → `LibraryBrowseScreen::new` → set footer (“Loading…” / “Refreshing…”) → queue deferred ROM load if non-empty → `spawn_library_metadata_refresh`.
4. Call `poll_background_tasks` at start of each `run` loop iteration; expose `pub fn poll_background_tasks` for tests/embedders.
5. Remove now-unused synchronous platform/collection fetch from that branch.

**Verify:** `cargo test --test tui_app`

---

### Task 4: Docs and env

**Files:**

- Create: `docs/plans/2026-04-18-tui-startup-latency-design.md`
- Modify: `README.md` (env table)
- Modify: `docs/tui.md` (event loop + snapshot section)

**Verify:** proofread paths and env names

---

### Task 5: Final verification

**Run:** `cargo test`  
**Expected:** all tests pass.

---

**Plan complete and saved to `docs/plans/2026-04-18-tui-startup-latency-implementation-plan.md`.**

**Execution options:**

1. **Subagent-Driven (this session)** — dispatch per task with review between tasks  
2. **Parallel Session (separate)** — new session with executing-plans and checkpoints  

*(Implementation above matches this plan; use the plan for review, bisect, or follow-up work.)*
