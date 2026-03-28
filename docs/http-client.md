# HTTP client (`RommClient`) and endpoints

This document focuses on how HTTP calls are structured.

## Endpoint trait

Each ROMM API route is described by a small type implementing an
`Endpoint` trait:

- method (e.g. `"GET"`)
- path (e.g. `"/api/platforms"`)
- query params (`Vec<(String, String)>`)
- optional JSON body

These types live under `src/endpoints/*`.

## RommClient

`RommClient` is a thin wrapper around `reqwest::Client`:

- stores:
  - base URL (RomM **site** origin — same as in the browser; no `/api`. Paths already include `"/api/..."`.)
  - authentication config
- exposes:
  - `call<E: Endpoint>(&self, ep: &E)` – typed request/response
  - `request_json` – lower-level helper that just returns `serde_json::Value`
  - `download_rom` – specialized streaming download with a progress
    callback

The idea is that frontends never touch `reqwest` directly; they use
`RommClient` and endpoint types instead.

## Streaming downloads

`download_rom` demonstrates how to:

- build a URL with query parameters
- stream the body by repeatedly calling `resp.chunk().await?`
- write chunks to disk
- call a callback with `(received_bytes, total_bytes)` so callers can
  display progress

`DownloadManager` builds on top of this by:

- creating a `DownloadJob`
- spawning a `tokio` task
- updating shared progress state in an `Arc<Mutex<Vec<DownloadJob>>>`

