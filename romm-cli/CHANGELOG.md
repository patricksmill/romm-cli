# Changelog

All notable changes to **romm-cli** are documented in this file.

Entries before the workspace split (1.0.0) are filtered from the unified monolith history by conventional-commit scope. Shared library changes appear in [romm-api/CHANGELOG.md](../romm-api/CHANGELOG.md). Terminal UI changes appear in [romm-tui/CHANGELOG.md](../romm-tui/CHANGELOG.md).

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [1.2.0](https://github.com/patricksmill/romm-cli/compare/romm-cli-v1.1.0...romm-cli-v1.2.0) (2026-07-03)


### Features

* **achievements:** integrate RetroAchievements into game detail view ([7a80ed6](https://github.com/patricksmill/romm-cli/commit/7a80ed63ce35b5d5003dd3dbdad7e9c935f8fe16))
* add ROM metadata search, match, and edit ([ba5031e](https://github.com/patricksmill/romm-cli/commit/ba5031e44c3c5cf9cf00e4121b6b283bb2642915))
* **metadata:** add ROM metadata editing functionality to CLI and TUI ([d041ffb](https://github.com/patricksmill/romm-cli/commit/d041ffb0ac57413b533bab9b2a4627c7eaa793c0))
* **metadata:** enhance error handling and metadata application in TUI and API ([9ac601d](https://github.com/patricksmill/romm-cli/commit/9ac601d45e873cdfc4743de2bbe121107fd3c7ac))

## [1.1.0](https://github.com/patricksmill/romm-cli/compare/romm-cli-v1.0.0...romm-cli-v1.1.0) (2026-06-21)


### Features

* **cli:** generate shell completions at runtime only ([af08376](https://github.com/patricksmill/romm-cli/commit/af08376f09e5c8172e69aadfd55345832e3293cb))

## [1.0.0](https://github.com/patricksmill/romm-cli/compare/romm-cli-v0.40.0...romm-cli-v1.0.0) (2026-06-07)

### ⚠ BREAKING CHANGES
* **romm-cli:** removed the `tui` feature and `romm-cli tui` subcommand; the terminal UI ships only as the separate `romm-tui` crate.

### Features
* **romm-cli:** fresh 1.0.0 release after workspace split and TUI decoupling

## [0.39.0](https://github.com/patricksmill/romm-cli/compare/v0.38.0...v0.39.0) (2026-06-06)

### Features
* **cli:** close Gap 7 with CliPresentation and JSON-safe output ([4dcfdad](https://github.com/patricksmill/romm-cli/commit/4dcfdad39d8bf280e1d70b3fa6a2a5d9d54b2781))
* **errors:** implement distinct exit codes for improved error handling ([6d521e7](https://github.com/patricksmill/romm-cli/commit/6d521e7b1f11299c7381385ea955f04dbf21a32a))

## [0.38.0](https://github.com/patricksmill/romm-cli/compare/v0.37.0...v0.38.0) (2026-06-06)

### Features
* **completions:** add shell completion scripts and build script ([f54d761](https://github.com/patricksmill/romm-cli/commit/f54d76119e99694c4ce41797f3b9488a3b4843f8))
* **completions:** enhance shell completion support and CI integration ([9d2db49](https://github.com/patricksmill/romm-cli/commit/9d2db490891ddeca1201c485893f7895da1fb988))

## [0.34.0](https://github.com/patricksmill/romm-cli/compare/v0.33.1...v0.34.0) (2026-05-22)

### Features
* **update:** enhance update process with detailed outcomes and options ([2e11bfc](https://github.com/patricksmill/romm-cli/commit/2e11bfcb894382d8854587ca8751ad4b291e67c3))

## [0.32.0](https://github.com/patricksmill/romm-cli/compare/v0.31.0...v0.32.0) (2026-05-07)

### Features
* add download extras command to fetch covers, manuals, and DLC for games ([11fb0f4](https://github.com/patricksmill/romm-cli/commit/11fb0f4346c755ed44937b02d208cd7a2926b178))

## [0.30.1](https://github.com/patricksmill/romm-cli/compare/v0.30.0...v0.30.1) (2026-05-01)

### Bug Fixes
* **update:** fix redundant update prompt after skipping cli update when running romm-cli tui ([6872855](https://github.com/patricksmill/romm-cli/commit/6872855a5ea05985547e943a63c65bb79dbbaabb))

## [0.29.0](https://github.com/patricksmill/romm-cli/compare/v0.28.0...v0.29.0) (2026-05-01)

### Features
* **update:** enhance update mechanism with dynamic binary name resolution and add tests ([947db3f](https://github.com/patricksmill/romm-cli/commit/947db3f02176359b0be13a924e77b393391d999d))
* **update:** improve binary name extraction from path for better compatibility ([deae08a](https://github.com/patricksmill/romm-cli/commit/deae08abb0cf5863c65149647916aa1bb8724810))

## [0.28.0](https://github.com/patricksmill/romm-cli/compare/v0.27.0...v0.28.0) (2026-04-29)

### Features
* **auth:** add authentication command group to manage credentials ([72b9182](https://github.com/patricksmill/romm-cli/commit/72b9182b439f81e22fc58f67632a3486036a6695))

## [0.26.0](https://github.com/patricksmill/romm-cli/compare/v0.25.0...v0.26.0) (2026-04-27)

### Features
* **update:** add startup update checks and interactive prompt ([e601631](https://github.com/patricksmill/romm-cli/commit/e601631598ed9cf47f4d1abbc51d4fa9ec3d3706))

## [0.25.0](https://github.com/patricksmill/romm-cli/compare/v0.24.0...v0.25.0) (2026-04-21)

### Bug Fixes
* **cli:** fix testing to reflect using --platform instead of --platform-id ([f6ef5b4](https://github.com/patricksmill/romm-cli/commit/f6ef5b47618c2b66f442d4d552559c0428842b2b))

## [0.22.0](https://github.com/patricksmill/romm-cli/compare/v0.21.0...v0.22.0) (2026-04-19)

### Features
* **roms:** implement ROM file upload functionality ([b418c15](https://github.com/patricksmill/romm-cli/commit/b418c1556c894ae1dc9dbdbf23d84dcb8994162f))
* **roms:** implement ROM file upload functionality ([15fc288](https://github.com/patricksmill/romm-cli/commit/15fc288c1394940d46c417b0d9e9c68d329700f5))
* **scan:** add library scan functionality post-ROM upload ([e71855d](https://github.com/patricksmill/romm-cli/commit/e71855d0c8ebeffe89faf67c5f630b7333a42891))
* **scan:** enhance library scan functionality with cache management ([349e316](https://github.com/patricksmill/romm-cli/commit/349e316b1e2c6b0f5035c103de6316aa31db4163))

## [0.21.0](https://github.com/patricksmill/romm-cli/compare/v0.20.0...v0.21.0) (2026-04-19)

### Features
* **cache:** add cache management commands and functionality ([3c99a05](https://github.com/patricksmill/romm-cli/commit/3c99a05c23123326f9f9dfc53fefdfbba32ae58c))

## [0.17.0](https://github.com/patricksmill/romm-cli/compare/v0.16.0...v0.17.0) (2026-04-11)

### Features
* **init:** add non-interactive setup flags for automated configuration ([4bfc8f3](https://github.com/patricksmill/romm-cli/commit/4bfc8f3c54cad769669a67a615d8c900d81a62a5))

## [0.16.0](https://github.com/patricksmill/romm-cli/compare/v0.15.2...v0.16.0) (2026-03-31)

### Features
* **init:** add non-interactive flags for automated setup and browser API token import (`--url`, `--token-file`, `--check`)

## [0.15.0](https://github.com/patricksmill/romm-cli/compare/v0.14.0...v0.15.0) (2026-03-30)

### Features
* overhaul CLI with Resource-Action subcommands and aliases ([90b5c43](https://github.com/patricksmill/romm-cli/commit/90b5c43225a00e91088118ea48427800d9259996))

## [0.14.0](https://github.com/patricksmill/romm-cli/compare/v0.13.1...v0.14.0) (2026-03-30)

### Features
* **cli:** implement ROM and platform API endpoints, services, and CLI commands ([9026a5f](https://github.com/patricksmill/romm-cli/commit/9026a5fd25593e5827ed48422cc79ea8cf6514a7))

## [0.10.0](https://github.com/patricksmill/romm-cli/compare/v0.9.0...v0.10.0) (2026-03-28)

### Features
* Add `init` command for interactive user configuration, including secure credential storage via OS keyring. ([7432a6b](https://github.com/patricksmill/romm-cli/commit/7432a6b98ce7e80d910d72419784b69d8e275cb5))
* Add a self-update command to the CLI, including necessary dependencies and integration into the command structure. ([476bfdf](https://github.com/patricksmill/romm-cli/commit/476bfdff8645074ced5448794ecae22c6b1fabd6))

## [0.6.0](https://github.com/patricksmill/romm-cli/compare/v0.5.0...v0.6.0) (2026-03-27)

### Features
* Add `init` command for interactive user configuration, including secure credential storage via OS keyring. ([7432a6b](https://github.com/patricksmill/romm-cli/commit/7432a6b98ce7e80d910d72419784b69d8e275cb5))
* Add a self-update command to the CLI, including necessary dependencies and integration into the command structure. ([476bfdf](https://github.com/patricksmill/romm-cli/commit/476bfdff8645074ced5448794ecae22c6b1fabd6))

## [0.4.0](https://github.com/patricksmill/romm-cli/compare/v0.3.0...v0.4.0) (2026-03-27)

### Features
* Add `init` command for interactive user configuration, including secure credential storage via OS keyring. ([7432a6b](https://github.com/patricksmill/romm-cli/commit/7432a6b98ce7e80d910d72419784b69d8e275cb5))

## [0.2.0] - 2026-03-20

### Added
- Interactive `romm-cli init` (alias `setup`) to write user config (`API_BASE_URL` and optional auth) under the OS config directory
- `romm-cli init --print-path` and `--force`
