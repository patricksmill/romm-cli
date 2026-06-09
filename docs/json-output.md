# JSON output reference

Use global `--json` (or a subcommand’s local `--json` flag) for machine-readable stdout. Field names are **stable within a major release**; new fields may be added without breaking scripts.

Progress bars and human status lines never appear on stdout in JSON mode.

## Versioning

- Additive changes only within a major version (new optional fields).
- Breaking renames or removals happen only on a major version bump.

## Commands with JSON output

### `platforms list` / `platforms get`

Passthrough RomM API platform JSON (array or object).

### `roms`

Subcommands that call the API return API-shaped JSON. `roms upload` returns:

```json
{
  "uploaded": 2,
  "failed": 0,
  "total": 2
}
```

### `scan`

Without `--wait`: task start payload from the server (`task_id`, `status`, …). `scan --platform <slug[,slug]>` has the same output shape; the platform filter is sent to the server as `platform_slugs` task kwargs.

With `--wait`: start payload plus `final_status` object when the task completes.

### `sync`

Structured plan/run/session output per subcommand (see command help). Unsupported save-sync preflight emits:

```json
{
  "error": "save_sync_unsupported",
  "message": "...",
  "missing_endpoints": ["..."]
}
```

### `auth status`

Effective and on-disk auth mode metadata (no secrets).

### `api call`

Raw API response body as JSON.

### `download`

Summary after batch, single-ROM, or extras downloads:

```json
{
  "succeeded": 1,
  "failed": 0,
  "cancelled": 0,
  "paths": ["/path/to/file.zip"]
}
```

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

### `cache clear`

```json
{ "cleared": true }
```

### `update`

```json
{ "status": "updated", "version": "1.0.1" }
```

or `{ "status": "up_to_date", "version": "1.0.0" }`.

## Text-only commands

These ignore `--json` for structured output (human text or paths only):

- `init`
- `completions`
- `cache path`
