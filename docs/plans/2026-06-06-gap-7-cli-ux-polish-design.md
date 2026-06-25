# Gap 7: CLI/TUI UX Polish Design

**Status:** Implemented

## Overview

Close [rust-guidelines.md](../rust-guidelines.md) Gap 7 by centralizing CLI presentation rules (color, progress, JSON stdout) and applying them consistently across commands. TUI `NO_COLOR` handling in `tui/theme.rs` is unchanged; this design covers the `romm-cli` binary only.

## `CliPresentation`

New module `src/cli_presentation.rs`:

```rust
pub struct CliPresentation {
    pub format: OutputFormat,
    pub verbose: bool,
}
```

Built once in `frontend/cli.rs` and passed to handlers that emit human UI or long-running progress.

### Color (`supports_ansi_color`)

Disable ANSI when any of:

1. `NO_COLOR` is set (any value) — [no-color.org](https://no-color.org/)
2. `CLICOLOR=0` — unless `CLICOLOR_FORCE=1`
3. `stdout` is not a terminal

Progress templates use a plain variant (no `.cyan/blue`) when color is disabled.

### Progress (`shows_progress`, `progress_draw_target`)

| Condition | Progress |
|-----------|----------|
| `OutputFormat::Json` | Suppressed entirely |
| Non-interactive stdout | Suppressed |
| Text + TTY stdout | Allowed on **stderr** draw target |

**Rule:** indicatif must never draw to stdout when `--json` is active. Human status lines (`Found N ROMs…`) go to stderr in text mode; omitted in JSON mode.

### Output helpers

- `emit_status(&self, msg)` — `eprintln!` in text mode, no-op in JSON
- `emit_json<T: Serialize>(&self, value)` — pretty JSON on stdout in JSON mode
- `multi_progress(&self) -> Option<MultiProgress>` — `None` when progress suppressed
- `progress_style(plain, color) -> ProgressStyle`

## JSON stdout shapes (new)

### `download` (batch, single, extras)

```json
{
  "succeeded": 2,
  "failed": 0,
  "cancelled": 0,
  "paths": ["/path/to/rom.zip"]
}
```

Field names are stable within a major release; new fields are additive.

### `update`

```json
{ "status": "updated", "version": "0.39.0" }
```

or `{ "status": "up_to_date", "version": "0.38.0" }`.

### `cache info`

```json
{
  "path": "/home/user/.cache/romm-cli/roms.json",
  "exists": true,
  "size_bytes": 4096,
  "version": 1,
  "entries": 42,
  "parse_error": null
}
```

### Existing commands

`platforms`, `roms`, `sync`, `scan`, `auth status`, `api` — document pass-through API JSON in `docs/json-output.md`; no shape changes unless interleaving is fixed.

Text-only commands: `init`, `completions`, `cache path`, `cache clear`.

## Error UX

### `main.rs`

```text
Error: <user_message>
Details: <full chain>    # only when --verbose
```

Both on stderr.

### `user_message()` extensions

| Variant | Hint |
|---------|------|
| `ApiError::NotFound` | Check server URL and resource ID |
| `ApiError::Forbidden` | Check credentials or run `romm-cli auth` |
| `ApiError::RateLimited` | Retry after a short wait |
| `ApiError::ClientError` (4xx) | Check command arguments |
| `DownloadError::IoContext` | Check disk permissions and path |

Command-local per-item failures (batch download) use `format_command_error(&RommError)` for consistency.

## Help examples (`after_help`)

| Command | Examples |
|---------|----------|
| `download` | single ROM, batch, extras |
| `sync` | `plan`, `run` |
| `scan` | `--wait`, `--platform` |
| `roms upload` | `--scan` |
| `api` | `call GET /api/platforms` |

Top-level `romm-cli --help` after_help links to `docs/json-output.md`.

## Testing

- Unit: `cli_presentation` env/TTY/color/progress rules
- Integration: `tests/cli_output.rs` — `NO_COLOR=1` no ANSI on stdout; `--json` stdout is parseable JSON without spinner artifacts; error hints for NotFound

## Out of scope

- TUI theme / `NO_COLOR` (done)
- Full `anyhow` → typed error migration (Gap 1 residual)
- Machine-readable JSON Schema files (markdown doc only)
