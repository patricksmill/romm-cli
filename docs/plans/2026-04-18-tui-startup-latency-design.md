# TUI startup latency — design

## Goal

Reduce first-open TUI latency (time to visible platform/collection list) for medium libraries by avoiding full ROM-list work at startup and by reusing a compact on-disk metadata snapshot.

## Approach (metadata-first + snapshot)

1. **Persist** a small JSON snapshot: platforms + merged collections (manual, smart, virtual) + schema version + save timestamp.
2. **On “Library” entry**: hydrate `LibraryBrowseScreen` from the snapshot immediately when present; **do not** await network before switching screens.
3. **Defer** the first pane’s ROM list load via existing `deferred_load_roms` (same as ↑/↓), so the UI paints before the (possibly large) ROM payload.
4. **Background refresh**: `tokio::spawn` fetches live metadata (same endpoints/timeouts as before), merges sources, applies updates if the refresh **generation** still matches (ignores stale completions).
5. **Persist** snapshot after a successful refresh; surface non-fatal source failures in an on-screen footer line instead of blocking the UI.

## Storage

- Default path: OS cache dir / `romm-cli` / `library-metadata-snapshot.json` (alongside ROM list cache).
- Override: `ROMM_LIBRARY_METADATA_SNAPSHOT_PATH` (full file path).

## Error handling

- Snapshot missing, corrupt, or wrong version: treat as empty metadata; show “Loading library metadata…” until refresh completes.
- Partial API failure: merge what succeeded; record warnings in footer; do not replace working stale lists with empties unless no data was loaded.
- Stale async refresh: generation counter on `App` drops completions from older spawns.

## Testing

- Unit tests: snapshot round-trip save/load, invalid JSON ignored, version mismatch ignored.
- Integration-style tests optional: fetch helper returns merged shape (mock client not required for snapshot I/O).

## Out of scope

- Changing ROM pagination or `RomCache` semantics for on-demand loads.
- CLI `roms` command behavior.
