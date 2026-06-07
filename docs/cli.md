# romm-cli

[![Crates.io](https://img.shields.io/crates/v/romm-cli.svg)](https://crates.io/crates/romm-cli)
[![Docs.rs](https://docs.rs/romm-cli/badge.svg)](https://docs.rs/romm-cli)

Command-line interface for scripting and automation against a [RomM](https://github.com/romm-retro/romm) server. Depends on [`romm-api`](api.md) for HTTP, config, and domain logic.

**Crate path:** `romm-cli/` in the workspace. The `romm-cli` binary is the primary user-facing tool.

---

## Install

### Cargo

```bash
cargo install romm-cli
```

The TUI feature is enabled by default (`romm-cli tui`). For a CLI-only build:

```bash
cargo install romm-cli --no-default-features
```

### Binary releases

Prebuilt archives for Windows, Linux, and macOS are on the [Releases page](https://github.com/patricksmill/romm-cli/releases). Each archive includes:

- `romm-cli` — full command-line tool, including `romm-cli tui`
- `romm-tui` — shortcut executable that opens the TUI directly

---

## Configuration

Run the setup wizard:

```bash
romm-cli init
```

Full details (environment variables, precedence, custom paths, keyring): **[romm-api configuration](api.md#configuration)**.

**Auth troubleshooting:** [troubleshooting-auth.md](troubleshooting-auth.md).

---

## Usage

Run `romm-cli --help` or `romm-cli <command> --help` for the full flag list. Many subcommands have short aliases (e.g. `setup` for `init`, `p` for `platforms`, `r` for `roms`, `dl` for `download`).

### Common commands

```bash
# List platforms
romm-cli platforms

# Search and print JSON
romm-cli roms list --search-term "zelda" --json

# Upload a ROM, then optionally rescan the library on the server
romm-cli roms upload --platform <slug-or-name> path/to/rom.bin --scan
romm-cli roms upload --platform <slug-or-name> ./folder --scan --wait

# Download covers, manuals, and sibling updates/DLC for one game
romm-cli download extras <rom-id>

# Trigger a full library scan; optional --wait
romm-cli scan
romm-cli scan --wait --wait-timeout-secs 3600

# Save sync (RomM 4.9.0-alpha.2+)
romm-cli sync device register --name "My Handheld" --sync-mode api
romm-cli sync plan --device-id <device-id> --manifest ./sync-manifest.json
romm-cli sync run --device-id <device-id> --manifest ./sync-manifest.json

# Self-update
romm-cli update

# Cache utilities
romm-cli cache path
romm-cli cache info
romm-cli cache clear

# Rotate credentials without re-entering ROM path/base URL
romm-cli auth status
romm-cli auth login --token-file ~/.romm-token
romm-cli auth logout
```

On interactive startup, `romm-cli` checks for newer releases and can prompt to update. Disable with `ROMM_CHECK_UPDATES=false` (see [api.md](api.md#environment-variables)).

### TUI subcommand

```bash
romm-cli tui
```

For the standalone TUI binary and interactive features, see **[TUI documentation](tui.md)**.

### Post-upload library scan

After chunked uploads, RomM needs a **library scan** before new games appear in search and the TUI. See [scan-after-upload.md](scan-after-upload.md).

### Save sync

Save sync uses RomM sync endpoints from the `4.9.0-alpha.2` pre-release onward. See [save-sync.md](save-sync.md) for manifest schema and conflict handling.

---

## JSON output

Commands that support `--json` emit stable, machine-readable JSON on stdout. Progress and status go to stderr. See [json-output.md](json-output.md) for field reference and versioning policy.

---

## Exit codes

| Code | Meaning | Typical cause |
|------|---------|---------------|
| 0 | Success | Command completed (including user-cancelled downloads) |
| 1 | General failure | Unexpected error, app-level validation |
| 2 | Usage error | Invalid flags or arguments (clap) |
| 3 | Config / auth | Missing `API_BASE_URL`, bad credentials |
| 4 | API / network | Server errors, connection failures, download failures |

Example:

```bash
romm-cli platforms
case $? in
  0) echo ok ;;
  3) echo "fix config or auth" ;;
  4) echo "API or network issue" ;;
  *) echo "other failure" ;;
esac
```

Errors are printed to stderr with actionable hints (for example, suggesting `romm-cli init` or `romm-cli auth`).

---

## Shell completions

Tab completion is available for bash, zsh, fish, PowerShell, and Elvish. Scripts live in [`romm-cli/completions/`](../romm-cli/completions/) and can be regenerated at runtime:

```bash
romm-cli completions bash
romm-cli completions zsh
romm-cli completions fish
romm-cli completions powershell
```

**Bash** — user install:

```bash
mkdir -p ~/.local/share/bash-completion/completions
romm-cli completions bash > ~/.local/share/bash-completion/completions/romm-cli
```

**Zsh** — clap generates `_romm-cli` (underscore prefix required):

```bash
mkdir -p ~/.zfunc
cp romm-cli/completions/_romm-cli ~/.zfunc/_romm-cli
# Add to ~/.zshrc before compinit:
# fpath=(~/.zfunc $fpath); autoload -Uz compinit; compinit
```

**Fish**:

```bash
romm-cli completions fish > ~/.config/fish/completions/romm-cli.fish
```

**PowerShell**:

```powershell
romm-cli completions powershell > $PROFILE.CurrentUserCurrentHost
```

Start a new shell session after installing completions.

---

## Library use

The `romm-cli` crate re-exports `romm_api` modules for backward compatibility with existing crates.io consumers:

```rust
use romm_cli::client::RommClient;
use romm_cli::config::load_config;
```

New embedders should depend on [`romm-api`](api.md) directly.

---

## Related documentation

- [romm-api](api.md) — configuration, HTTP client, errors
- [romm-tui](tui.md) — terminal UI
- [Save sync](save-sync.md)
- [Post-upload library scan](scan-after-upload.md)
- [JSON output](json-output.md)
