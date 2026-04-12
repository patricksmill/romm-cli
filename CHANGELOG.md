# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.18.0](https://github.com/patricksmill/romm-cli/compare/v0.17.0...v0.18.0) (2026-04-12)


### Features

* **client:** add unauthenticated JSON request method ([bb904f0](https://github.com/patricksmill/romm-cli/commit/bb904f061e72995a1ab7ed1900a3b527f5aaedf0))
* **config:** enhance keyring integration and update config handling ([4c90381](https://github.com/patricksmill/romm-cli/commit/4c90381f71bb3ea0bd4163d50edfafa22e014cdd))
* **tui:** implement global error handling and integrate wiremock for testing ([883be87](https://github.com/patricksmill/romm-cli/commit/883be8702b7e43a70eaf17f38f60a4e6864e92ef))


### Bug Fixes

* ran cargo fmt ([ffb4629](https://github.com/patricksmill/romm-cli/commit/ffb4629b1085914cde3b963d8657b553fb31f080))
* **tests:** update assertion for HTTPS configuration in tests ([24c43e6](https://github.com/patricksmill/romm-cli/commit/24c43e6a27abbebf47d55103d03186767fdc9fbd))

## [0.17.0](https://github.com/patricksmill/romm-cli/compare/v0.16.0...v0.17.0) (2026-04-11)


### Features

* **init:** add non-interactive setup flags for automated configuration ([4bfc8f3](https://github.com/patricksmill/romm-cli/commit/4bfc8f3c54cad769669a67a615d8c900d81a62a5))

## [0.16.0](https://github.com/patricksmill/romm-cli/compare/v0.15.2...v0.16.0) (2026-03-31)


### Features

* **init:** add non-interactive flags for automated setup and browser API token import (`--url`, `--token-file`, `--check`)
* **settings:** allow user to chang base url, enable/disable https, and edit password in settings in tui ([c49c30e](https://github.com/patricksmill/romm-cli/commit/c49c30e40926cca28093aa968612c4bd5f647d80))


### Bug Fixes

* **settings:** fix formatting ([cfb61be](https://github.com/patricksmill/romm-cli/commit/cfb61bee8640711c6c4fba30471a86416678e1e8))
* **settings:** fix formatting ([90ec22d](https://github.com/patricksmill/romm-cli/commit/90ec22dffec6bcae18f4f4f5a29092b371e5fad1))

## [0.15.2](https://github.com/patricksmill/romm-cli/compare/v0.15.1...v0.15.2) (2026-03-30)


### Bug Fixes

* **CI:** update CI to use native rust release-type ([4db2dab](https://github.com/patricksmill/romm-cli/commit/4db2dab1a7fb71459d252dba689048bde349a637))

## [0.15.1](https://github.com/patricksmill/romm-cli/compare/v0.15.0...v0.15.1) (2026-03-30)


### Bug Fixes

* **cargo:** update cargo.lock ([f011b40](https://github.com/patricksmill/romm-cli/commit/f011b40f311571a39d168715b43b6692eb7e7b92))

## [0.15.0](https://github.com/patricksmill/romm-cli/compare/v0.14.0...v0.15.0) (2026-03-30)


### Features

* overhaul CLI with Resource-Action subcommands and aliases ([90b5c43](https://github.com/patricksmill/romm-cli/commit/90b5c43225a00e91088118ea48427800d9259996))

## [0.14.0](https://github.com/patricksmill/romm-cli/compare/v0.13.1...v0.14.0) (2026-03-30)


### Features

* **cli:** implement ROM and platform API endpoints, services, and CLI commands ([9026a5f](https://github.com/patricksmill/romm-cli/commit/9026a5fd25593e5827ed48422cc79ea8cf6514a7))
* **tui:** add startup steps to tui ([7488831](https://github.com/patricksmill/romm-cli/commit/7488831e78deb61e1b5c053e11e277d242763fac))


### Bug Fixes

* fix clippy warnings ([46a7561](https://github.com/patricksmill/romm-cli/commit/46a75613d736dfa95872424a506384863cf27c9a))
* **setup:** formatting ([4569c3e](https://github.com/patricksmill/romm-cli/commit/4569c3ec33a7b5d874888487a631cf228044d6cf))

## [0.13.1](https://github.com/patricksmill/romm-cli/compare/v0.13.0...v0.13.1) (2026-03-29)


### Bug Fixes

* **openapi_sync:** fix default url scheme ([53ec673](https://github.com/patricksmill/romm-cli/commit/53ec6739a062f0669a2e9f5446e4fad390fe0556))

## [0.13.0](https://github.com/patricksmill/romm-cli/compare/v0.12.0...v0.13.0) (2026-03-28)


### Features

* **docs:** update README and client code to support optional OpenAPI base URL and local path configuration ([b74d3af](https://github.com/patricksmill/romm-cli/commit/b74d3af1d63e88c1c41d1eb674673113f876765d))

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
