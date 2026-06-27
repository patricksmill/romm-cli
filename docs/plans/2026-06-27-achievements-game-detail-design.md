# Achievements in game detail — design

**Date:** 2026-06-27  
**Status:** Approved (scope locked with user)

## Goal

Show RetroAchievements progress in **romm-tui game detail** — same place users already open a ROM from library/search. Text-only list with completion summary; no new screen, no CLI, no Settings sync in v1.

## Context

- RomM web UI shows achievements under game detail **Personal** when RA is configured, ROM is matched (`ra_id`), and the user has synced `ra_progression`.
- romm-cli today has `has_ra` list filter and `ra_id` in metadata match, but **`Rom` drops RA fields** from `GET /api/roms/{id}` and game detail renders saves only.
- Bundled OpenAPI (RomM 4.9.2) includes `merged_ra_metadata`, `RAUserGameProgression`, `EarnedAchievement`, and `POST /api/users/{id}/ra/refresh`.

## Approaches considered

| Approach | Pros | Cons |
|----------|------|------|
| **A. TUI detail section (recommended)** | Mirrors saves worker pattern; minimal UI; reuses RomM as RA proxy | Needs `DetailedRom` + merge helper in `romm-api` |
| B. Call RetroAchievements API directly from client | Full control | Requires RA API key in client; duplicates RomM; rejected |
| C. Separate achievements screen / tab | More room for badges | New navigation, keys, state; YAGNI for v1 |

**Recommendation:** A — fetch `GET /api/roms/{id}` + `GET /api/users/me`, merge server-side data in `romm-api`, render a scroll-free text block (first N rows, like saves).

## Architecture

```
Game detail open
  → spawn_achievements_worker(rom_id)
      → GET /api/roms/{id}     → DetailedRom { ra_id, merged_ra_metadata }
      → GET /api/users/me      → { ra_username, ra_progression }
      → merge_achievements()   → Vec<AchievementRow>
  → AchievementListState on GameDetailScreen
  → achievement_lines() in render panel (below Saves)
```

**Data merge:** Match user progression by `rom_ra_id == rom.ra_id`. Join catalog achievements with `earned_achievements` by badge id (`EarnedAchievement.id` ↔ achievement `badge_id`). Sort by `display_order`.

**Empty states (explicit messages):**

| Condition | Message |
|-----------|---------|
| Feature compat unsupported | Same pattern as metadata-edit / save-sync |
| `ra_id` missing | Not matched to RetroAchievements |
| `ra_username` empty | Set RA username in RomM profile (web) |
| Catalog empty but `ra_id` set | No achievement metadata on server |
| Loaded | `4/32 (12%)` header + list |

## Components

### `romm-api`

- `types/achievements.rs` — `RaAchievement`, `RaUserProgression`, `AchievementRow`, `MergedRaMetadata`
- `types/detailed_rom.rs` — superset of `Rom` for `GET /api/roms/{id}` (or extend `Rom` with optional RA fields + `#[serde(default)]`)
- `core/achievements.rs` — `merge_achievements()`, `progression_for_ra_id()`, unit tests with JSON fixtures
- `endpoints/roms.rs` — change `GetRom::Output` to `DetailedRom` (still deserializes for callers that only need base fields)
- `feature_compat.rs` — `ACHIEVEMENTS_FEATURE` gate: `GET /api/roms/{id}`, `GET /api/users/me`

### `romm-tui`

- `game_detail/types.rs` — `AchievementListState` enum
- `game_detail/achievements.rs` — `achievement_lines()` (parallel to `saves.rs`)
- `game_detail/render.rs` — Achievements section after Saves
- `app/background/` — `AchievementLoadDone`, channel, worker, `apply_achievement_load_complete`
- `handlers/library.rs` + `handlers/search.rs` — call `refresh_current_game_achievements()` on detail open

## Out of scope v1

- Badge images in terminal
- `j`/`k` selection or earned-only toggle
- Settings RA sync / username edit
- `romm-cli` subcommand
- `ra_hash` verification badge
- `POST /users/{id}/ra/refresh`

## Testing

- `romm-api`: merge logic unit tests (fixture JSON)
- `romm-tui`: `achievement_lines` state rendering tests in `game_detail/tests.rs`
- Manual: game with RA match + synced user on RomM 4.9+

## Prerequisites (operator)

1. `RETROACHIEVEMENTS_API_KEY` on RomM
2. ROM scan matched (`ra_id`)
3. User `ra_username` + sync via RomM web UI

## Reference

- [2026-06-26-metadata-editing-research.md](2026-06-26-metadata-editing-research.md) — `merged_ra_metadata`, `DetailedRomSchema`
- RomM metadata docs: RetroAchievements setup
- Implementation plan: [2026-06-27-achievements-game-detail.md](2026-06-27-achievements-game-detail.md)
