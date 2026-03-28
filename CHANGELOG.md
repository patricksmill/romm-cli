# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.12.0](https://github.com/patricksmill/romm-cli/compare/v0.11.1...v0.12.0) (2026-03-28)


### Features

* **client:** enhance OpenAPI fetching with scheme fallback and alternate paths; add local openapi.json support in sync process ([7593925](https://github.com/patricksmill/romm-cli/commit/7593925825bb9e5e1a59fe1d90e0d7ac70b070b0))

## [0.11.1](https://github.com/patricksmill/romm-cli/compare/v0.11.0...v0.11.1) (2026-03-28)


### Bug Fixes

* **main:** fix autoupdater ([09cab5f](https://github.com/patricksmill/romm-cli/commit/09cab5fba1854617be3efcbaba5e84ccf74960fb))

## [0.11.0](https://github.com/patricksmill/romm-cli/compare/v0.10.1...v0.11.0) (2026-03-28)


### Features

* **main:** fetch openapi.json from Romm server when not present; update automatically on version check. Add server version in settings ([a598295](https://github.com/patricksmill/romm-cli/commit/a5982950eec31c63025483e5241e2194ed1c9a3f))


### Bug Fixes

* **main:** formatting ([0b76924](https://github.com/patricksmill/romm-cli/commit/0b769249de1a0ccab2cb36f39031d77c38045f1b))
* **main:** Update cargo files ([7f6467a](https://github.com/patricksmill/romm-cli/commit/7f6467ab51a7bec52fa796fd1232e43c417e62f4))
* **main:** update readme ([26d7b0a](https://github.com/patricksmill/romm-cli/commit/26d7b0ab273fb19dff172ee4efec15bdd0719c3a))

## [0.10.1](https://github.com/patricksmill/romm-cli/compare/v0.10.0...v0.10.1) (2026-03-28)


### Bug Fixes

* **main:** fix artifact path of romm-tui in release-please.yml ([16964c2](https://github.com/patricksmill/romm-cli/commit/16964c24b94c85faa88cec985fb123c8dfce3741))

## [0.10.0](https://github.com/patricksmill/romm-cli/compare/v0.9.0...v0.10.0) (2026-03-28)


### Features

* Add `init` command for interactive user configuration, including secure credential storage via OS keyring. ([7432a6b](https://github.com/patricksmill/romm-cli/commit/7432a6b98ce7e80d910d72419784b69d8e275cb5))
* Add a self-update command to the CLI, including necessary dependencies and integration into the command structure. ([476bfdf](https://github.com/patricksmill/romm-cli/commit/476bfdff8645074ced5448794ecae22c6b1fabd6))
* Implement initial CLI application structure with TUI, API client, and download command. ([0a041dc](https://github.com/patricksmill/romm-cli/commit/0a041dcacc647fe50e0a4b16ea049e334f623a8a))
* Implement TUI and CLI frontends, add project metadata, and establish CI/CD workflows. ([3013a41](https://github.com/patricksmill/romm-cli/commit/3013a4146493e6e7faccdd7567a969fee66cebee))

## [0.6.0](https://github.com/patricksmill/romm-cli/compare/v0.5.0...v0.6.0) (2026-03-27)


### Features

* Add `init` command for interactive user configuration, including secure credential storage via OS keyring. ([7432a6b](https://github.com/patricksmill/romm-cli/commit/7432a6b98ce7e80d910d72419784b69d8e275cb5))
* Add a self-update command to the CLI, including necessary dependencies and integration into the command structure. ([476bfdf](https://github.com/patricksmill/romm-cli/commit/476bfdff8645074ced5448794ecae22c6b1fabd6))
* Implement initial CLI application structure with TUI, API client, and download command. ([0a041dc](https://github.com/patricksmill/romm-cli/commit/0a041dcacc647fe50e0a4b16ea049e334f623a8a))

## [0.4.0](https://github.com/patricksmill/romm-cli/compare/v0.3.0...v0.4.0) (2026-03-27)


### Features

* Add `init` command for interactive user configuration, including secure credential storage via OS keyring. ([7432a6b](https://github.com/patricksmill/romm-cli/commit/7432a6b98ce7e80d910d72419784b69d8e275cb5))
* Implement initial CLI application structure with TUI, API client, and download command. ([0a041dc](https://github.com/patricksmill/romm-cli/commit/0a041dcacc647fe50e0a4b16ea049e334f623a8a))

## [Unreleased]

## [0.2.0] - 2026-03-20

### Added

- Interactive `romm-cli init` (alias `setup`) to write user config (`API_BASE_URL` and optional auth) under the OS config directory
- Layered environment loading: cwd `.env` then user `romm-cli/.env` (project values override user defaults for the same keys)
- `romm-tui` binary to launch the TUI without a subcommand; `ROMM_VERBOSE=1` enables HTTP logging
- `romm-cli init --print-path` and `--force`

### Changed

- Release archives now ship both `romm-cli` and `romm-tui` (per platform)

## [0.1.0] - 2026-03-20

### Added

- Initial release of romm-cli: Rust CLI + TUI client for the ROMM API

[Unreleased]: https://github.com/patricksmill/romm-cli/compare/v0.2.0...HEAD
[0.2.0]: https://github.com/patricksmill/romm-cli/releases/tag/v0.2.0
[0.1.0]: https://github.com/patricksmill/romm-cli/releases/tag/v0.1.0
