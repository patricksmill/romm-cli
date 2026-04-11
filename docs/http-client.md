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
  - `verbose` flag (when true, logs HTTP requests to stderr)
- uses a custom `User-Agent` (`romm-cli/VERSION`) which can be overridden via the `ROMM_USER_AGENT` environment variable to bypass strict reverse proxies.
- exposes:
  - `call<E: Endpoint>(&self, ep: &E)` – typed request/response
  - `request_json` – lower-level helper that returns `serde_json::Value`. It gracefully handles empty bodies (mapping them to `Value::Null`) and non-JSON text responses (wrapping them in `{"_non_json_body": "..."}`).
  - `download_rom` – specialized streaming download with a progress callback
  - `fetch_openapi_json` – fetches the OpenAPI spec from the server

### OpenAPI Helpers

The client includes logic to automatically discover and fetch the server's OpenAPI specification (`openapi.json`):
- **`resolve_openapi_root`**: Determines the base origin for the spec (respecting `ROMM_OPENAPI_BASE_URL` if the spec is hosted on a different domain).
- **`openapi_spec_urls`**: Generates a list of fallback URLs to try (e.g., `https://.../openapi.json`, `http://.../openapi.json`, `.../api/openapi.json`).
- **`fetch_openapi_json`**: Iterates through the fallback URLs until it successfully downloads the spec.

The idea is that frontends never touch `reqwest` directly; they use
`RommClient` and endpoint types instead.

## Streaming downloads

`download_rom` demonstrates how to:

- build a URL with query parameters
- support HTTP Range requests for **resuming interrupted downloads** (checks the existing file size and sends `Range: bytes=X-`, handling `206 Partial Content` vs `200 OK`)
- stream the body by repeatedly calling `resp.chunk().await?`
- write chunks to disk
- call a callback with `(received_bytes, total_bytes)` so callers can
  display progress

`DownloadManager` builds on top of this by:

- creating a `DownloadJob`
- spawning a `tokio` task
- updating shared progress state in an `Arc<Mutex<Vec<DownloadJob>>>`

