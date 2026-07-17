# ROM partial / cache cleanup — implementation plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** One shared “fetch complete” predicate, symmetric complete-only disk writes, intentional Failed/partial policy, and delete the thin wrappers/dead branch from the partial-resume path.

**Architecture:** Keep the three-tier model (disk complete-only / memory partials / gen+selection). Put `rom_list_fetch_complete` next to `ROM_PAGE_CEILING` in `romm-api`, use it everywhere, gate prefetch inserts the same way as primary loads, document Failed = keep partial for retry-from-progress, then shrink TUI helpers.

**Tech Stack:** Rust workspace (`romm-api`, `romm-tui`), existing `cargo test` / clippy / fmt gates.

---

## Decisions (locked)

| Topic | Choice | Why |
| --- | --- | --- |
| Shared predicate location | `romm_api::core::roms::rom_list_fetch_complete` | Sits next to `ROM_PAGE_CEILING`; both call sites already depend on that module |
| Prefetch disk write | Insert only when complete | Matches primary path; incomplete prefetch must not poison disk |
| Failed + partials | **Keep** partial on `RomLoadEvent::Failed` | Last successful `Batch` already stashed progress; retry should resume, not restart empty. `expected` mismatch still evicts via `matching_rom_partial` |
| Thin helpers | Delete `rom_partial_resume_offset`, `stash_rom_partial`, `clear_rom_partial` | Call sites use `HashMap` / `items.len()` directly |

---

## Task 0: Branch

```bash
git checkout -b fix/rom-partial-cache-cleanup
```

Baseline (must pass before commits later):

```bash
cargo fmt --check
cargo clippy --all-targets --all-features -- -D warnings
cargo clippy --all-targets --no-default-features -- -D warnings
```

---

## Task 1: Shared `rom_list_fetch_complete` in `romm-api`

**Files:**
- Modify: `romm-api/src/core/roms.rs`
- Modify: `romm-api/src/core/cache.rs`
- Test: `romm-api/src/core/roms.rs` (add unit tests) or extend `romm-api/src/core/cache.rs` tests

**Step 1: Write failing tests in `roms.rs`**

Add a `#[cfg(test)] mod tests` block:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::RomList;

    fn list(items: usize, total: u64) -> RomList {
        RomList {
            items: (0..items)
                .map(|i| crate::types::Rom {
                    id: i as u64,
                    name: format!("r{i}"),
                    ..Default::default() // adjust if Rom has no Default — use minimal fixture from cache tests
                })
                .collect(),
            total,
            limit: 50,
            offset: 0,
        }
    }

    #[test]
    fn complete_when_items_cover_total() {
        assert!(rom_list_fetch_complete(&list(3, 3)));
    }

    #[test]
    fn incomplete_when_short_of_total() {
        assert!(!rom_list_fetch_complete(&list(1, 100)));
    }

    #[test]
    fn complete_at_page_ceiling_even_if_total_higher() {
        // Build list with items.len() == ROM_PAGE_CEILING without allocating 20k Roms if heavy:
        // either construct via unsafe-free stub or test the predicate with a hand-built RomList
        // whose items vec is resized — prefer a tiny helper that only sets len via a test-only
        // path. Simplest: assert the boolean formula with a RomList that has
        // items.len() >= ROM_PAGE_CEILING using vec![minimal_rom; ROM_PAGE_CEILING as usize]
        // ONLY if CI time is fine; otherwise test `loaded >= total` paths only and rely on
        // cache test for ceiling via a comment that ceiling branch is covered by cache insert path.
        let mut big = list(0, ROM_PAGE_CEILING + 1);
        big.items = vec![minimal_rom(); ROM_PAGE_CEILING as usize];
        assert!(rom_list_fetch_complete(&big));
    }
}
```

**Practical fixture note:** Reuse whatever minimal `Rom` construction `romm-api/src/core/cache.rs` tests already use (`sample_rom_list` / similar). Do not invent a second fixture style. If allocating `ROM_PAGE_CEILING` ROMs is too heavy, skip the ceiling test and cover ceiling only via a unit test that builds `RomList { items: Vec::with_capacity(...); /* set len with resize_with */ }` — or keep the existing cache incomplete test and add only total/items cases here.

**Step 2: Run test — expect fail (function missing / private)**

```bash
cargo test -p romm-api rom_list_fetch_complete -- --nocapture
```

Expected: compile error or FAIL.

**Step 3: Implement in `romm-api/src/core/roms.rs`**

```rust
/// True when a paginated ROM list has enough rows to stop fetching
/// (`items.len() >= total`, or the safety ceiling was hit).
pub fn rom_list_fetch_complete(list: &RomList) -> bool {
    let loaded = list.items.len() as u64;
    loaded >= list.total || loaded >= ROM_PAGE_CEILING
}
```

Optionally refactor `fetch_roms_paginated` loop condition to use it for the “need another page?” check:

```rust
while !rom_list_fetch_complete(&roms) {
    // ...
    if next_batch.items.is_empty() {
        break;
    }
    roms.items.extend(next_batch.items);
}
```

(Keep the empty-batch break — complete predicate alone does not cover empty page.)

**Step 4: Wire `cache.rs` to the shared fn**

Delete private `fn rom_list_fetch_complete` in `romm-api/src/core/cache.rs`. In `get_valid`:

```rust
.filter(|(stored_count, list)| {
    *stored_count == expected_count && crate::core::roms::rom_list_fetch_complete(list)
})
```

Keep existing `get_valid_rejects_incomplete_paginated_list` test — it should still pass.

**Step 5: Run tests**

```bash
cargo test -p romm-api
```

Expected: PASS.

**Step 6: Commit**

```bash
git add romm-api/src/core/roms.rs romm-api/src/core/cache.rs
git commit -m "$(cat <<'EOF'
refactor(api): share rom_list_fetch_complete next to page ceiling

EOF
)"
```

---

## Task 2: TUI uses shared predicate; delete duplicate + thin helpers

**Files:**
- Modify: `romm-tui/src/tui/app/rom_load.rs`
- Modify: `romm-tui/src/tui/app/background/update.rs`
- Modify: `romm-tui/src/tui/app/background/tasks.rs`
- Modify: `romm-tui/src/tui/app/update.rs`
- Modify: `romm-tui/src/tui/app/tests.rs`

**Step 1: Update imports / call sites before deleting**

In `rom_load.rs`:
- Remove local `rom_list_fetch_complete` and `rom_partial_resume_offset`.
- Import `romm_api::core::roms::rom_list_fetch_complete`.
- Delete `stash_rom_partial` and `clear_rom_partial` methods.
- Keep `matching_rom_partial` (real logic); call `rom_list_fetch_complete` from `romm_api`.

**Step 2: Fix callers**

`background/update.rs` Batch arm:

```rust
RomLoadEvent::Batch(roms) => {
    // ponytail: disk cache stays complete-only; in-memory partials resume mid-fetch.
    let fetch_complete = romm_api::core::roms::rom_list_fetch_complete(&roms);
    if let Some(ref k) = done.key {
        if fetch_complete {
            self.rom_cache
                .insert(k.clone(), roms.clone(), done.expected);
            self.rom_partials.remove(k);
        } else {
            self.rom_partials
                .insert(k.clone(), (done.expected, roms.clone()));
        }
    }
    // ... set_roms unchanged
}
```

`RomLoadEvent::Complete`:

```rust
RomLoadEvent::Complete => {
    if let Some(ref k) = done.key {
        self.rom_partials.remove(k);
    }
    // ... set_rom_loading(false)
}
```

`RomLoadEvent::Failed` — **keep partial**, document why:

```rust
RomLoadEvent::Failed(e) => {
    // Keep rom_partials for this key: last Batch already stashed progress;
    // next open resumes offset instead of restarting empty after a transient error.
    if let AppScreen::LibraryBrowse(ref mut lib) = self.screen {
        lib.set_metadata_footer(Some(format!(
            "Could not load games: {}",
            user_message(&e)
        )));
        lib.set_rom_loading(false);
    }
}
```

`background/tasks.rs` — replace `clear_rom_partial` with `self.rom_partials.remove(...)`.

`update.rs` deferred resume — delete dead `else`, set offset directly:

```rust
if let Some(ref k) = key {
    if let Some(partial) = self.matching_rom_partial(k, expected) {
        if let AppScreen::LibraryBrowse(ref mut lib) = self.screen {
            if super::rom_load::primary_rom_load_result_matches_selection(lib, &key) {
                lib.set_roms(partial.clone());
                tracing::debug!(
                    "rom-list-render context={} latency_ms={} (partial_resume items={})",
                    context,
                    started.elapsed().as_millis(),
                    partial.items.len()
                );
            }
        }
        // matching_rom_partial only returns incomplete lists
        if let Some(ref mut r) = req {
            r.offset = Some(partial.items.len() as u32);
        }
        aggregated = Some(partial);
    }
}
```

**Step 3: Fix tests**

In `romm-tui/src/tui/app/tests.rs` `deferred_rom_load_seeds_ui_from_partial_and_resumes_offset`:
- Replace `rom_partial_resume_offset(&partial)` assert with:

```rust
assert_eq!(partial.items.len(), 2);
assert!(!romm_api::core::roms::rom_list_fetch_complete(&partial));
```

(Offset behavior is covered when `process_deferred_rom_load_for_test` runs — optionally assert the spawned request offset by inspecting `req` before spawn if the test already can; otherwise keep UI item-count assert as today.)

Existing partial-batch / complete-batch tests should still pass (they touch `rom_partials` / `rom_cache` maps directly).

**Step 4: Run tests**

```bash
cargo test -p romm-tui primary_rom_load_partial -- --nocapture
cargo test -p romm-tui deferred_rom_load_seeds -- --nocapture
cargo test -p romm-tui primary_rom_load_complete -- --nocapture
cargo test -p romm-api
```

Expected: PASS.

**Step 5: Commit**

```bash
git add romm-tui/src/tui/app/rom_load.rs \
  romm-tui/src/tui/app/background/update.rs \
  romm-tui/src/tui/app/background/tasks.rs \
  romm-tui/src/tui/app/update.rs \
  romm-tui/src/tui/app/tests.rs
git commit -m "$(cat <<'EOF'
refactor(tui): use shared fetch-complete and drop partial helpers

EOF
)"
```

---

## Task 3: Prefetch inserts only complete lists

**Files:**
- Modify: `romm-tui/src/tui/app/background/update.rs` (`apply_collection_prefetch_complete`)
- Test: `romm-tui/src/tui/app/tests.rs` (add one smoke test if a prefetch-complete harness already exists; otherwise unit-test via applying a `CollectionPrefetchDone` if tests can construct `App`)

**Step 1: Prefer a failing test**

If `app_with_library` + `poll_background_tasks` / `apply_background` can inject `CollectionPrefetchDone`:

```rust
#[test]
fn collection_prefetch_incomplete_list_is_not_disk_cached() {
    let mut app = app_with_library(vec![platform(1, "NES", 100)]);
    app.apply_background(BackgroundAction::CollectionPrefetch(CollectionPrefetchDone {
        key: RomCacheKey::Platform(1),
        expected: 100,
        roms: Some(RomList {
            total: 100,
            limit: 50,
            offset: 0,
            items: vec![rom_fixture()],
        }),
        warning: None,
    }));
    assert!(
        app.rom_cache
            .get_valid(&RomCacheKey::Platform(1), 100)
            .is_none()
    );
}
```

(Adjust type paths to match `background/types.rs`. If `apply_background` is `pub(in crate::tui::app)` and tests are in the same module tree, this works like existing RomLoad tests.)

If wiring a prefetch test is awkward, skip the new test and rely on the one-line gate + existing `get_valid` incomplete rejection — still implement the gate.

**Step 2: Implement gate**

```rust
fn apply_collection_prefetch_complete(&mut self, done: super::types::CollectionPrefetchDone) {
    self.collection_prefetch_inflight_keys.remove(&done.key);
    if let Some(roms) = done.roms {
        if romm_api::core::roms::rom_list_fetch_complete(&roms) {
            self.rom_cache.insert(done.key, roms, done.expected);
        } else {
            tracing::debug!(
                "collection prefetch incomplete; not writing disk cache key={:?}",
                done.key
            );
        }
    } else if let Some(warning) = done.warning {
        tracing::debug!("{warning}");
    }
}
```

Do **not** stash prefetch results into `rom_partials` (YAGNI — prefetch is best-effort full fetch via `fetch_roms_paginated`).

**Step 3: Run tests**

```bash
cargo test -p romm-tui
```

Expected: PASS.

**Step 4: Commit**

```bash
git add romm-tui/src/tui/app/background/update.rs romm-tui/src/tui/app/tests.rs
git commit -m "$(cat <<'EOF'
fix(tui): skip disk cache insert for incomplete collection prefetches

EOF
)"
```

---

## Task 4: Fmt / clippy / full verify

**Step 1: Run required gates (repo rule)**

```bash
cargo fmt
cargo fmt --check
cargo clippy --all-targets --all-features -- -D warnings
cargo clippy --all-targets --no-default-features -- -D warnings
cargo test
```

All must exit 0. Fix any fallout (unused imports after deleting helpers).

**Step 2: Commit only if fmt/clippy touched files**

```bash
git add -u
git commit -m "$(cat <<'EOF'
chore: fmt/clippy after rom partial cache cleanup

EOF
)"
```

(Skip empty commit if clean.)

---

## Out of scope

- Persisting partials to disk
- Merging `rom_partials` into `RomCache`
- Changing gen/selection stale-drop behavior
- Clearing partials on `Failed` (explicitly rejected — see Decisions)

---

## Done when

- [ ] One `rom_list_fetch_complete` definition in `romm-api`
- [ ] Primary + prefetch disk writes only when complete
- [ ] Failed arm documents keep-partial policy
- [ ] No `rom_partial_resume_offset` / `stash_rom_partial` / `clear_rom_partial`
- [ ] Dead resume `else` gone
- [ ] fmt + both clippy feature sets + tests green
