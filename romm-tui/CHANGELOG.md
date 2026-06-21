# Changelog

All notable changes to **romm-tui** are documented in this file.

Entries before the workspace split (1.0.0) are filtered from the unified monolith history by conventional-commit scope. Shared library changes appear in [romm-api/CHANGELOG.md](../romm-api/CHANGELOG.md). CLI and scripting changes appear in [romm-cli/CHANGELOG.md](../romm-cli/CHANGELOG.md).

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [1.1.1](https://github.com/patricksmill/romm-cli/compare/romm-tui-v1.1.0...romm-tui-v1.1.1) (2026-06-21)


### Bug Fixes

* **release:** use crate-relative changelog paths for release-please ([e7151fd](https://github.com/patricksmill/romm-cli/commit/e7151fd8e3101cd7a2dbb477c1062704818517f9))

## [1.1.0](https://github.com/patricksmill/romm-cli/compare/romm-tui-v1.0.0...romm-tui-v1.1.0) (2026-06-07)

### Features

* **tui:** restore pane filter on f and improve keyboard help layout ([22bf426](https://github.com/patricksmill/romm-cli/commit/22bf4269b967cbc193f1454b77a633a5d40783ef))

## [1.0.0](https://github.com/patricksmill/romm-cli/compare/romm-tui-v0.40.0...romm-tui-v1.0.0) (2026-06-07)

### Features

* **tui:** fresh 1.0.0 release as standalone frontend crate after workspace split

## [0.40.0](https://github.com/patricksmill/romm-cli/compare/v0.39.0...v0.40.0) (2026-06-06)

### Features
* **tui:** add persisted panel layout and fix overlay keyboard handling ([e526ad5](https://github.com/patricksmill/romm-cli/commit/e526ad5458c0c7f6525a3746acab31038953058f))
* **tui:** replace main menu with global search and keybinding navigation ([e8fbc36](https://github.com/patricksmill/romm-cli/commit/e8fbc3638f5dc96522d739f0dc630ad0cfbe0b72))

## [0.39.0](https://github.com/patricksmill/romm-cli/compare/v0.38.0...v0.39.0) (2026-06-06)

### Features
* **tui:** implement event/action pipeline for improved TUI architecture ([c24ef8a](https://github.com/patricksmill/romm-cli/commit/c24ef8ab0b1c19805978eb42499a838bc830720d))

### Bug Fixes
* **tui:** restore tab navigation on settings Appearance tab ([2ee3457](https://github.com/patricksmill/romm-cli/commit/2ee3457a5616b27abad6662fba9abb6c0a48c11e))

## [0.37.0](https://github.com/patricksmill/romm-cli/compare/v0.36.0...v0.37.0) (2026-05-28)

### Features
* **rom-load:** enhance ROM loading logic with selection validation ([3e016f0](https://github.com/patricksmill/romm-cli/commit/3e016f02c4380bef00654159284330f48ffd63d7))
* **settings:** enhance settings exit behavior and unsaved changes handling ([c6cd88c](https://github.com/patricksmill/romm-cli/commit/c6cd88ca8c02c058db9c8d70bf95e1ba9f809598))
* **settings:** introduce Appearance tab for theme selection and cycling ([cd14fba](https://github.com/patricksmill/romm-cli/commit/cd14fbaff27b5a5879feb9d3627f6b5994a23b8b))
* **setup-wizard:** modularize setup wizard into submodules ([f6a4725](https://github.com/patricksmill/romm-cli/commit/f6a4725aeba290c6609183165a79ab0e39c21a03))
* **tui:** add RommStyles theme wrapper on App ([705b593](https://github.com/patricksmill/romm-cli/commit/705b593d1a788738f054902b0a6d6d21a88eed4e))
* **tui:** enhance RommStyles for native terminal compatibility ([75cc8da](https://github.com/patricksmill/romm-cli/commit/75cc8da5566c848a38f48f81de16bbb834942a5b))
* **tui:** implement immersive theming across TUI components ([14b28b4](https://github.com/patricksmill/romm-cli/commit/14b28b42bf8d188be5529cd740544fe9b8a92f93))
* **tui:** implement theme application and revert functionality ([e5297e6](https://github.com/patricksmill/romm-cli/commit/e5297e6639ba225eb61ed83369388f199e1f6d3e))
* **tui:** integrate RommStyles into UI components for consistent theming ([9f59ebd](https://github.com/patricksmill/romm-cli/commit/9f59ebdce433109df7af21f8a499054d929bf3e2))
* **tui:** optimize theme handling and improve release profile ([c8844f1](https://github.com/patricksmill/romm-cli/commit/c8844f13a2d45dc0ebb32895d672992e8db927f6))

### Bug Fixes
* **setup-wizard:** align ROMs directory step label with 3/6 flow ([b11293f](https://github.com/patricksmill/romm-cli/commit/b11293fa78dcb6eb21f7dc5a677dd1dd960a82bd))

## [0.36.0](https://github.com/patricksmill/romm-cli/compare/v0.35.0...v0.36.0) (2026-05-24)

### Features
* **app:** enhance global shortcut handling and startup splash logic ([cbc9967](https://github.com/patricksmill/romm-cli/commit/cbc99671c45cd45b8f9ee4c66530baacd02771a9))

## [Unreleased]

### Added
- **config/TUI:** Per-console custom save directory overrides (`save_sync.platform_dirs`), configured under **Settings → Saves → Save console paths** or in `config.json`.

### Changed
- **TUI:** Save downloads now use `{save_base}/{platform-slug}/{game}/` by default (previously `{save_base}/{game}/`). Custom per-console mappings use absolute paths, same model as ROM downloads.

## [0.33.0](https://github.com/patricksmill/romm-cli/compare/v0.32.0...v0.33.0) (2026-05-12)

### Features
* enhance settings screen with tab navigation and save sync options ([ca44184](https://github.com/patricksmill/romm-cli/commit/ca44184a6c3cb52648844a14a2ebc1abad3e2cd6))

## [0.31.0](https://github.com/patricksmill/romm-cli/compare/v0.30.1...v0.31.0) (2026-05-01)

### Features
* **settings:** add "Delete Cache" option ([c6cbc43](https://github.com/patricksmill/romm-cli/commit/c6cbc439fa070d16b1e0c4a15b0e8668e05c3827))
* **tui:** implement lazyloading for uncached game list scrolling ([4c09cff](https://github.com/patricksmill/romm-cli/commit/4c09cffcaa7726e956d0e24ab39df9e15b6cea42))

### Bug Fixes
* **tui:** retain cursor position when a list of roms is still being populated ([61af926](https://github.com/patricksmill/romm-cli/commit/61af926d0d11b090d520f9a5bef6ce0661b1a79b))
* **tui:** show correct number of distinct roms in a given console count, instead of number of files ([29dbf1b](https://github.com/patricksmill/romm-cli/commit/29dbf1b58de600a29884c5e16bd247be1ab2e519))

## [0.30.1](https://github.com/patricksmill/romm-cli/compare/v0.30.0...v0.30.1) (2026-05-01)

### Bug Fixes
* **tui:** clear "Opened in Browser" after 3 seconds, clear "Failed to open:" after 5 seconds ([6668362](https://github.com/patricksmill/romm-cli/commit/6668362a1e74d41d5e09c53bc326899321e8bf41))

## [0.30.0](https://github.com/patricksmill/romm-cli/compare/v0.29.0...v0.30.0) (2026-05-01)

### Features
* **settings:** iallow pasting into setup wizard, add reset feature to settings ([202c4a0](https://github.com/patricksmill/romm-cli/commit/202c4a0bb6705a532d63e8108207f93c50067309))
* **setup_wizard:** add user hints to the setup prcoess, rearrange the authentication method list to have the easiest method (8 character pairing) on top. ([7b5fc47](https://github.com/patricksmill/romm-cli/commit/7b5fc47ceff13ec6a4c63fa531fa3e31959fd473))
* **tui:** clear metadata footer and display tooltips after 3s, add help menu tooltip, fix cursor placement issue in settings when entering baseurl field ([7d8e230](https://github.com/patricksmill/romm-cli/commit/7d8e2300e5e68b7680148653c4d654a512af61ef))

## [0.26.0](https://github.com/patricksmill/romm-cli/compare/v0.25.0...v0.26.0) (2026-04-27)

### Bug Fixes
* **setup_wizard:** Removed inline cursor glyph from password and API key fields in the setup wizard, ensuring they rely on the terminal cursor instead. ([34123e2](https://github.com/patricksmill/romm-cli/commit/34123e25a1eb5ca30de90b2ea2a00e4d038d116d))

## [0.25.0](https://github.com/patricksmill/romm-cli/compare/v0.24.0...v0.25.0) (2026-04-21)

### Bug Fixes
* **tui:** remove unused cover_protocol from GameDetailScreen ([b7b7679](https://github.com/patricksmill/romm-cli/commit/b7b7679bc0ba899850b330cfd389ab2fafce4e11))

## [0.24.0](https://github.com/patricksmill/romm-cli/compare/v0.23.0...v0.24.0) (2026-04-21)

### Features
* **tui:** implement path picker for file and directory selection ([6a640ed](https://github.com/patricksmill/romm-cli/commit/6a640ed8e0471b816782d2e2ab1fa45e1d40b4e2))
* **tui:** implement path picker for file and directory selection ([f07149b](https://github.com/patricksmill/romm-cli/commit/f07149b5c51c9534a593c0cc1270183301cd63e5))

### Bug Fixes
* **tui:** update GameDetailPrevious to use Box for LibraryBrowseScreen ([c966b3e](https://github.com/patricksmill/romm-cli/commit/c966b3e0294ae0b02630855b17b6109f32238bd4))

## [0.23.0](https://github.com/patricksmill/romm-cli/compare/v0.22.0...v0.23.0) (2026-04-20)

### Features
* **cover:** enhance game detail view with cover image loading ([8ea7a0f](https://github.com/patricksmill/romm-cli/commit/8ea7a0f3bfedcfe4bb77fba674c8971ddcb4cc1c))

## [0.22.0](https://github.com/patricksmill/romm-cli/compare/v0.21.0...v0.22.0) (2026-04-19)

### Features
* **upload:** add ROM upload functionality to Library screen ([24ce155](https://github.com/patricksmill/romm-cli/commit/24ce155bc6f310a5ac90020c11d9267b99143eb6))

## [0.21.0](https://github.com/patricksmill/romm-cli/compare/v0.20.0...v0.21.0) (2026-04-19)

### Features
* **tui:** enhance library browsing with collection digest and prefetching ([7f779c2](https://github.com/patricksmill/romm-cli/commit/7f779c2c3b7f3d4c7e80a169d49113dbfc6e861e))
* **tui:** enhance ROM loading management and error handling ([f3085b9](https://github.com/patricksmill/romm-cli/commit/f3085b940633c0383dca35d99daac9ed9f5ed1d8))
* **tui:** enhance settings screen and server version handling ([e59b015](https://github.com/patricksmill/romm-cli/commit/e59b015f43f68b9766095c664c212bb98edf08b3))
* **tui:** implement library metadata snapshot for faster TUI startup ([d33b36e](https://github.com/patricksmill/romm-cli/commit/d33b36e607df13268b1324b2fbd21d551d1f7c04))
* **tui:** implement search loading state and improve error handling ([7d17fbd](https://github.com/patricksmill/romm-cli/commit/7d17fbde998aa14d21378b65c3d9fd498de7f169))
* **tui:** improve ROM loading state management in library browsing ([2c46982](https://github.com/patricksmill/romm-cli/commit/2c46982ed0bba4039395630f98953ab50fb57c30))

### Bug Fixes
* **tui:** fix clippy warning in deferred_load_roms ([27c6059](https://github.com/patricksmill/romm-cli/commit/27c60598ba7f74c2879b3aea461e5e4396edc632))
* **tui:** prevent deferred ROM loading for zero-ROM platforms ([7e2f89b](https://github.com/patricksmill/romm-cli/commit/7e2f89b926c1ac14adaa565b0f281814b3d5b7dd))

## [0.20.0](https://github.com/patricksmill/romm-cli/compare/v0.19.0...v0.20.0) (2026-04-17)

### Features
* **tui:** enhance keyboard navigation and help overlay ([c7f47aa](https://github.com/patricksmill/romm-cli/commit/c7f47aa252d8ed3d977f0d665d1c8113286e3182))
* **tui:** implement enhanced search functionality for library and ROM panes ([96309ae](https://github.com/patricksmill/romm-cli/commit/96309ae8ebf45089102bf3111467af98f2647289))

### Bug Fixes
* **tui:** improve filter browsing behavior and add index clamping ([b5e01e4](https://github.com/patricksmill/romm-cli/commit/b5e01e481953985e3cc9636283725fb68e32c5a2))
* **tui:** refine search behavior and results handling ([99a9f03](https://github.com/patricksmill/romm-cli/commit/99a9f03861704e241640104bd8857809f56dc5b3))

## [0.19.0](https://github.com/patricksmill/romm-cli/compare/v0.18.0...v0.19.0) (2026-04-14)

### Features
* **tui:** add pairing authentication step to setup wizard ([10fa984](https://github.com/patricksmill/romm-cli/commit/10fa9842803e2dd78bdafc2869831e051576b09b))

## [0.18.0](https://github.com/patricksmill/romm-cli/compare/v0.17.0...v0.18.0) (2026-04-12)

### Features
* **tui:** implement global error handling and integrate wiremock for testing ([883be87](https://github.com/patricksmill/romm-cli/commit/883be8702b7e43a70eaf17f38f60a4e6864e92ef))

## [0.16.0](https://github.com/patricksmill/romm-cli/compare/v0.15.2...v0.16.0) (2026-03-31)

### Features
* **settings:** allow user to chang base url, enable/disable https, and edit password in settings in tui ([c49c30e](https://github.com/patricksmill/romm-cli/commit/c49c30e40926cca28093aa968612c4bd5f647d80))

### Bug Fixes
* **settings:** fix formatting ([cfb61be](https://github.com/patricksmill/romm-cli/commit/cfb61bee8640711c6c4fba30471a86416678e1e8))
* **settings:** fix formatting ([90ec22d](https://github.com/patricksmill/romm-cli/commit/90ec22dffec6bcae18f4f4f5a29092b371e5fad1))

## [0.14.0](https://github.com/patricksmill/romm-cli/compare/v0.13.1...v0.14.0) (2026-03-30)

### Features
* **tui:** add startup steps to tui ([7488831](https://github.com/patricksmill/romm-cli/commit/7488831e78deb61e1b5c053e11e277d242763fac))

## [0.10.0](https://github.com/patricksmill/romm-cli/compare/v0.9.0...v0.10.0) (2026-03-28)

### Features
* Implement TUI and CLI frontends, add project metadata, and establish CI/CD workflows. ([3013a41](https://github.com/patricksmill/romm-cli/commit/3013a4146493e6e7faccdd7567a969fee66cebee))

## [0.2.0] - 2026-03-20

### Added
- `romm-tui` binary to launch the TUI without a subcommand; `ROMM_VERBOSE=1` enables HTTP logging
