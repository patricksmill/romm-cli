# Post-upload library scan (`--scan`) and `scan` command

RomM does not automatically index a ROM after chunked upload. Users must run a **library scan** so the new game appears in searches, collections, and metadata flows. This document describes what `romm-cli` implements and how it behaves.

## Implemented behavior

### `romm-cli roms upload … --scan`

- **`--scan` / `-s`:** After uploads finish, triggers `POST /api/tasks/run/scan_library` with **no** `task_kwargs` body.
- **Opt-in:** `--scan` defaults to off so bulk uploads can scan once at the end (or via `romm-cli scan`).
- **Batch uploads:** If the path is a directory, every file is uploaded in order. **At most one** `scan_library` run is triggered after the loop (not per file).
- **Scan only after at least one successful upload:** If every file failed, the CLI prints a skip message and does **not** call the scan API.
- **Exit status:** If `--scan` is set and a scan is attempted (because at least one upload succeeded), a failed start or failed `--wait` causes the process to exit with a **non-zero** status (`anyhow::Error` propagated to `main`).

### `romm-cli roms upload … --scan --wait`

- **`--wait`:** Requires `--scan`. Polls `GET /api/tasks/{task_id}` every **2 seconds** until the job reaches a terminal RQ-style status.
- **`--wait-timeout-secs`:** Requires `--wait`. Caps wait time (default **3600** seconds if omitted). On timeout the command fails.
- **UX:** An `indicatif` spinner shows progress while waiting.
- **Terminal statuses:** `finished` is treated as success. `failed`, `stopped`, `canceled`, and `cancelled` fail the command.

### `romm-cli scan` and `romm-cli scan --wait`

Same scan path as upload: shared implementation in [`src/commands/library_scan.rs`](../src/commands/library_scan.rs). Use this when files were uploaded outside the CLI or you only want a rescan.

### JSON output

With global `--json`, `scan` (and the scan phase of `roms upload --scan`) prints pretty JSON: the start response from `run_task`, and after `--wait` an extra `final_status` object with the last poll body.

### On-disk ROM list cache after a successful `--wait`

When a scan finishes in the **finished** state after `--wait`, the CLI updates the persistent ROM cache ([`RomCache`](../src/core/cache.rs)) so the next TUI (or CLI) session does not keep a stale platform list:

- **`roms upload … --scan --wait`:** removes the cache entry for the upload `platform_id` only.
- **`scan --wait`:** removes **all** cached platform lists (full-library scan); collection-type cache entries are not cleared here.

If `--wait` is not used, the cache is left unchanged (the server scan may still be running).

### TUI: rescan from the Library screen

In the Library (consoles / games) screen, **Ctrl+R** starts the same `scan_library` task and waits for completion (no `indicatif` spinner; status is shown in the metadata footer). On success:

1. All platform ROM list entries are removed from `RomCache`, plus the current row’s cache key when it is a collection (not a platform).
2. A library metadata refresh runs, then the current console/collection’s game list is **reloaded automatically** (same deferred fetch path as after a cache miss), so you do not need to move selection to force a refetch.

While a filter/jump search bar is open in the Library, **Ctrl+R** is ignored (same idea as other global shortcuts during typing). Only one scan may run at a time; a second **Ctrl+R** is ignored until the first completes.

## Client API helpers

In [`src/client.rs`](../src/client.rs):

- `RommClient::run_task(task_name, kwargs)` — `POST /api/tasks/run/{task_name}`; optional JSON body for `task_kwargs`.
- `RommClient::get_task_status(task_id)` — `GET /api/tasks/{task_id}`.

## RomM server notes (kwargs and manual run)

Upstream RomM (tasks endpoint) forwards `task_kwargs` to the task’s `run()` method. The **`scan_library`** implementation’s `run(self)` takes **no** extra keyword arguments and always calls internal scan logic with an **empty platform list** (full-library behavior). Passing `platform_id` in kwargs would not scope the scan and could cause the worker job to error on unexpected keys.

**Do not** send `task_kwargs` for `scan_library` from this CLI.

RomM may also set `manual_run=False` on the scheduled `scan_library` task in some versions, which makes `POST …/run/scan_library` return **400** (“task cannot be run”). That is a server configuration/version concern, not something `romm-cli` can fix locally.

## Verification

### Automated

- `cargo check`, `cargo clippy --all-targets --all-features -- -D warnings`, `cargo test`.
- Integration tests in [`tests/cli_scan.rs`](../tests/cli_scan.rs) mock `scan_library` start (and wait with a finished status).

### Manual

```bash
romm-cli roms upload 4 "path/to/rom.bin" --scan
romm-cli roms upload 4 ./folder --scan --wait --wait-timeout-secs 7200
romm-cli scan --wait
```

Confirm the ROM appears in the TUI or API after the scan completes.

## Historical note

Earlier drafts of this doc proposed always passing `{"platform_id": …}` as kwargs; that was **dropped** after verifying the server task signature. Open questions about defaulting `--scan` on vs off are resolved in favor of **opt-in** `--scan` as documented above.
