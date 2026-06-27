# Achievements in game detail (v1) implementation plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Show RetroAchievements catalog + user earned state in romm-tui game detail as a text list with completion summary.

**Architecture:** Add `DetailedRom` and achievement merge helpers in `romm-api`. On game detail open, background worker fetches `GET /api/roms/{id}` and `GET /api/users/me`, merges progression by `ra_id`, and updates `AchievementListState` on `GameDetailScreen` (same pattern as saves). Feature-compat gate for old RomM servers.

**Tech stack:** Rust 2021, `serde`/`serde_json`, `tokio`, `ratatui`, existing `RommClient` + OpenAPI registry.

**Decisions (locked):**

| # | Choice |
|---|--------|
| v1 surface | TUI game detail only (user confirmed) |
| Display | Text list + `earned/total (pct%)` header; first 8 rows like saves |
| Images | No badge rendering |
| Settings | No RA username/sync in TUI v1 — user syncs via RomM web |
| CLI | Out of scope |
| Min RomM | 4.9+ (RA fields in OpenAPI); gate via feature compat |
| Transport | RomM REST only — no direct RetroAchievements API |

**Out of scope v1:** earned-only filter, achievement selection keys, `ra_hash` UI, CLI command, refresh endpoint, badge URLs.

**Reference:** [2026-06-27-achievements-game-detail-design.md](2026-06-27-achievements-game-detail-design.md)

---

## Task 0: Branch

**Step 1:** Create branch from clean `main` (save-sync work stays in stash).

```bash
git checkout -b feat/achievements-game-detail
```

**Step 2:** Baseline checks:

```bash
cargo fmt --check
cargo clippy --all-targets --all-features -- -D warnings
cargo clippy --all-targets --no-default-features -- -D warnings
cargo test
```

Expected: all green.

---

## Task 1: Achievement types and merge logic (`romm-api`)

**Files:**
- Create: `romm-api/src/types/achievements.rs`
- Create: `romm-api/src/core/achievements.rs`
- Modify: `romm-api/src/types.rs` (mod + re-exports)
- Modify: `romm-api/src/core/mod.rs`
- Test: `romm-api/src/core/achievements.rs` (`#[cfg(test)]` at bottom)

**Step 1: Write failing merge test**

Add to `romm-api/src/core/achievements.rs`:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn merge_marks_earned_by_badge_id() {
        let catalog = json!({
            "achievements": [
                {"ra_id": 1, "title": "First steps", "points": 5, "badge_id": "85541", "display_order": 0},
                {"ra_id": 2, "title": "Speed run", "points": 10, "badge_id": "85542", "display_order": 1}
            ]
        });
        let progression = json!({
            "results": [{
                "rom_ra_id": 14402,
                "num_awarded": 1,
                "max_possible": 2,
                "earned_achievements": [{"id": "85541", "date": "2022-08-23 22:56:38"}]
            }]
        });
        let rows = merge_achievements(14402, &catalog, &progression).unwrap();
        assert_eq!(rows.len(), 2);
        assert!(rows[0].earned);
        assert!(!rows[1].earned);
        assert_eq!(summary(&rows), (1, 2));
    }
}
```

**Step 2: Run test — expect fail**

```bash
cargo test -p romm-api merge_marks_earned -- --nocapture
```

Expected: FAIL (module/function missing).

**Step 3: Implement types + merge**

`types/achievements.rs` — deserialize shapes from RomM:

```rust
#[derive(Debug, Clone, Deserialize, Default)]
pub struct MergedRaMetadata {
    #[serde(default)]
    pub achievements: Vec<RaAchievement>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RaAchievement {
    pub title: String,
    #[serde(default)]
    pub points: Option<i64>,
    #[serde(default)]
    pub badge_id: Option<String>,
    #[serde(default, alias = "badge_name")]
    pub badge_name: Option<String>,
    #[serde(default)]
    pub display_order: Option<i64>,
}

#[derive(Debug, Clone, Deserialize, Default)]
pub struct RaUserProgression {
    #[serde(default)]
    pub results: Vec<RaUserGameProgression>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RaUserGameProgression {
    pub rom_ra_id: Option<i64>,
    #[serde(default)]
    pub num_awarded: Option<i64>,
    #[serde(default)]
    pub max_possible: Option<i64>,
    #[serde(default)]
    pub earned_achievements: Vec<EarnedAchievement>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct EarnedAchievement {
    pub id: String,
    #[serde(default)]
    pub date: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AchievementRow {
    pub title: String,
    pub points: Option<i64>,
    pub earned: bool,
    pub earned_at: Option<String>,
}
```

`core/achievements.rs`:

```rust
pub fn merge_achievements(
    ra_id: i64,
    metadata: &MergedRaMetadata,
    progression: &RaUserProgression,
) -> Vec<AchievementRow> { /* join by badge id, sort display_order */ }

pub fn summary(rows: &[AchievementRow]) -> (usize, usize) {
    let earned = rows.iter().filter(|r| r.earned).count();
    (earned, rows.len())
}

pub fn empty_reason(
    ra_id: Option<i64>,
    ra_username: Option<&str>,
    rows: &[AchievementRow],
) -> Option<&'static str> { /* None = show list; Some = message */ }
```

Use badge id from `badge_id.or(badge_name)` when matching `EarnedAchievement.id`.

**Step 4: Run tests**

```bash
cargo test -p romm-api achievements -- --nocapture
```

Expected: PASS.

**Step 5: Commit**

```bash
git add romm-api/src/types/achievements.rs romm-api/src/core/achievements.rs romm-api/src/types.rs romm-api/src/core/mod.rs
git commit -m "feat(romm-api): add RetroAchievements merge helpers"
```

---

## Task 2: `DetailedRom` for `GET /api/roms/{id}`

**Files:**
- Create: `romm-api/src/types/detailed_rom.rs`
- Modify: `romm-api/src/types.rs`
- Modify: `romm-api/src/endpoints/roms.rs` (`GetRom::Output`)
- Modify: call sites that need `.into()` or field access (grep `GetRom`)

**Step 1: Add `DetailedRom`**

Extend list `Rom` fields and add:

```rust
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct DetailedRom {
    // copy all fields from Rom today, OR:
    #[serde(flatten)]
    pub base: Rom, // if Rom stays list shape — pick one approach and stay consistent

    #[serde(default)]
    pub ra_id: Option<i64>,
    #[serde(default)]
    pub merged_ra_metadata: Option<MergedRaMetadata>,
}
```

Prefer **adding optional fields to existing `Rom`** with `#[serde(default)]` if that avoids duplicate field lists — ponytail: fewer structs if list endpoint ignores extra JSON.

**Step 2: Change `GetRom::Output`**

```rust
impl Endpoint for GetRom {
    type Output = crate::types::Rom; // now includes ra_id + merged_ra_metadata
}
```

**Step 3: Fix compile errors**

Run:

```bash
cargo build -p romm-api -p romm-cli -p romm-tui
```

Update any struct literals in tests to include new optional fields (or use `..Default` if added).

**Step 4: Commit**

```bash
git commit -am "feat(romm-api): deserialize RA fields on Rom detail"
```

---

## Task 3: Parse `GET /api/users/me` progression

**Files:**
- Modify: `romm-api/src/endpoints/system.rs` — add typed output or helper
- Create: `romm-api/src/types/user.rs` (minimal: `ra_username`, `ra_progression`)
- Modify: `romm-api/src/types.rs`

**Step 1: Add `CurrentUser` type**

```rust
#[derive(Debug, Clone, Deserialize, Default)]
pub struct CurrentUser {
    pub id: u64,
    #[serde(default)]
    pub ra_username: Option<String>,
    #[serde(default)]
    pub ra_progression: Option<RaUserProgression>,
}
```

**Step 2: Change `GetUsersMe::Output` to `CurrentUser`** (or add `GetUsersMe::parse` helper if changing Output is too wide).

**Step 3: Unit test deserialize fixture**

```rust
#[test]
fn current_user_deserializes_ra_progression() {
    let json = r#"{"id":1,"ra_username":"player1","ra_progression":{"total":1,"results":[]}}"#;
    let u: CurrentUser = serde_json::from_str(json).unwrap();
    assert_eq!(u.ra_username.as_deref(), Some("player1"));
}
```

**Step 4: Commit**

```bash
git commit -am "feat(romm-api): typed CurrentUser with ra_progression"
```

---

## Task 4: Feature compatibility gate

**Files:**
- Modify: `romm-api/src/feature_compat.rs`
- Modify: `romm-tui/src/tui/app/mod.rs` (store compat, pass to `App::new`)
- Modify: `romm-tui/src/tui/openapi_sync.rs` if compat tuple grows
- Test: `romm-api/src/feature_compat.rs`

**Step 1: Add constants**

```rust
pub const ACHIEVEMENTS_FEATURE: &str = "achievements";
pub const ACHIEVEMENTS_UNSUPPORTED_MESSAGE: &str =
    "This RomM server does not expose achievement fields; upgrade RomM to 4.9+.";
pub const ACHIEVEMENTS_REQUIRED_ENDPOINTS: [RequiredEndpoint; 2] = [
    RequiredEndpoint { method: "GET", path: "/api/roms/{id}" },
    RequiredEndpoint { method: "GET", path: "/api/users/me" },
];
```

**Step 2: Wire TUI `App`**

Add `achievements_compat: AchievementsCompatibility` alongside save-sync / metadata-edit. Tests use `supported_achievements_compatibility()`.

**Step 3: Commit**

```bash
git commit -am "feat(romm-api): achievements feature compatibility gate"
```

---

## Task 5: TUI background loader

**Files:**
- Modify: `romm-tui/src/tui/app/background/types.rs` — `AchievementLoadDone`
- Modify: `romm-tui/src/tui/app/background/tasks.rs` — `spawn_achievement_load_worker`, `refresh_current_game_achievements`
- Modify: `romm-tui/src/tui/app/background/update.rs` — apply completion
- Modify: `romm-tui/src/tui/app/mod.rs` — channel + poll
- Modify: `romm-tui/src/tui/app/event.rs` if needed

**Step 1: Add completion type**

```rust
pub(crate) struct AchievementLoadDone {
    pub(crate) rom_id: u64,
    pub(crate) result: Result<AchievementLoadResult, RommError>,
}

pub(crate) struct AchievementLoadResult {
    pub(crate) rows: Vec<AchievementRow>,
    pub(crate) empty_message: Option<String>,
}
```

**Step 2: Implement worker** (mirror `spawn_save_list_worker`)

```rust
pub(in crate::tui::app) fn spawn_achievement_load_worker(&mut self, rom_id: u64) {
    if !self.achievements_compat.supported {
        // set Failed/Unsupported on detail screen
        return;
    }
    detail.set_achievements_loading();
    tokio::spawn(async move {
        let rom = client.call(&GetRom { id: rom_id }).await?;
        let user = client.call(&GetUsersMe).await?;
        let rows = merge or empty_reason...
        tx.send(AchievementLoadDone { rom_id, result: Ok(...) })
    });
}
```

**Step 3: Hook open detail**

In `handlers/library.rs` and `handlers/search.rs`, after `refresh_current_game_saves()`:

```rust
self.refresh_current_game_achievements();
```

**Step 4: Manual smoke**

Run TUI against RomM with a matched RA game; confirm worker fires (log or loading state).

**Step 5: Commit**

```bash
git commit -am "feat(romm-tui): background achievement load on game detail"
```

---

## Task 6: Game detail UI

**Files:**
- Modify: `romm-tui/src/tui/screens/game_detail/types.rs` — `AchievementListState`
- Create: `romm-tui/src/tui/screens/game_detail/achievements.rs`
- Modify: `romm-tui/src/tui/screens/game_detail/mod.rs`
- Modify: `romm-tui/src/tui/screens/game_detail/state.rs` — setters
- Modify: `romm-tui/src/tui/screens/game_detail/render.rs` — section after Saves
- Modify: `romm-tui/src/tui/screens/game_detail/tests.rs`
- Modify: `docs/tui.md` — one line under game detail features

**Step 1: State enum**

```rust
pub enum AchievementListState {
    Idle,
    Loading,
    Loaded { rows: Vec<AchievementRow>, summary: (usize, usize) },
    Empty(String),
    Failed(String),
    Unsupported(String),
}
```

**Step 2: `achievement_lines` test first**

```rust
#[test]
fn achievement_lines_shows_earned_marker() {
    let state = AchievementListState::Loaded {
        rows: vec![AchievementRow { title: "A".into(), points: Some(5), earned: true, earned_at: None }],
        summary: (1, 1),
    };
    assert!(achievement_lines(&state)[1].to_string().contains('✓')); // or [x] / E
}
```

**Step 3: Render block**

After Saves in `render_metadata_panel`:

```rust
lines.push(Line::from(Span::styled("Achievements:", styles.label())));
lines.extend(achievement_lines(&self.achievements_state));
```

Format row: `  [✓] Title — 5 pts` / `  [ ] Title — 10 pts`  
Header when loaded: `  4/32 (12%)`

**Step 4: Run tests**

```bash
cargo test -p romm-tui game_detail
```

**Step 5: Commit**

```bash
git commit -am "feat(romm-tui): render achievements in game detail"
```

---

## Task 7: Integration test (optional but recommended)

**Files:**
- Modify: `romm-tui/tests/tui_app.rs`

**Step 1:** Add test that opens game detail with mocked HTTP returning `ra_id` + `merged_ra_metadata` + user progression; assert render path sets `AchievementListState::Loaded`.

Use existing `httpmock` patterns from `library_enter_opens_game_detail`.

**Step 2: Commit**

```bash
git commit -am "test(romm-tui): achievement section loads on game detail"
```

---

## Task 8: Docs and compatibility row

**Files:**
- Modify: `docs/tui.md` — game detail bullet + keyboard note (read-only, no new keys v1)
- Modify: `docs/compatibility.toml` — row when shipping (can defer until release)

**Step 1:** Document prerequisites (RA key, match, web sync).

**Step 2: Commit**

```bash
git commit -am "docs: achievements in game detail"
```

---

## Task 9: Final verification

```bash
cargo fmt
cargo fmt --check
cargo clippy --all-targets --all-features -- -D warnings
cargo clippy --all-targets --no-default-features -- -D warnings
cargo test
```

Manual checklist:

- [ ] Game without `ra_id` → “Not matched to RetroAchievements”
- [ ] Game with `ra_id`, no `ra_username` → profile hint
- [ ] Matched + synced user → list + percentage
- [ ] Old RomM / missing endpoints → compat message

---

## Execution handoff

Plan saved to `docs/plans/2026-06-27-achievements-game-detail.md`.

**Two execution options:**

1. **Subagent-Driven (this session)** — @superpowers:subagent-driven-development, one task at a time with review  
2. **Parallel session** — new session with @superpowers:executing-plans in a worktree (`../romm-cli-achievements`)

Which approach do you want?
