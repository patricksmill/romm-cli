# CLI ⊇ TUI Parity (Phase 1) Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Close Phase 1 CLI ⊇ TUI gaps with config approach D (full registry + source-aware introspection), plus `saves` and `collections` CLI command groups.

**Architecture:** Add `romm-api` config registry and small `core/saves` + `core/collections` helpers; wire thin `romm-cli` commands. Refactor `load_config` to merge via registry while preserving existing precedence. No TUI changes in Phase 1.

**Tech stack:** Rust 2021, `clap`, `serde`/`serde_json`, `thiserror`, `httpmock`, `assert_cmd`, existing `RommClient` + `Endpoint` trait.

**Design spec:** [../specs/2026-08-12-cli-tui-parity-design.md](../specs/2026-08-12-cli-tui-parity-design.md)

## Global Constraints

- **No play/launch:** Do not add emulator launch, EmulatorJS, or netplay features.
- **Precedence:** defaults < `config.json` < env < keyring < command-specific flags (unchanged).
- **`config set`:** Writes `config.json` only; never sets process env.
- **Secrets:** Redact in `config show`; `--reveal-secrets` only on TTY with confirmation.
- **JSON:** Every new subcommand accepts global or local `--json`; document shapes in `docs/json-output.md`.
- **Errors:** Public API returns typed errors; CLI maps via `RommError` / `exit_code()`.
- **Branch:** `cursor/cli-tui-parity-phase1-ffca`.
- **Commits:** Conventional Commits 1.0.0 (`feat(api):`, `feat(cli):`, `docs:`).

---

## File map

| File | Responsibility |
|------|----------------|
| `romm-api/src/config/registry.rs` | `ConfigKey`, env mapping, dotted-path get/set, `ConfigField`, `ConfigSources` |
| `romm-api/src/config.rs` | Refactor merge to call registry; export `load_config_with_sources` |
| `romm-api/src/core/saves.rs` | Save list/download/upload orchestration |
| `romm-api/src/core/collections.rs` | List/get/delete by collection type |
| `romm-cli/src/commands/config.rs` | `config` subcommand group |
| `romm-cli/src/commands/saves.rs` | `saves` subcommand group |
| `romm-cli/src/commands/collections.rs` | `collections` subcommand group |
| `romm-cli/tests/cli_config.rs` | Config command integration tests |
| `romm-cli/tests/cli_saves.rs` | Saves command integration tests |
| `romm-cli/tests/cli_collections.rs` | Collections command integration tests |

---

### Task 0: Branch and baseline

**Files:** (none)

- [ ] **Step 1: Create branch**

```bash
git checkout -b cursor/cli-tui-parity-phase1-ffca
```

- [ ] **Step 2: Verify baseline**

```bash
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test
```

Expected: all green.

- [ ] **Step 3: Commit design artifacts** (if not already committed)

```bash
git add docs/superpowers/specs/2026-08-12-cli-tui-parity-design.md docs/superpowers/plans/2026-08-12-cli-tui-parity-phase1.md
git commit -m "docs: add CLI/TUI parity Phase 1 design and plan"
```

---

### Task 1: Config registry types (`romm-api`)

**Files:**
- Create: `romm-api/src/config/registry.rs`
- Modify: `romm-api/src/config.rs` (add `mod registry; pub use registry::*;`)
- Test: `romm-api/src/config/registry.rs` (`#[cfg(test)]`)

**Interfaces:**
- Produces: `ConfigSource` enum, `ConfigField<T>`, `ConfigSources`, `ConfigKey` (parse from dotted str), `set_config_key(&mut Config, &str, &str) -> Result<(), ConfigError>`, `env_var_for_key(&str) -> Option<&'static str>`

- [ ] **Step 1: Write failing tests**

Add to `romm-api/src/config/registry.rs`:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_dotted_keys() {
        assert_eq!(
            ConfigKey::parse("save_sync.device_id").unwrap(),
            ConfigKey::SaveSyncDeviceId
        );
        assert_eq!(
            ConfigKey::parse("roms_layout.platform_dirs.7").unwrap(),
            ConfigKey::RomsPlatformDir(7)
        );
    }

    #[test]
    fn set_and_get_extras_bool() {
        let mut cfg = minimal_config();
        set_config_key(&mut cfg, "extras_defaults.include_cover", "false").unwrap();
        assert!(!cfg.extras_defaults.include_cover);
    }

    fn minimal_config() -> Config {
        Config {
            base_url: "http://localhost".into(),
            download_dir: "/tmp/roms".into(),
            use_https: false,
            auth: None,
            extras_defaults: ExtrasDefaults::default(),
            save_sync: SaveSyncConfig::default(),
            roms_layout: RomsLayoutConfig::default(),
            theme: default_theme_id(),
            tui_layout: TuiLayoutConfig::default(),
        }
    }
}
```

- [ ] **Step 2: Run tests — expect FAIL**

```bash
cargo test -p romm-api config::registry -- --nocapture
```

- [ ] **Step 3: Implement registry module**

Define at minimum:

```rust
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ConfigSource {
    Default,
    File,
    Env(String),
    Keyring,
}

#[derive(Debug, Clone, Serialize)]
pub struct SourcedValue<T> {
    pub value: T,
    pub source: ConfigSource,
}

pub enum ConfigKey {
    BaseUrl,
    DownloadDir,
    UseHttps,
    Theme,
    ExtrasIncludeRelatedRoms,
    ExtrasIncludeCover,
    ExtrasIncludeManual,
    SaveSyncSaveDir,
    SaveSyncDeviceId,
    SaveSyncPlatformDir(u64),
    RomsPlatformDir(u64),
    // tui_layout fields as needed for show/set
}

impl ConfigKey {
    pub fn parse(s: &str) -> Result<Self, ConfigError> { /* ... */ }
    pub fn env_var(&self) -> Option<&'static str> { /* ... */ }
}
```

Implement `set_config_key` with bool parsing (`true`/`false`/`1`/`0`/`yes`/`no`) matching `roms.rs` tri-state helpers.

- [ ] **Step 4: Run tests — expect PASS**

```bash
cargo test -p romm-api config::registry -- --nocapture
```

- [ ] **Step 5: Commit**

```bash
git add romm-api/src/config/registry.rs romm-api/src/config.rs
git commit -m "feat(api): add config key registry with dotted-path set"
```

---

### Task 2: Source-aware config load (`romm-api`)

**Files:**
- Modify: `romm-api/src/config.rs` (`load_config`, new `load_config_with_sources`)
- Test: `romm-api/src/config.rs` (extend existing tests)

**Interfaces:**
- Produces: `pub fn load_config_with_sources() -> Result<(Config, ConfigSources), ConfigError>`
- Consumes: Task 1 registry types

- [ ] **Step 1: Write failing test**

Add to `config.rs` tests module:

```rust
#[test]
fn load_config_with_sources_marks_env_override() {
    with_env_lock(|| {
        std::env::set_var("API_BASE_URL", "http://env.test");
        std::env::set_var("ROMM_TEST_CONFIG_DIR", "/tmp/romm-test-empty");
        let (_cfg, sources) = load_config_with_sources().unwrap();
        assert_eq!(
            sources.base_url.source,
            ConfigSource::Env("API_BASE_URL".into())
        );
        assert_eq!(sources.base_url.value, "http://env.test");
    });
}
```

Use existing test helpers / `ROMM_TEST_CONFIG_DIR` pattern from `config.rs`.

- [ ] **Step 2: Run test — expect FAIL**

```bash
cargo test -p romm-api load_config_with_sources_marks_env_override -- --nocapture
```

- [ ] **Step 3: Refactor `load_config`**

Extract layer merge into functions that record `ConfigSource` per scalar field. Keep `load_config()` as:

```rust
pub fn load_config() -> Result<Config, ConfigError> {
    load_config_with_sources().map(|(c, _)| c)
}
```

Apply new env vars in this task:

| Env var | Field |
|---------|-------|
| `ROMM_SAVE_SYNC_SAVE_DIR` | `save_sync.save_dir` |
| `ROMM_SAVE_SYNC_DEVICE_ID` | `save_sync.device_id` |
| `ROMM_SAVE_SYNC_PLATFORM_DIR_{id}` | `save_sync.platform_dirs[id]` |
| `ROMM_ROMS_PLATFORM_DIR_{id}` | `roms_layout.platform_dirs[id]` |
| `ROMM_EXTRAS_INCLUDE_RELATED_ROMS` | `extras_defaults.include_related_roms` |
| `ROMM_EXTRAS_INCLUDE_COVER` | `extras_defaults.include_cover` |
| `ROMM_EXTRAS_INCLUDE_MANUAL` | `extras_defaults.include_manual` |

Optional JSON merge: `ROMM_ROMS_LAYOUT_JSON`, `ROMM_SAVE_SYNC_PLATFORM_DIRS_JSON`.

- [ ] **Step 4: Run full romm-api config tests**

```bash
cargo test -p romm-api config -- --nocapture
```

- [ ] **Step 5: Commit**

```bash
git add romm-api/src/config.rs
git commit -m "feat(api): add source-aware config load and env field coverage"
```

---

### Task 3: Config reset helper (`romm-api`)

**Files:**
- Modify: `romm-api/src/config.rs`
- Test: `romm-api/src/config.rs`

**Interfaces:**
- Produces: `pub fn reset_user_config() -> Result<(), ConfigError>` — deletes config file, clears keyring entries (`API_TOKEN`, `API_PASSWORD`, `API_KEY`), mirrors TUI Settings reset.

- [ ] **Step 1: Write failing test** using `ROMM_TEST_CONFIG_DIR`, write a config file, call reset, assert file gone.

- [ ] **Step 2: Run test — expect FAIL**

```bash
cargo test -p romm-api reset_user_config -- --nocapture
```

- [ ] **Step 3: Implement** reusing keyring clear logic from TUI settings handler (`clear_keyring_secrets` pattern in `romm-api` if exists, else extract from `persist_user_config` inverse).

- [ ] **Step 4: Run test — expect PASS**

- [ ] **Step 5: Commit**

```bash
git commit -m "feat(api): add reset_user_config helper"
```

---

### Task 4: `romm-cli config` command

**Files:**
- Create: `romm-cli/src/commands/config.rs`
- Modify: `romm-cli/src/commands/mod.rs`
- Modify: `romm-cli/src/frontend/cli.rs`
- Modify: `romm-cli/src/main.rs` (if `config` must bypass normal client init for `path`/`show --file` — prefer loading config like other commands)
- Test: `romm-cli/tests/cli_config.rs`

**Interfaces:**
- Consumes: `load_config_with_sources`, `set_config_key`, `persist_user_config`, `reset_user_config`, `user_config_json_path`, `ConfigSources`

- [ ] **Step 1: Write failing integration test**

Create `romm-cli/tests/cli_config.rs`:

```rust
#[test]
fn config_show_sources_json_reports_env_base_url() {
    let temp = tempfile::tempdir().unwrap();
    std::env::set_var("ROMM_TEST_CONFIG_DIR", temp.path());
    std::env::set_var("API_BASE_URL", "http://from-env.test");
    let mut cmd = Command::cargo_bin("romm-cli").unwrap();
    cmd.args(["config", "show", "--sources", "--json"]);
    cmd.assert().success().stdout(predicate::str::contains("from-env.test"));
}
```

- [ ] **Step 2: Run test — expect FAIL**

```bash
cargo test -p romm-cli config_show_sources -- --nocapture
```

- [ ] **Step 3: Implement clap structure**

```rust
#[derive(Subcommand, Debug)]
pub enum ConfigAction {
    Show {
        #[arg(long)] file: bool,
        #[arg(long)] sources: bool,
        #[arg(long)] reveal_secrets: bool,
    },
    Set { key: String, value: String },
    EnvMap { key: Option<String> },
    Path,
    Reset { #[arg(long)] yes: bool },
}
```

`Show` behavior:
- default: effective config JSON/text (redacted)
- `--file`: raw on-disk JSON or error if missing
- `--sources`: use `SourcedValue` serialization

`Set`: load file config (or defaults), apply `set_config_key`, `persist_user_config`.

`Reset`: require `--yes`; call `reset_user_config`.

- [ ] **Step 4: Wire into `Commands` enum** as `Config(config::ConfigCommand)`.

- [ ] **Step 5: Run tests — expect PASS**

```bash
cargo test -p romm-cli cli_config -- --nocapture
```

- [ ] **Step 6: Commit**

```bash
git add romm-cli/src/commands/config.rs romm-cli/src/commands/mod.rs romm-cli/src/frontend/cli.rs romm-cli/tests/cli_config.rs
git commit -m "feat(cli): add config show/set/env-map/reset commands"
```

---

### Task 5: Core saves helpers (`romm-api`)

**Files:**
- Create: `romm-api/src/core/saves.rs`
- Modify: `romm-api/src/core/mod.rs`
- Test: `romm-api/src/core/saves.rs`

**Interfaces:**
- Produces:
  - `pub async fn list_saves(client: &RommClient, filter: SaveListFilter) -> Result<Vec<SaveSchema>, RommError>`
  - `pub async fn download_save_to_path(client: &RommClient, save_id: u64, dest: &Path) -> Result<PathBuf, RommError>`
  - `pub async fn upload_save_for_rom(client: &RommClient, rom_id: u64, path: &Path, opts: SaveUploadOptions<'_>) -> Result<SaveSchema, RommError>`

Wrap existing `ListSaves`, `GetSave`, `client.download_save_content`, `client.upload_save_file_with_options`.

- [ ] **Step 1: Write unit test** for `safe_save_destination` path helper (unique filename logic — reuse TUI pattern or extract to core).

- [ ] **Step 2: Implement helpers** (thin wrappers, no new HTTP).

- [ ] **Step 3: Run tests**

```bash
cargo test -p romm-api core::saves -- --nocapture
```

- [ ] **Step 4: Commit**

```bash
git commit -m "feat(api): add core save list/download/upload helpers"
```

---

### Task 6: `romm-cli saves` command

**Files:**
- Create: `romm-cli/src/commands/saves.rs`
- Modify: `romm-cli/src/commands/mod.rs`, `romm-cli/src/frontend/cli.rs`
- Test: `romm-cli/tests/cli_saves.rs`

**Interfaces:**
- Consumes: Task 5 core helpers, `resolve_console_save_dir`, `resolved_save_dir`

- [ ] **Step 1: Write httpmock integration test**

Mock:
- `GET /api/saves?rom_id=42` → JSON array
- `GET /api/saves/9/content` → bytes
- `POST /api/saves` → 201 + schema

Test:
```bash
romm-cli saves list --rom-id 42 --json
romm-cli saves download 9 --output /tmp/out.sav
```

- [ ] **Step 2: Run test — expect FAIL**

```bash
cargo test -p romm-cli cli_saves -- --nocapture
```

- [ ] **Step 3: Implement subcommands** `list`, `get`, `download`, `upload` with `--json` and human tables via `CliPresentation`.

- [ ] **Step 4: Run tests — expect PASS**

- [ ] **Step 5: Commit**

```bash
git commit -m "feat(cli): add saves list/get/download/upload commands"
```

---

### Task 7: Core collections helpers (`romm-api`)

**Files:**
- Create: `romm-api/src/core/collections.rs`
- Modify: `romm-api/src/core/mod.rs`
- Test: `romm-api/src/core/collections.rs`

**Interfaces:**
- Produces:
  - `pub enum CollectionKind { Manual, Smart, Virtual, All }`
  - `pub async fn list_collections(client, kind) -> Result<Vec<Collection>, RommError>`
  - `pub async fn get_collection(client, kind, id) -> Result<Value, RommError>`
  - `pub async fn delete_collection(client, kind, id) -> Result<Value, RommError>`

Reuse `merge_all_collection_sources` from `endpoints/collections.rs`.

- [ ] **Step 1: Unit test** `CollectionKind` parse from CLI strings.

- [ ] **Step 2: Implement**

- [ ] **Step 3: Commit**

```bash
git commit -m "feat(api): add core collections list/get/delete helpers"
```

---

### Task 8: `romm-cli collections` command

**Files:**
- Create: `romm-cli/src/commands/collections.rs`
- Modify: `romm-cli/src/commands/mod.rs`, `romm-cli/src/frontend/cli.rs`
- Test: `romm-cli/tests/cli_collections.rs`

- [ ] **Step 1: httpmock tests** for list all (3 endpoints), get manual, delete smart with `--yes`.

- [ ] **Step 2: Implement** with table output + `--json`.

- [ ] **Step 3: Commit**

```bash
git commit -m "feat(cli): add collections list/get/delete commands"
```

---

### Task 9: Documentation

**Files:**
- Modify: `docs/cli.md`
- Modify: `docs/api.md` (env var table)
- Modify: `docs/json-output.md`

- [ ] **Step 1: Add `config`, `saves`, `collections` sections to `cli.md`** with examples mirroring metadata-editing style.

- [ ] **Step 2: Extend `api.md` env table** with all new `ROMM_*` vars from Task 2.

- [ ] **Step 3: Add JSON schemas** to `json-output.md` for:
  - `config show --sources`
  - `saves list`
  - `collections list`

- [ ] **Step 4: Commit**

```bash
git commit -m "docs: document config, saves, and collections CLI commands"
```

---

### Task 10: Final verification

- [ ] **Step 1: Full workspace checks**

```bash
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test
```

- [ ] **Step 2: Manual smoke**

```bash
romm-cli config path
romm-cli config env-map save_sync.device_id
romm-cli config show --sources --json
romm-cli collections list --json
romm-cli saves list --rom-id 1 --json   # against real server if available
```

- [ ] **Step 3: Update changelogs**

Add entries under `[Unreleased]` in `romm-api/CHANGELOG.md` and `romm-cli/CHANGELOG.md`.

- [ ] **Step 4: Commit + push**

```bash
git add -A
git commit -m "chore: changelog entries for CLI/TUI parity Phase 1"
git push -u origin cursor/cli-tui-parity-phase1-ffca
```

---

## Plan self-review

| Spec requirement | Task |
|------------------|------|
| Config D registry | 1, 2 |
| Env coverage all fields | 2 |
| `config show/set/sources/env-map/reset` | 4 |
| `saves` CLI ⊇ TUI | 5, 6 |
| `collections` CLI ⊇ TUI | 7, 8 |
| `--json` + docs | 9 |
| No play/launch | Global Constraints |
| Shared core in romm-api | 5, 7 |
| Phase 1c global flags | Out of scope (noted in spec) |
| Phase 2 TUI catch-up | Out of scope |

No placeholders remain in task steps.

---

## Follow-up plans (not in this file)

- **Phase 1c:** `docs/superpowers/plans/2026-08-12-config-global-flags.md` (optional)
- **Phase 2:** TUI props/notes/delete/find/sync UX
- **Phase 3:** firmware, states, play-sessions
