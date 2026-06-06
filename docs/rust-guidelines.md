# Rust guidelines for romm-cli

Reference for Rust CLI/TUI best practices and known improvement areas in this project. Use this when implementing features or refactoring so changes stay aligned with current ecosystem conventions (2025–2026).

For how the codebase is organized today, see [architecture.md](./architecture.md).

---

## Industry consensus (2025–2026)

Modern Rust CLI tools (ripgrep, bat, starship, uv-style) generally follow:

| Area | Recommended approach |
|------|----------------------|
| **Structure** | Thin `main.rs`, library crate (`lib.rs`), `commands/` per subcommand, separate `client/` for I/O |
| **CLI** | `clap` derive API, global flags, shell completions via `clap_complete` |
| **Async** | `tokio` only at I/O boundaries; keep business logic sync where possible |
| **Errors** | `thiserror` in library/public APIs; `anyhow` at the binary boundary with `.context()` |
| **Logging** | `tracing` + `tracing-subscriber` (not raw `println!` for diagnostics) |
| **HTTP** | `reqwest` + `rustls` (avoid OpenSSL), typed endpoints, serde models |
| **TUI** | `ratatui` + `crossterm`, event loop + channels for background work |
| **Testing** | `assert_cmd` + `wiremock` / `httpmock` for integration tests |
| **Distribution** | `strip = true`, cross-compiled release artifacts, optional self-update |

### External references

- [Rust CLI tools best practices guide](https://github.com/Dicklesworthstone/coding_agent_session_search/blob/main/RUST_CLI_TOOLS_BEST_PRACTICES_GUIDE.md)
- [Modern Rust CLI development (2026 cheat sheet)](https://techbytes.app/posts/modern-rust-cli-development-2026-cheat-sheet/)
- [Microsoft RustTraining — error handling patterns](https://github.com/microsoft/RustTraining/blob/main/rust-patterns-book/src/ch10-error-handling-patterns.md)
- [Ratatui FAQ — when to use tokio](https://ratatui.rs/faq/)
- [Ratatui async actions tutorial](https://ratatui.rs/tutorials/counter-async-app/full-async-actions/)
- [Ratatui templates — component / event-driven-async](https://github.com/ratatui/templates)

---

## Target architecture (API client + CLI + TUI)

**Current layout** (single crate — maintained; see Gap 4 for why a workspace split is not planned):

```text
romm-cli/
├── Cargo.toml          # lib + bins, optional `tui` feature
├── src/
│   ├── lib.rs          # public API surface
│   ├── main.rs         # thin: parse → init → dispatch → exit code
│   ├── commands/       # one module per subcommand
│   ├── client/         # HTTP layer only (reqwest hidden here)
│   ├── endpoints/      # typed API routes + request/response types
│   ├── core/           # domain logic (downloads, cache, resolve)
│   ├── config.rs       # file + keyring auth
│   ├── frontend/       # CLI vs TUI dispatch
│   └── tui/            # screens, handlers, background tasks
│       ├── app/
│       ├── screens/
│       └── app/background/   # tokio tasks → mpsc → UI
└── tests/              # integration tests against mocked API
```

### Design rules

1. **Library-first** — binaries are thin wrappers; logic lives in the crate so CLI, TUI, and tests share it.
2. **One HTTP client type** — everything goes through `RommClient`; no scattered `reqwest` calls.
3. **Feature-gated TUI** — `default = ["tui"]` but CI can build `--no-default-features` for headless/smaller artifacts.
4. **Typed errors at boundaries** — e.g. `ApiError`, `ConfigError`, `DownloadError`; compose with `#[from]`; use `anyhow` only in `main`.
5. **Background work via channels** — TUI main loop polls/receives; network/download runs on tokio tasks.

---

## What romm-cli already does well

- Library crate with `client`, `commands`, `endpoints`, `core`, `frontend` split
- `clap` derive, global `--json` / `--verbose`, optional TUI feature
- `reqwest` with `rustls`, no native-tls (good for cross-builds)
- `tracing`, `keyring`, `wiremock` / `assert_cmd` in dev-dependencies
- TUI uses screen modules + background task polling
- Release profile strips binaries

---

## Improvement gaps

Each gap below is a self-contained guideline. When working on one, read its section and check off acceptance criteria before considering it done.

---

### Gap 1: Typed errors in the client layer

**Current state:** `RommClient` and `load_config` return typed errors (`ApiError`, `ConfigError`, `DownloadError`, composed as `RommError`). Some legacy command handlers still use `anyhow` internally and are wrapped at the CLI boundary; TUI background channels may still stringify errors.

**Recommended approach:**

- Define per-domain error enums with `thiserror`:
  - `ApiError` — HTTP status, auth, rate limits
  - `ConfigError` — missing file, invalid JSON, keyring failures
  - `DownloadError` — I/O, checksum, interrupted transfer
- Use `#[from]` for automatic conversion from lower-level errors (`reqwest::Error`, `serde_json::Error`, `std::io::Error`).
- Compose at higher levels with `#[error(transparent)]` or wrapper variants.
- Reserve `anyhow` for `main.rs` / top-level `run_app()` only; map typed errors to user-facing messages and exit codes there.

**Rule of thumb:** Use `thiserror` when callers need to *handle* errors (match on variants). Use `anyhow` when callers only need to *report* errors (log, display, exit).

**Example pattern:**

```rust
#[derive(Debug, thiserror::Error)]
pub enum ApiError {
    #[error("unauthorized — check credentials")]
    Unauthorized,
    #[error("request failed: {0}")]
    Request(#[from] reqwest::Error),
    #[error("invalid response: {0}")]
    Parse(#[from] serde_json::Error),
}
```

**Acceptance criteria:**

- [x] Public API of `RommClient` returns typed errors (or a composed `RommError`), not `anyhow::Error`
- [x] CLI/TUI map known variants to actionable messages (e.g. “run `romm-cli init`” on auth failure) via `user_message()` / `set_error(RommError)`
- [x] Error chains preserved via `#[source]` / `#[from]` for debugging with `{:#}` on typed variants

**References:** [Microsoft error handling patterns](https://github.com/microsoft/RustTraining/blob/main/rust-patterns-book/src/ch10-error-handling-patterns.md), [thiserror + anyhow guide](https://oneuptime.com/blog/post/2026-01-25-error-types-thiserror-anyhow-rust/view)

---

### Gap 2: Shell completions

**Current state:** `build.rs` reruns completion generation when CLI modules change (via `romm-complete-gen` after the first build). Static scripts live in `completions/`; users can also run `romm-cli completions <shell>`.

**Recommended approach:**

- Add `clap_complete` as a build dependency (or dev-dependency + `build.rs`).
- Generate completions in `build.rs` from the `Cli` / `Commands` types in `commands/mod.rs`.
- Ship scripts under `completions/` (bash, zsh, fish, powershell, elvish).
- Document install steps in README (e.g. `romm-cli completions bash > …` or copy from `completions/`).

**Acceptance criteria:**

- [x] `build.rs` regenerates completions when CLI surface changes
- [x] At least bash, zsh, fish, and PowerShell covered
- [x] README section explains how to install completions per shell

**References:** [Clap completions docs](https://docs.rs/clap_complete/latest/clap_complete/), [Rust CLI best practices guide — completions section](https://github.com/Dicklesworthstone/coding_agent_session_search/blob/main/RUST_CLI_TOOLS_BEST_PRACTICES_GUIDE.md)

---

### Gap 3: Meaningful exit codes

**Current state:** `exit_code()` in `src/error.rs` maps `RommError` to `romm_cli::exit::{SUCCESS,GENERAL,USAGE,CONFIG,API}`. `main.rs` calls `std::process::exit(exit_code(&e))`. Legacy command handlers still return `anyhow::Result`; `from_anyhow()` downcasts typed domain errors at the CLI boundary. Clap parse errors exit with `2` before `run_app()` runs.

**Recommended approach:**

- Define a small `ExitCode` enum or constants:
  - `0` — success
  - `1` — general/unknown failure
  - `2` — usage / invalid arguments (clap already uses 2 for parse errors if configured)
  - `3` — configuration / auth error
  - `4` — API / network error
- Map typed errors (Gap 1) to codes in `run_app()` before `std::process::exit`.
- Document exit codes in README for scripting users.

**Acceptance criteria:**

- [x] Scripts can distinguish auth/config failures from generic errors
- [x] Exit codes documented in README or `--help` long about text
- [x] Integration tests assert expected exit codes where relevant

**References:** [Rust CLI error handling with clap](https://oneuptime.com/blog/post/2026-01-07-rust-cli-clap-error-handling/view)

---

### Gap 4: Workspace split (not planned)

**Current state:** **Single crate is the maintained approach.** One package hosts the library plus binaries (`romm-cli`, `romm-tui`, `romm-openapi-gen`, `romm-complete-gen`). TUI is feature-gated (`default = ["tui"]`); CI runs `--no-default-features` for headless builds. A workspace split is **not on the roadmap** until an ADR trigger fires — see [workspace-split ADR](plans/2026-06-06-workspace-split-adr.md). The [migration playbook](plans/2026-06-06-workspace-split-migration.md) is reference only.

**Do not split preemptively.** Prefer the `tui` feature flag and module boundaries (`core/` vs `tui/`) until compile times, external library consumers, or a third frontend force the issue.

**Recommended approach (if a trigger fires later):**

Split only when compile times or API boundaries justify it:

```text
romm-cli/          # workspace root
├── romm-api/      # client, endpoints, types, core (no clap/ratatui)
├── romm-cli/      # binary + commands
└── romm-tui/      # binary + tui/ (depends on romm-api)
```

- Keep `openapi_gen` as a bin in `romm-api` or a small `tools/` crate.
- Share types and `RommClient` from `romm-api`; frontends depend on it.

**When to do it:** Only after an ADR trigger — external `RommClient` consumer, compile-time pain, third frontend, or separate crates.io publish. Revisit quarterly; otherwise treat Gap 4 as **done (deferred by design)**.

**Acceptance criteria:**

- [x] N/A until split is triggered — decision criteria documented; maintainers chose to wait
- [ ] If split: `cargo test -p romm-api` runs without TUI feature graph *(out of scope until triggered)*

**References:** [Rust project structure best practices](https://www.djamware.com/post/rust-project-structure-and-best-practices-for-clean-scalable-code)

---

### Gap 5: TUI event / action separation

**Current state:** `tui/app/run.rs` delegates to `TuiSession` + `poll_frame_events` → `map_event` → `App::update`. Background work drains via `drain_background_events` → `Action`. Screen key mapping lives in `handlers/screen_keys.rs`; complex screens still use per-screen `*Key` actions applied in `update.rs`.

**Recommended approach (Ratatui async template):**

- Add `tui/event.rs` (or `tui/app/event.rs`):
  - `Event` — raw input (key, resize, tick, background message)
  - `Action` — semantic intents (`NavigateHome`, `StartDownload`, `Quit`)
- Main loop becomes: poll events → map to actions → `app.update(action)` → render.
- Background tasks send `Action` on `tokio::sync::mpsc` instead of mutating shared state directly where possible.

**Acceptance criteria:**

- [x] Input handling centralized; render screens do not read `crossterm` (widgets/path picker and setup wizard input are the boundary)
- [x] Background completions arrive as actions via `drain_background_events` → `App::update`
- [x] `run.rs` is loop + dispatch; business logic lives in `update.rs` / handlers

**References:** [Ratatui event-driven-async template](https://github.com/ratatui/templates/blob/main/event-driven-async/README.md), [Full async actions tutorial](https://ratatui.rs/tutorials/counter-async-app/full-async-actions/), [Component template](https://ratatui.rs/templates/component/)

---

### Gap 6: Layered configuration

**Current state:** Layered merge in [`load_config()`](../src/config.rs) (defaults → `config.json` → env → keyring) plus narrow command-specific CLI runtime overrides. Documented in module docs, README, and [architecture.md](./architecture.md).

**Precedence model (per field):**

1. Built-in defaults
2. `config.json`
3. Environment variables (override file)
4. OS keyring (resolve secret sentinels after merge)
5. Command-specific CLI runtime (`download --output`, `sync run --download-dir`; `init`/`auth login` persist to file)

There is no global `CLI > env` for connection settings on normal commands.

**Acceptance criteria:**

- [x] Single documented precedence order (see README *Configuration precedence* and `config.rs` module docs)
- [x] Sensitive values never logged by `tracing` (passwords, tokens, API keys)
- [x] `ROMM_*` env vars listed in README

**References:** [Modern Rust CLI — serde flatten](https://techbytes.app/posts/modern-rust-cli-development-2026-cheat-sheet/)

---

### Gap 7: CLI/TUI UX polish

**Current state:** JSON output, `indicatif` progress, `tracing` — good baseline. Some polish items remain inconsistent across commands.

**Recommended approach:**

| Item | Guideline |
|------|-----------|
| **Color** | Respect `NO_COLOR` and `CLICOLOR=0`; disable styling when stdout is not a TTY |
| **Errors** | Print to stderr; use `{:#}` for full anyhow chains in verbose mode only |
| **JSON mode** | Stable field names; version schema in docs if scripts depend on it |
| **Progress** | Use `indicatif` for long operations; suppress bars when `--json` or non-TTY |
| **Help text** | Every subcommand has `about` + examples where non-obvious |

**Acceptance criteria:**

- [ ] No ANSI codes when `NO_COLOR` is set
- [ ] `--json` never interleaves human progress UI on stdout
- [ ] Error messages suggest next step where possible (init, auth, check URL)

**References:** [Building CLI tools with Clap (2026)](https://lucaberton.com/blog/rust-cli-tools-clap-2026/)

---

## Reference projects to study

| Project / template | Learn from it |
|--------------------|---------------|
| [ratatui/templates — component](https://github.com/ratatui/templates/tree/main/component) | Tokio + screen components + async events |
| [ratatui/templates — event-driven-async](https://github.com/ratatui/templates/tree/main/event-driven-async) | Minimal event/action loop |
| ripgrep, bat, fd | CLI ergonomics, performance, error messages |
| starship, uv | Self-update, cross-platform releases, feature flags |

---

## Quick checklist before merging significant changes

- [ ] Logic in `core/` or `client/`, not duplicated in CLI and TUI
- [ ] New API surface goes through `endpoints/` + `RommClient`
- [ ] Errors typed or given context; no new bare `unwrap()` in production paths
- [ ] Tests: unit tests for pure logic; `wiremock`/`httpmock` for HTTP; `assert_cmd` for CLI if behavior is user-visible
- [ ] TUI changes: background work does not block the render loop
- [ ] Feature `tui` remains optional; CI can build without it

---

*Last updated: 2026-06-06 — Gap 6 complete (layered config docs + tracing redaction); Gap 5 complete (event/action pipeline); Gap 4 closed as deferred (single crate maintained).*
