# Contributing

Thanks for helping improve romm-cli.

## Before you open a PR

```bash
git clone https://github.com/patricksmill/romm-cli
cd romm-cli
cargo test --workspace
```

Required checks (match CI):

```bash
cargo fmt
cargo fmt --check
cargo clippy --all-targets --all-features -- -D warnings
cargo clippy --all-targets --no-default-features -- -D warnings
```

## Commits and changelogs

Use [Conventional Commits](https://www.conventionalcommits.org/). Scope by crate so Release Please routes changelog entries:

| Scope | Crate |
|-------|-------|
| `api`, `config`, `download`, `client` | `romm-api` |
| `cli`, `completions`, `auth` | `romm-cli` |
| `tui`, `settings` | `romm-tui` |

See [docs/releases.md](docs/releases.md) for versioning and publish order (`romm-api` first).

## Where to change code

| Area | Start here |
|------|------------|
| HTTP client, downloads, config | [docs/api.md](docs/api.md), [docs/architecture.md](docs/architecture.md) |
| CLI commands, `--json` | [docs/cli.md](docs/cli.md) |
| TUI screens, keybindings | [docs/tui.md](docs/tui.md) |
| Rust conventions, gaps | [docs/rust-guidelines.md](docs/rust-guidelines.md) |

## Please don't

- Commit generated shell completion scripts (use `romm-cli completions <shell>` at install time).
- Include API tokens, keyring dumps, or full `config.json` in issues or PRs.
- Open drive-by refactors unrelated to the issue you are fixing.

## Questions

- Auth / keyring / Docker: [docs/troubleshooting-auth.md](docs/troubleshooting-auth.md)
- RomM server bugs: [rommapp/romm](https://github.com/rommapp/romm/issues)
