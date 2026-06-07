# HTTP client (`RommClient`) and endpoints

This document focuses on how HTTP calls are structured in the [`romm-api`](api.md) crate.

## Endpoint trait

Each ROMM API route is described by a small type implementing an `Endpoint` trait:

- method (e.g. `"GET"`)
- path (e.g. `"/api/platforms"`)
- query params (`Vec<(String, String)>`)
- optional JSON body

These types live under `romm-api/src/endpoints/*`, grouped by API area:

- `platforms`, `roms`, `collections` for core library operations
- `client_tokens` for pairing-token APIs
- `device`, `saves`, `sync` for save-sync lifecycle (device registration, negotiation, session completion, save content transfer)
- `system` for health/profile/stat endpoints (`/api/heartbeat`, `/api/stats`, `/api/users/me`)
- `tasks` for queue/task runner endpoints (`/api/tasks`, `/api/tasks/run`, `/api/tasks/status`)

## RommClient

`RommClient` is a thin wrapper around `reqwest::Client`:

- stores:
  - base URL (RomM **site** origin — same as in the browser; no `/api`. Paths already include `"/api/..."`.)
  - authentication config
  - `verbose` flag (when true, logs HTTP requests to stderr)
- uses a custom `User-Agent` in the form `Mozilla/5.0 (compatible; romm-cli/<version>; +https://github.com/patricksmill/romm-cli)`, which can be overridden via the `ROMM_USER_AGENT` environment variable to bypass strict reverse proxies.
- exposes:
  - `call<E: Endpoint>(&self, ep: &E)` – typed request/response
  - `request_json` – lower-level helper that returns `serde_json::Value`. It gracefully handles empty bodies (mapping them to `Value::Null`) and non-JSON text responses (wrapping them in `{"_non_json_body": "..."}`).
  - `download_rom` – specialized streaming download with a progress callback
  - `upload_save_file_with_options` – save upload with sync-aware query fields (`device_id`, `session_id`, `slot`, `overwrite`)
  - `download_save_content` – helper for `GET /api/saves/{id}/content` (used by `sync run` download operations)
  - `fetch_openapi_json` – fetches the OpenAPI spec from the server

### OpenAPI Helpers

The client includes logic to automatically discover and fetch the server's OpenAPI specification (`openapi.json`):
- **`resolve_openapi_root`**: Determines the base origin for the spec (respecting `ROMM_OPENAPI_BASE_URL` if the spec is hosted on a different domain).
- **`openapi_spec_urls`**: Generates a list of fallback URLs to try (e.g., `https://.../openapi.json`, `http://.../openapi.json`, `.../api/openapi.json`).
- **`fetch_openapi_json`**: Iterates through the fallback URLs until it successfully downloads the spec.

The idea is that frontends never touch `reqwest` directly; they use `RommClient` and endpoint types instead.

### Error contract

Public `RommClient` methods return typed errors from `romm-api/src/error.rs`:

| Method family | Error type |
|---------------|------------|
| `call`, `request_json`, `get_bytes`, `post_bytes`, uploads, tasks, OpenAPI | `ApiError` |
| `download_rom`, `download_url_*` | `DownloadError` (includes `Api` and `Cancelled` variants) |

HTTP failures are classified by status code (`Unauthorized`, `Forbidden`, `NotFound`, `RateLimited`, `ClientError`, `ServerError`). Use `ApiError::status_code()` and `ApiError::is_auth_failure()` instead of string matching.

`load_config()` returns `ConfigError`. Library consumers should use `RommError` as the composed type; the binary maps it to user messages via `user_message()` and exit codes via `exit_code()`.

Exit code constants live in `romm_api::exit` (`SUCCESS`, `GENERAL`, `USAGE`, `CONFIG`, `API`). Config and auth failures exit with `CONFIG` (3); API, network, and download failures exit with `API` (4). See [cli.md — exit codes](cli.md#exit-codes).

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

