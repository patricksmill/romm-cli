# Game metadata editing — API research notes

**Status:** Research only (no implementation).  
**Purpose:** Ground a future plan for metadata edit/scrape across `romm-api`, `romm-cli`, and `romm-tui`.  
**Sources:** Bundled OpenAPI (`romm-tui/openapi.json` v4.6.1), live RomM demo OpenAPI (v4.9.2), RomM upstream (`rommapp/romm` master), RomM docs (4.8–4.9), and this repo’s existing CLI/TUI wiring.

---

## Executive summary

RomM already exposes everything needed to **search metadata providers and apply matches** without the CLI/TUI holding IGDB, ScreenScraper, MobyGames, or SteamGridDB API keys. The server stores those credentials and proxies requests.

The natural client flow mirrors the RomM web UI:

1. **Search** — `GET /api/search/roms` (and optionally `GET /api/search/cover` for SteamGridDB artwork).
2. **Apply match** — `PUT /api/roms/{id}` with `multipart/form-data`, setting a provider ID (e.g. `igdb_id`). RomM fetches full metadata server-side and persists it.
3. **Manual overrides** — same `PUT` endpoint can set `name`, `summary`, `fs_name`, `url_cover`, `url_manual`, or upload `artwork`.
4. **Unmatch** — `PUT /api/roms/{id}?unmatch_metadata=true` clears provider links and resets scraped fields.

Bulk “rescan unmatched / update all metadata” is a **different** surface: the web UI drives rich scans over **WebSocket** (`scan` event with `type`, `platforms`, `apis`). The REST task `POST /api/tasks/run/scan_library` is a narrower, sometimes unavailable path (see gaps below).

**No client-side metadata API keys are required** for search-and-match flows, as long as the RomM instance has providers configured and enabled.

---

## Authentication and scopes

All endpoints require auth (Basic or OAuth2 bearer). Relevant write scopes from [RomM API docs](https://docs.romm.app/4.8.1/API-and-Development/API-Reference/):

| Scope | Needed for |
|-------|------------|
| `roms.read` | List/get ROM, search endpoints |
| `roms.write` | `PUT /api/roms/{id}` (game metadata, match, cover/manual) |
| `roms.user.write` | `PUT /api/roms/{id}/props` (per-user play state, rating, backlog) |
| `platforms.write` | `PUT /api/platforms/{id}` (platform display name, aspect ratio) |
| `tasks.run` | `POST /api/tasks/run/{task_name}` |

Existing romm-cli auth (username/password or OAuth token in config) should suffice; no new secret types.

---

## What romm-cli already has

### `romm-api` endpoints (typed, in `endpoints/roms.rs`)

| Endpoint | Implemented | Notes |
|----------|-------------|-------|
| `GET /api/roms`, `GET /api/roms/{id}` | Yes | `Rom` type is a **subset** of `DetailedRomSchema` |
| `GET /api/search/roms` | Yes | `GetSearchRoms` → raw `Value` |
| `GET /api/search/cover` | Yes | `GetSearchCover` → raw `Value` |
| `PUT /api/roms/{id}/props` | Yes | `PutRomUserProps` |
| `GET/POST/PUT/DELETE` notes | Yes | |
| `POST /api/roms/{id}/manuals` | Yes | via `RommClient::upload_rom_manual` |
| **`PUT /api/roms/{id}`** | **No** | Core metadata edit / match endpoint |
| `GET /api/roms/identifiers` | No | New in newer RomM (list of ROM ids) |
| `PUT /api/platforms/{id}` | Typed (`PutPlatform`) | No CLI command yet |

### `romm-cli` (`roms` subcommands)

Already exposed:

- `roms cover-search` — calls both `GetSearchCover` and `GetSearchRoms`, prints combined JSON.
- `roms props` — user properties (`backlogged`, `rating`, `status`, etc.).
- Notes CRUD, manual upload, find-by-hash/metadata-id.

**Not exposed:** applying a search result (`PUT /api/roms/{id}`), unmatch, remove cover, editing `name`/`summary`/`fs_name`.

### `romm-tui`

Game detail screen is **read-only** for metadata (title, summary, cover URL, file path). No search-or-match UI.

### OpenAPI snapshot gap

Bundled `romm-tui/openapi.json` is **4.6.1** and under-documents `PUT /api/roms/{id}` (only `artwork` in the generated body schema). Demo RomM **4.9.2** OpenAPI lists the full multipart field set (see below). Plan should include refreshing the bundled spec or generating types from a live instance.

---

## Core API: update ROM metadata (`PUT /api/roms/{id}`)

**Method:** `PUT`  
**Content-Type:** `multipart/form-data`  
**Scope:** `roms.write`  
**Response:** `DetailedRomSchema` (rich; includes nested `igdb_metadata`, `ss_metadata`, genres, regions, siblings, etc.)

### Query parameters

| Param | Type | Effect |
|-------|------|--------|
| `remove_cover` | bool | Remove stored cover images |
| `unmatch_metadata` | bool | Clear all provider IDs and scraped metadata; reset name to `fs_name`, clear summary/cover/manual |

When `unmatch_metadata=true`, the body is ignored; RomM performs a full unmatch server-side.

### Form fields (RomM 4.9.2 OpenAPI / upstream `update_rom`)

**Provider IDs** (strings in form; RomM coerces to int where needed). Setting a new ID triggers **server-side fetch** from that provider:

- `igdb_id`, `moby_id`, `ss_id`, `ra_id`, `launchbox_id`, `hasheous_id`, `tgdb_id`, `flashpoint_id`, `hltb_id`, `libretro_id`, `sgdb_id`

**Raw metadata overrides** (JSON strings):

- `raw_igdb_metadata`, `raw_moby_metadata`, `raw_ss_metadata`, `raw_launchbox_metadata`, `raw_hasheous_metadata`, `raw_flashpoint_metadata`, `raw_hltb_metadata`, `raw_manual_metadata`

**Direct fields:**

- `name`, `summary`, `fs_name` (renaming on disk if changed; re-parses filename tags)
- `url_cover`, `url_manual` (RomM validates URL, fetches server-side; SSRF protections apply)
- `artwork` — binary file upload for custom cover (PNG/JPEG/WebP/GIF; validated with libmagic)

**Upstream behavior when an ID changes** (from `backend/endpoints/roms/__init__.py`):

- Calls the matching `meta_*_handler.get_rom_by_id` (IGDB, SS, Moby, RA, LaunchBox, Flashpoint, etc.).
- Merges returned metadata into DB; downloads cover/manual/screenshots/media to RomM storage.
- RetroAchievements badges and ScreenScraper media types handled when `ss_id` changes.

**Implication for romm-cli:** need a `multipart` upload helper on `RommClient` (pattern exists for saves/states/screenshots in `client/upload.rs`). JSON-only `request_json` is insufficient for `artwork` file upload.

---

## Search metadata providers (`GET /api/search/roms`)

**Scope:** `roms.read`

| Query | Required | Description |
|-------|----------|-------------|
| `rom_id` | yes | Context ROM (platform used for provider queries) |
| `search_term` | no | Defaults to ROM’s `fs_name_no_tags` |
| `search_by` | no | `"name"` (default) or `"id"` |

**Returns:** `list[SearchRomSchema]` — merged candidates across enabled providers.

Each result includes (among others): `name`, `slug`, `summary`, `platform_id`, `is_identified`, provider IDs (`igdb_id`, `moby_id`, `ss_id`, `launchbox_id`, `flashpoint_id`, `sgdb_id`), and per-provider cover URLs (`igdb_url_cover`, `ss_url_cover`, etc.).

**Server requirements:** At least one of IGDB, ScreenScraper, MobyGames, Flashpoint, or LaunchBox (cloud) must be enabled; otherwise **500** `"No metadata providers enabled"`.

**Search flow (matches web UI “identify game”):**

1. `GET /api/search/roms?rom_id={id}&search_term={query}&search_by=name`
2. Present results to user.
3. `PUT /api/roms/{id}` with the chosen `igdb_id` / `ss_id` / … (only fields being set need to be sent; RomM uses `model_fields_set` for partial form updates).

`search_by=id` looks up a specific provider ID across IGDB, Moby, SS, LaunchBox in parallel.

---

## Cover search (`GET /api/search/cover`)

**Scope:** `roms.read`

| Query | Description |
|-------|-------------|
| `search_term` | Search string (can be empty) |

**Returns:** `list[SearchCoverSchema]` — SteamGridDB grids with `name` and `resources[]` (dimensions, URLs).

**Requires:** SteamGridDB enabled on the server (`STEAMGRIDDB_API_ENABLED` in heartbeat). Otherwise **500** / **401** on bad SGDB key.

**Typical use:** pick a grid URL, then `PUT /api/roms/{id}` with `url_cover={sgdb_url}` or set `sgdb_id` if matching via metadata path.

---

## Per-user ROM data (`PUT /api/roms/{id}/props`)

**Scope:** `roms.user.write`  
**Content-Type:** `application/json` (RomM 4.9+: body is `RomUserData` directly; query flags separate)

| Field | Type | Notes |
|-------|------|-------|
| `is_main_sibling`, `backlogged`, `now_playing`, `hidden` | bool | |
| `rating`, `difficulty` | 0–10 | |
| `completion` | 0–100 | |
| `status` | enum | `incomplete`, `finished`, `completed_100`, `retired`, `never_playing` |

Query: `update_last_played`, `remove_last_played` (mutually exclusive).

**Already implemented** in CLI as `roms props`. Distinct from **game metadata** (IGDB summary, genres, etc.).

---

## Platform metadata (`PUT /api/platforms/{id}`)

**Scope:** `platforms.write`  
**Body:** JSON — at minimum `custom_name`, `aspect_ratio` (per 4.9.2 OpenAPI).

Typed endpoint exists in `romm-api`; no CLI/TUI surface yet. Lower priority unless we want platform rename in TUI.

---

## Manuals

| Endpoint | Status in romm-api |
|----------|-------------------|
| `POST /api/roms/{id}/manuals` | `upload_rom_manual` (raw body + `x-upload-filename`) |
| `DELETE /api/roms/{id}/manuals` | Not wrapped |
| Via `PUT` with `url_manual` or manual file | Not wrapped |

---

## Reading rich metadata (`GET /api/roms/{id}`)

Response schema: **`DetailedRomSchema`** (4.9.2), far larger than our `Rom` struct.

Notable nested objects:

| Field | Content |
|-------|---------|
| `igdb_metadata` | ratings, release date, genres, franchises, companies, age ratings, similar games, … |
| `ss_metadata` | many artwork types (box, fanart, wheel, video, …) |
| `moby_metadata`, `launchbox_metadata`, `hasheous_metadata`, `hltb_metadata`, `flashpoint_metadata`, `gamelist_metadata` | provider-specific |
| `merged_ra_metadata` | achievements |
| `regions`, `languages`, `tags`, `alternative_names`, `siblings` | |
| `rom_user` | per-user props embedded in detail view |
| `is_identifying` | async identification in progress |

**Gap:** expanding `Rom` (or adding `DetailedRom`) is required to display/edit meaningfully in TUI. Deserializing with `serde` + `#[serde(flatten)]` or a dedicated `DetailedRom` with `Value` fallbacks for nested metadata are options for the plan phase.

---

## Metadata provider health (`GET /api/heartbeat`)

`HeartbeatResponse.METADATA_SOURCES` → `MetadataSourcesDict`:

- `ANY_SOURCE_ENABLED`
- Per-provider: `IGDB_API_ENABLED`, `SS_API_ENABLED`, `MOBY_API_ENABLED`, `STEAMGRIDDB_API_ENABLED`, `RA_API_ENABLED`, `LAUNCHBOX_API_ENABLED`, `HASHEOUS_API_ENABLED`, `PLAYMATCH_API_ENABLED`, `TGDB_API_ENABLED`, `FLASHPOINT_API_ENABLED`, `HLTB_API_ENABLED`

`GET /api/heartbeat/metadata/{source}` — boolean liveness check for one source.

**Use in UI:** before offering search, check heartbeat so we can show “IGDB not configured on server” instead of opaque 500s.

---

## Bulk metadata operations (scan tasks)

### What the web UI does

Socket event `scan` with options (from `backend/endpoints/sockets/scan.py`):

```json
{
  "platforms": [1, 2],
  "type": "quick",
  "apis": ["igdb", "ss", "moby"],
  "roms_ids": [],
  "launchbox_remote_enabled": true,
  "playmatch_enabled": true
}
```

`type` maps to `ScanType`: `new_platforms`, `quick`, `update`, `unmatched`, `complete`, `hashes`.

`apis` values match `MetadataSource` enum: `igdb`, `moby`, `ss`, `ra`, `launchbox`, `hasheous`, `tgdb`, `sgdb`, `flashpoint`, `hltb`, `gamelist`, `libretro`, `playmatch`.

**There is no REST equivalent** with this parameter surface in the bundled OpenAPI. Web UI scan progress uses WebSocket (`scan:log`, `scan:done_ok`, etc.).

### What romm-cli uses today

`POST /api/tasks/run/scan_library` with optional body `{"platform_slugs": ["gba"]}` — see [scan-after-upload.md](../scan-after-upload.md).

### Upstream realities (important for planning)

1. **`scan_library` scheduled task** (`backend/tasks/scheduled/scan_library.py`): `manual_run=False` in current master → `POST /api/tasks/run/scan_library` returns **400** “task cannot be run” on many instances. Already documented in this repo.
2. **`scan_library.run()`** does not accept `platform_slugs` or `scan_type` in current master; it always runs a scheduled-style **Quick** scan of all enabled providers. The CLI’s `platform_slugs` kwarg may be ignored or cause errors depending on RomM version.
3. **`sync_folder_scan`** manual task is for **device save sync folders**, not library metadata.

**Conclusion:** per-ROM metadata edit via `PUT` + `GET /api/search/roms` is the reliable REST story. Bulk unmatched/update scans may need WebSocket support, upstream RomM REST improvements, or documenting “use web UI / fix server task flags” as out of scope for v1.

---

## Other related endpoints

| Endpoint | Relevance |
|----------|-----------|
| `GET /api/roms/by-metadata-provider` | Find existing ROM by provider ID (dedup when linking) — CLI `roms find --igdb-id` |
| `GET /api/roms/by-hash` | Hash-based lookup — CLI `roms find --md5` etc. |
| `GET /api/roms/filters` | Genre/franchise filter values — CLI `roms filters` |
| `POST /api/roms/delete` | Already in CLI |
| `GET /api/platforms/supported` | IGDB platform catalog for setup |

---

## Suggested layering (for later plan, not implementation)

```
romm-api
  endpoints/roms.rs     → PutRom (multipart builder)
  client/upload.rs      → update_rom_multipart(...)
  types.rs              → DetailedRom and/or SearchRom, SearchCover types
  core/metadata.rs?     → optional: search + apply match helpers (thin)

romm-cli
  roms match / roms edit / roms unmatch  (or subcommands under roms metadata)

romm-tui
  game detail: edit mode, search picker, apply match, refresh ROM after PUT
```

---

## Open design questions (grill-me)

Each item needs a decision before implementation. **Recommended default** in italics.

### 1. Scope of “metadata editing” for v1?

- **A)** Match/unmatch + edit `name`/`summary`/`cover` only (parity with web “identify + basic edit”).
- **B)** Full `DetailedRom` field exposure (genres, regions, raw provider JSON, siblings, …).
- **C)** Split: v1 = A; v2 = B.

*Recommendation: **C** — ship search-and-match + basic text/cover first; expand types when TUI layout is proven.*

### 2. User props vs game metadata?

`roms props` already covers backlog/rating/status. Should TUI merge these into one “edit” screen or keep separate (“My progress” vs “Game info”)?

*Recommendation: **separate panes** — different API endpoints, scopes, and mental models.*

### 3. CLI UX for match flow?

- **A)** Two commands: `roms search <id> --query …` then `roms match <id> --igdb-id …`.
- **B)** Interactive TUI-style picker in terminal (`dialoguer` list).
- **C)** Single `roms match <id> --query … --apply 0` non-interactive.

*Recommendation: **A + B** — scriptable JSON flags plus optional interactive when stdout is a TTY.*

### 4. Multipart vs JSON for `PUT /api/roms/{id}`?

RomM requires multipart (even for text-only updates). Acceptable to always use multipart internally?

*Recommendation: **yes** — one code path; mirror existing upload helpers.*

### 5. `DetailedRom` typing strategy?

- **A)** Full structs for all nested metadata (large, brittle across RomM versions).
- **B)** `DetailedRom { … common fields …, extra: Value }` with typed search results only.
- **C)** Keep `Rom` for lists; fetch `Value` for detail/edit until schema stabilizes.

*Recommendation: **B** for list/detail split — typed `SearchRom`/`SearchCover`, pragmatic `Value` or partial structs for provider metadata blobs.*

### 6. Bulk “rescan unmatched” in CLI/TUI?

- **A)** Out of scope v1 (document: use RomM web UI).
- **B)** Add WebSocket client for scan control (large).
- **C)** Only fix/extend REST `scan_library` if server enables `manual_run` and kwargs.

*Recommendation: **A for v1** — per-ROM REST path is solid; bulk scan REST is inconsistent upstream.*

### 7. OpenAPI / min RomM version?

Bundled spec is 4.6.1; `PUT` body incomplete. Bump `min_romm_api` in [compatibility.toml](../compatibility.toml)?

*Recommendation: target **4.8+** (typed form fields, `RomUserData` props shape) and refresh bundled OpenAPI from demo or release tag.*

### 8. Cache invalidation after edit?

After `PUT`, invalidate `RomCache` platform entry + library metadata snapshot?

*Recommendation: **yes** — same pattern as post-scan invalidation; optional background refetch in TUI.*

### 9. OAuth scope requirements?

Do we document/enforce `roms.write` on client tokens used for metadata edit?

*Recommendation: **yes** — fail fast at startup or first edit with a clear error if heartbeat/token scopes available.*

### 10. Cover search dependency?

`GET /api/search/cover` requires SGDB on server. Separate command or bundled into match wizard?

*Recommendation: **optional step** in match flow; skip gracefully when `STEAMGRIDDB_API_ENABLED` is false.*

---

## References

- RomM API overview: https://docs.romm.app/4.8.1/API-and-Development/API-Reference/
- Scan modes: https://docs.romm.app/4.9.0/administration/scanning-and-watcher/
- Scheduled tasks: https://docs.romm.app/4.9.0/administration/scheduled-tasks/
- Live OpenAPI example: https://demo.romm.app/openapi.json (fetched 4.9.2)
- Upstream: `backend/endpoints/roms/__init__.py` (`update_rom`), `backend/endpoints/search.py`, `backend/handler/scan_handler.py`
- This repo: `romm-api/src/endpoints/roms.rs`, `romm-cli/src/commands/roms.rs`, `docs/scan-after-upload.md`
