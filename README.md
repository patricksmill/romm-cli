# romm-cli (Rust)

Rust-based CLI client for the ROMM API.

## Prerequisites

- Rust toolchain (stable) via `rustup`
- ROMM server reachable, with API base URL (e.g. `http://mill-server:1738`)

## Environment variables

The CLI reads configuration from the environment (optionally via a `.env` file when run from the repo root):

- `API_BASE_URL` (required) – e.g. `http://mill-server:1738`
- **Authentication (pick one):**
  - Basic auth:
    - `API_USERNAME`
    - `API_PASSWORD`
  - Bearer token:
    - `API_TOKEN` (or `API_KEY`), must not be a placeholder like `your-bearer-token-here`
  - API key in custom header:
    - `API_KEY`
    - `API_KEY_HEADER` (e.g. `X-API-Key`)

Example `.env` in the repo root:

```env
API_BASE_URL=http://mill-server:1738
API_USERNAME=patrick
API_PASSWORD=your-password
```

## Build

From the `romm-cli` directory:

```bash
cargo build --release
```

The compiled binary will be at:

- `target/release/romm-cli`

## Usage

### List platforms

```bash
cd romm-cli
cargo run -- platforms
```

Or with a previously built binary:

```bash
./target/release/romm-cli platforms
```

### Output as JSON

```bash
cargo run -- platforms --json
```

### Help

```bash
cargo run -- --help
cargo run -- platforms --help
```

## Roadmap

- Add ROM-related commands (list/search ROMs, filter by platform)
- Add maintenance commands if exposed by ROMM API
- Add `--verbose`/`--debug` modes for richer error output

