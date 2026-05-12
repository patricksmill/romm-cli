# Save sync (`sync` command)

This document covers the one-shot save sync workflow added to `romm-cli` for RomM sync endpoints.

## Server compatibility

`romm-cli sync` uses sync endpoints introduced in RomM `4.9.0-alpha.2` (pre-release), including:

- `POST /api/sync/negotiate`
- `POST /api/sync/sessions/{session_id}/complete`
- `GET /api/sync/sessions`
- `GET /api/sync/sessions/{session_id}`
- `POST /api/sync/devices/{device_id}/push-pull`

It also uses device and save endpoints:

- `POST/GET /api/devices`
- `POST /api/saves`
- `GET /api/saves/{id}/content`

## Command surface

### Device setup

Register an API-mode device:

```bash
romm-cli sync device register --name "My Handheld" --sync-mode api
```

Inspect devices:

```bash
romm-cli sync device list
romm-cli sync device get <device-id>
```

### Plan only

Negotiate operations from a manifest without changing files:

```bash
romm-cli sync plan --device-id <device-id> --manifest ./sync-manifest.json
```

### Execute sync

Run negotiated operations and complete the sync session:

```bash
romm-cli sync run --device-id <device-id> --manifest ./sync-manifest.json
```

Choose conflict behavior:

```bash
romm-cli sync run --device-id <device-id> --manifest ./sync-manifest.json --conflict fail
romm-cli sync run --device-id <device-id> --manifest ./sync-manifest.json --conflict skip
```

`--conflict fail` is the default.

### Sessions and push-pull

```bash
romm-cli sync sessions list
romm-cli sync sessions list --device-id <device-id>
romm-cli sync sessions get <session-id>
romm-cli sync push-pull <device-id>
```

## Manifest format

Manifest file is JSON:

```json
{
  "saves": [
    {
      "rom_id": 123,
      "path": "saves/zelda.srm",
      "file_name": "zelda.srm",
      "slot": null,
      "emulator": "retroarch"
    }
  ]
}
```

Rules:

- `rom_id` and `path` are required.
- `path` may be absolute or relative to the manifest file directory.
- `file_name` is optional; if omitted, the filename from `path` is used.
- `slot` and `emulator` are optional.

## Runtime behavior

`sync plan` and `sync run` both:

1. Read all manifest save files.
2. Compute local metadata per file:
   - `updated_at` (UTC RFC3339)
   - `file_size_bytes`
   - `content_hash`
3. Call `POST /api/sync/negotiate`.

`sync run` then executes operations:

- `upload`: `POST /api/saves` with `rom_id`, `device_id`, and `session_id`.
- `download`: `GET /api/saves/{id}/content` and writes to `--download-dir` (or manifest directory).
- `no_op`: no file change.
- `conflict`: fail or skip based on `--conflict`.

After processing operations, `sync run` always calls:

- `POST /api/sync/sessions/{session_id}/complete`

with `operations_completed` and `operations_failed`.

## Content hash details

To match RomM server behavior:

- normal files: MD5 of file bytes
- zip files: MD5 of a deterministic string built from sorted zip entries as `name:entry_md5`, joined by newlines

This keeps negotiate comparisons consistent for save files that are zip archives.
