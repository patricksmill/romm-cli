# Changelog

All notable changes to **romm-api** are documented in this file.

Entries before the workspace split (1.0.0) are filtered from the unified monolith history by conventional-commit scope. Frontend-only changes appear in [romm-cli/CHANGELOG.md](../romm-cli/CHANGELOG.md) or [romm-tui/CHANGELOG.md](../romm-tui/CHANGELOG.md).

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Features

- **config:** config key registry, `load_config_with_sources`, env overrides for save sync / extras / platform dirs, `redact_config`, `reset_user_config` alias
- **core:** save and collection helpers for CLI/TUI parity

## [1.3.0](https://github.com/patricksmill/romm-cli/compare/romm-api-v1.2.0...romm-api-v1.3.0) (2026-07-20)


### Features

* **tui:** enhance ROM caching and loading behavior ([02fc454](https://github.com/patricksmill/romm-cli/commit/02fc45408ce1a9d5a23475901fcc0f550344b385))


### Bug Fixes

* **api:** reject empty OpenAPI bodies and fall back in TUI ([#60](https://github.com/patricksmill/romm-cli/issues/60)) ([710e072](https://github.com/patricksmill/romm-cli/commit/710e072a18222174203a381a9a07545753a87b9f))
* **tui:** resume partial ROM list loads after navigation ([#56](https://github.com/patricksmill/romm-cli/issues/56)) ([fcc5ac4](https://github.com/patricksmill/romm-cli/commit/fcc5ac4aa99c836a7ef67e10f42e8b59824664ef))

## [1.2.0](https://github.com/patricksmill/romm-cli/compare/romm-api-v1.1.0...romm-api-v1.2.0) (2026-07-03)


### Features

* **achievements:** integrate RetroAchievements into game detail view ([7a80ed6](https://github.com/patricksmill/romm-cli/commit/7a80ed63ce35b5d5003dd3dbdad7e9c935f8fe16))
* add ROM metadata search, match, and edit ([ba5031e](https://github.com/patricksmill/romm-cli/commit/ba5031e44c3c5cf9cf00e4121b6b283bb2642915))
* **game-detail:** tab-aware right panel and achievement navigation ([ad20caf](https://github.com/patricksmill/romm-cli/commit/ad20cafaa4001ee3f2411d2ea6a56039bbef6b92))
* **game-detail:** tabbed layout with context-aware right panel ([15370a2](https://github.com/patricksmill/romm-cli/commit/15370a2f4c44ea82818eaab15c151b12022a88a5))
* **metadata:** add ROM metadata editing functionality to CLI and TUI ([d041ffb](https://github.com/patricksmill/romm-cli/commit/d041ffb0ac57413b533bab9b2a4627c7eaa793c0))
* **metadata:** enhance error handling and metadata application in TUI and API ([9ac601d](https://github.com/patricksmill/romm-cli/commit/9ac601d45e873cdfc4743de2bbe121107fd3c7ac))

## [1.1.0](https://github.com/patricksmill/romm-cli/compare/romm-api-v1.0.0...romm-api-v1.1.0) (2026-06-21)


### Features

* **api:** export download_target_with_fallback for frontend reuse ([2364183](https://github.com/patricksmill/romm-cli/commit/2364183875c3ed4da475929d64b93970534f8853))

## [1.0.0](https://github.com/patricksmill/romm-cli/compare/romm-api-v0.40.0...romm-api-v1.0.0) (2026-06-07)

### Features

* **api:** fresh 1.0.0 release after workspace split; shared library crate for all frontends

## [0.40.0](https://github.com/patricksmill/romm-cli/compare/v0.39.0...v0.40.0) (2026-06-06)

### Features
* **download:** implement authentication header logic for download URLs ([9f80ffc](https://github.com/patricksmill/romm-cli/commit/9f80ffcfdf4b0470ce2192c3e8327cd4390e83ce))
* **sync:** add safe_download_file_name function to sanitize file names ([9f80ffc](https://github.com/patricksmill/romm-cli/commit/9f80ffcfdf4b0470ce2192c3e8327cd4390e83ce))

### Bug Fixes
* **paths:** enhance zip extraction to reject path traversal entries ([9f80ffc](https://github.com/patricksmill/romm-cli/commit/9f80ffcfdf4b0470ce2192c3e8327cd4390e83ce))

## [0.38.0](https://github.com/patricksmill/romm-cli/compare/v0.37.0...v0.38.0) (2026-06-06)

### Features
* **errors:** implement typed error handling across the codebase ([fcae759](https://github.com/patricksmill/romm-cli/commit/fcae75915d175462775c6a952bc36d15535366ed))

### Bug Fixes
* **config:** improve error handling in persist_user_config function ([920ee3c](https://github.com/patricksmill/romm-cli/commit/920ee3cab35cefd05b89903b18d6c9dcfeb340a5))

## [0.37.0](https://github.com/patricksmill/romm-cli/compare/v0.36.0...v0.37.0) (2026-05-28)

### Features
* **config:** add persisted TUI theme field ([737fd56](https://github.com/patricksmill/romm-cli/commit/737fd56f03925df26e5413c6f0f670043e982e20))
* **download:** implement download management system ([d8e5c36](https://github.com/patricksmill/romm-cli/commit/d8e5c365a03bd931abd567889a8ed56cd19813c9))

### Bug Fixes
* **download:** restore Platform import in test helpers ([d07505f](https://github.com/patricksmill/romm-cli/commit/d07505f8d456945a41d75dc157207d4d601380a9))

## [0.35.0](https://github.com/patricksmill/romm-cli/compare/v0.34.0...v0.35.0) (2026-05-24)

### Features
* **config:** implement custom console paths for ROM layout ([d360bca](https://github.com/patricksmill/romm-cli/commit/d360bca6de0ff24f3bad83301a4170049de958de))
* **config:** introduce customizable ROM layout options ([874d815](https://github.com/patricksmill/romm-cli/commit/874d81578d53e49ac2f3f3d3bd1ad182238f301a))
* **save-sync:** implement custom save paths for consoles ([6e2fcec](https://github.com/patricksmill/romm-cli/commit/6e2fcec52038cb9136ea8f894404ce585403ad2b))

## [Unreleased]

### Added
- **config/TUI:** Per-console custom save directory overrides (`save_sync.platform_dirs`), configured under **Settings → Saves → Save console paths** or in `config.json`.

### Changed
- **config:** Replace auto/manual ROM layout mode with optional per-console custom paths (`platform_dirs` only). Legacy `"mode"` in config is ignored on load and omitted on save. `ROMM_ROMS_LAYOUT` env var removed.

### Features
- **update:** add startup update checks and interactive update prompt with changelog shortcut (shared by CLI and TUI frontends)

## [0.34.0](https://github.com/patricksmill/romm-cli/compare/v0.33.1...v0.34.0) (2026-05-22)

### Features
* **update:** enhance update process with detailed outcomes and options ([2e11bfc](https://github.com/patricksmill/romm-cli/commit/2e11bfcb894382d8854587ca8751ad4b291e67c3))

## [0.33.1](https://github.com/patricksmill/romm-cli/compare/v0.33.0...v0.33.1) (2026-05-16)

### Bug Fixes
* **hash:** update hash computation to use finalize method ([beb23f4](https://github.com/patricksmill/romm-cli/commit/beb23f4cbde5f07f857a302155786ff4fbbf0064))
* **search:** fix games with the same name as games in other consoles not showing up ([1f8e717](https://github.com/patricksmill/romm-cli/commit/1f8e7172d865d2e4462210477d54c89f0f346ede))

## [0.33.0](https://github.com/patricksmill/romm-cli/compare/v0.32.0...v0.33.0) (2026-05-12)

### Features
* add device management and sync endpoints ([4431e4f](https://github.com/patricksmill/romm-cli/commit/4431e4fe6d1f99329b5ea4c74add9d8f99f70512))
* add OpenAPI parsing and endpoint registry for compatibility checks ([5b25da3](https://github.com/patricksmill/romm-cli/commit/5b25da3cf817a3937d394226d82bbf7f36fff52e))
* add save sync functionality to TUI and CLI ([fff28f4](https://github.com/patricksmill/romm-cli/commit/fff28f45dbeb33cec1a61da99a4371429758b448))
* implement save sync compatibility checks and enhance error handling for unsupported endpoints ([5b25da3](https://github.com/patricksmill/romm-cli/commit/5b25da3cf817a3937d394226d82bbf7f36fff52e))

## [0.32.0](https://github.com/patricksmill/romm-cli/compare/v0.31.0...v0.32.0) (2026-05-07)

### Features
* **config:** add ExtrasDefaults struct for TUI extras picker configuration ([d4bc2f6](https://github.com/patricksmill/romm-cli/commit/d4bc2f644d5e13697b4699b7533c8fc0552d1166))
* **download:** add validation for non-zero concurrent downloads and improve target preparation logic ([ebbf9a6](https://github.com/patricksmill/romm-cli/commit/ebbf9a6702ce5bb7a1a380f0f4d1d73b281fdc0d))
* **download:** enhance download target preparation and URL handling for ROM files ([def5feb](https://github.com/patricksmill/romm-cli/commit/def5feb8d4ba7ab7639d19b813f1e3c0b31712e7))
* enhance ROM file handling and download management ([8341e0b](https://github.com/patricksmill/romm-cli/commit/8341e0b3d424bca8df5eba3f96e8c39bfd01b17e))

## [0.29.0](https://github.com/patricksmill/romm-cli/compare/v0.28.0...v0.29.0) (2026-05-01)

### Features
* **update:** enhance update mechanism with dynamic binary name resolution and add tests ([947db3f](https://github.com/patricksmill/romm-cli/commit/947db3f02176359b0be13a924e77b393391d999d))
* **update:** improve binary name extraction from path for better compatibility ([deae08a](https://github.com/patricksmill/romm-cli/commit/deae08abb0cf5863c65149647916aa1bb8724810))

## [0.27.0](https://github.com/patricksmill/romm-cli/compare/v0.26.1...v0.27.0) (2026-04-28)

### Features
* **api:** add endpoints for tasks and system management, enhance ROM filtering options ([fcde4c8](https://github.com/patricksmill/romm-cli/commit/fcde4c857a7d2bb0a18cefa6cf5551f252b9859e))

## [0.26.0](https://github.com/patricksmill/romm-cli/compare/v0.25.0...v0.26.0) (2026-04-27)

### Features
* **search:** fix search when results should yield more than 50 results ([b2d93ad](https://github.com/patricksmill/romm-cli/commit/b2d93add4dc0fb4ce21a60916780863f4b3b127a))
* **update:** add startup update checks and interactive prompt ([e601631](https://github.com/patricksmill/romm-cli/commit/e601631598ed9cf47f4d1abbc51d4fa9ec3d3706))

## [0.25.0](https://github.com/patricksmill/romm-cli/compare/v0.24.0...v0.25.0) (2026-04-21)

### Features
* **interrupt:** implement cancellation support for long-running tasks ([e3c9ad4](https://github.com/patricksmill/romm-cli/commit/e3c9ad4b406cad3dab1879aaef3a5e681d5ed182))

### Bug Fixes
* **interrupt:** ran cargo fmt and clippy ([7c404bb](https://github.com/patricksmill/romm-cli/commit/7c404bbaf60a2b466b5fc029566ffcd0c19226ad))

## [0.23.0](https://github.com/patricksmill/romm-cli/compare/v0.22.0...v0.23.0) (2026-04-20)

### Features
* **config:** rename download directory to ROMs directory and enhance handling ([10c62c7](https://github.com/patricksmill/romm-cli/commit/10c62c7ef7a284b0136d7c3b229bf59579a50f33))

## [0.22.0](https://github.com/patricksmill/romm-cli/compare/v0.21.0...v0.22.0) (2026-04-19)

### Features
* **scan:** add library scan functionality post-ROM upload ([e71855d](https://github.com/patricksmill/romm-cli/commit/e71855d0c8ebeffe89faf67c5f630b7333a42891))
* **scan:** enhance library scan functionality with cache management ([349e316](https://github.com/patricksmill/romm-cli/commit/349e316b1e2c6b0f5035c103de6316aa31db4163))

## [0.20.0](https://github.com/patricksmill/romm-cli/compare/v0.19.0...v0.20.0) (2026-04-17)

### Features
* **collections:** introduce virtual and smart collections handling ([bd30eb8](https://github.com/patricksmill/romm-cli/commit/bd30eb8520871f1902008b2dd4d8853a56464254))

## [0.19.0](https://github.com/patricksmill/romm-cli/compare/v0.18.0...v0.19.0) (2026-04-14)

### Features
* **config:** cross-machine auth readiness ([4d9f44d](https://github.com/patricksmill/romm-cli/commit/4d9f44db9993a17281cf0d049f6cf1412768b351))

### Bug Fixes
* **config:** keyring warnings, persist merge, and doc accuracy ([b26cdd1](https://github.com/patricksmill/romm-cli/commit/b26cdd13f6036fa82b35bb679bfb185e9615acbf))

## [0.18.0](https://github.com/patricksmill/romm-cli/compare/v0.17.0...v0.18.0) (2026-04-12)

### Features
* **client:** add unauthenticated JSON request method ([bb904f0](https://github.com/patricksmill/romm-cli/commit/bb904f061e72995a1ab7ed1900a3b527f5aaedf0))
* **config:** enhance keyring integration and update config handling ([4c90381](https://github.com/patricksmill/romm-cli/commit/4c90381f71bb3ea0bd4163d50edfafa22e014cdd))

## [0.13.1](https://github.com/patricksmill/romm-cli/compare/v0.13.0...v0.13.1) (2026-03-29)

### Bug Fixes
* **openapi_sync:** fix default url scheme ([53ec673](https://github.com/patricksmill/romm-cli/commit/53ec6739a062f0669a2e9f5446e4fad390fe0556))

## [0.12.0](https://github.com/patricksmill/romm-cli/compare/v0.11.1...v0.12.0) (2026-03-28)

### Features
* **client:** enhance OpenAPI fetching with scheme fallback and alternate paths; add local openapi.json support in sync process ([7593925](https://github.com/patricksmill/romm-cli/commit/7593925825bb9e5e1a59fe1d90e0d7ac70b070b0))
