# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.38.0](https://github.com/patricksmill/romm-cli/compare/v0.37.0...v0.38.0) (2026-06-06)


### Features

* **completions:** add shell completion scripts and build script ([f54d761](https://github.com/patricksmill/romm-cli/commit/f54d76119e99694c4ce41797f3b9488a3b4843f8))
* **completions:** enhance shell completion support and CI integration ([9d2db49](https://github.com/patricksmill/romm-cli/commit/9d2db490891ddeca1201c485893f7895da1fb988))
* **errors:** implement typed error handling across the codebase ([fcae759](https://github.com/patricksmill/romm-cli/commit/fcae75915d175462775c6a952bc36d15535366ed))


### Bug Fixes

* **config:** improve error handling in persist_user_config function ([920ee3c](https://github.com/patricksmill/romm-cli/commit/920ee3cab35cefd05b89903b18d6c9dcfeb340a5))

## [0.37.0](https://github.com/patricksmill/romm-cli/compare/v0.36.0...v0.37.0) (2026-05-28)


### Features

* **config:** add persisted TUI theme field ([737fd56](https://github.com/patricksmill/romm-cli/commit/737fd56f03925df26e5413c6f0f670043e982e20))
* **download:** implement download management system ([d8e5c36](https://github.com/patricksmill/romm-cli/commit/d8e5c365a03bd931abd567889a8ed56cd19813c9))
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

* **download:** restore Platform import in test helpers ([d07505f](https://github.com/patricksmill/romm-cli/commit/d07505f8d456945a41d75dc157207d4d601380a9))
* **setup-wizard:** align ROMs directory step label with 3/6 flow ([b11293f](https://github.com/patricksmill/romm-cli/commit/b11293fa78dcb6eb21f7dc5a677dd1dd960a82bd))

## [0.36.0](https://github.com/patricksmill/romm-cli/compare/v0.35.0...v0.36.0) (2026-05-24)


### Features

* **app:** enhance global shortcut handling and startup splash logic ([cbc9967](https://github.com/patricksmill/romm-cli/commit/cbc99671c45cd45b8f9ee4c66530baacd02771a9))

## [0.35.0](https://github.com/patricksmill/romm-cli/compare/v0.34.0...v0.35.0) (2026-05-24)


### Features

* **config:** implement custom console paths for ROM layout ([d360bca](https://github.com/patricksmill/romm-cli/commit/d360bca6de0ff24f3bad83301a4170049de958de))
* **config:** introduce customizable ROM layout options ([874d815](https://github.com/patricksmill/romm-cli/commit/874d81578d53e49ac2f3f3d3bd1ad182238f301a))
* **save-sync:** implement custom save paths for consoles ([6e2fcec](https://github.com/patricksmill/romm-cli/commit/6e2fcec52038cb9136ea8f894404ce585403ad2b))


### Bug Fixes

* **tests:** update download test to reflect new directory structure ([65d47e8](https://github.com/patricksmill/romm-cli/commit/65d47e8796620c9ea8642d77071c8ec67639b48f))

## [Unreleased]

### Added

- **config/TUI:** Per-console custom save directory overrides (`save_sync.platform_dirs`), configured under **Settings → Saves → Save console paths** or in `config.json`.

### Changed

- **config:** Replace auto/manual ROM layout mode with optional per-console custom paths (`platform_dirs` only). Legacy `"mode"` in config is ignored on load and omitted on save. `ROMM_ROMS_LAYOUT` env var removed.
- **TUI:** Save downloads now use `{save_base}/{platform-slug}/{game}/` by default (previously `{save_base}/{game}/`). Custom per-console mappings use absolute paths, same model as ROM downloads.

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
* enhance settings screen with tab navigation and save sync options ([ca44184](https://github.com/patricksmill/romm-cli/commit/ca44184a6c3cb52648844a14a2ebc1abad3e2cd6))
* implement save sync compatibility checks and enhance error handling for unsupported endpoints ([5b25da3](https://github.com/patricksmill/romm-cli/commit/5b25da3cf817a3937d394226d82bbf7f36fff52e))

## [0.32.0](https://github.com/patricksmill/romm-cli/compare/v0.31.0...v0.32.0) (2026-05-07)


### Features

* add download extras command to fetch covers, manuals, and DLC for games ([11fb0f4](https://github.com/patricksmill/romm-cli/commit/11fb0f4346c755ed44937b02d208cd7a2926b178))
* **config:** add ExtrasDefaults struct for TUI extras picker configuration ([d4bc2f6](https://github.com/patricksmill/romm-cli/commit/d4bc2f644d5e13697b4699b7533c8fc0552d1166))
* **download:** add validation for non-zero concurrent downloads and improve target preparation logic ([ebbf9a6](https://github.com/patricksmill/romm-cli/commit/ebbf9a6702ce5bb7a1a380f0f4d1d73b281fdc0d))
* **download:** enhance download target preparation and URL handling for ROM files ([def5feb](https://github.com/patricksmill/romm-cli/commit/def5feb8d4ba7ab7639d19b813f1e3c0b31712e7))
* enhance ROM file handling and download management ([8341e0b](https://github.com/patricksmill/romm-cli/commit/8341e0b3d424bca8df5eba3f96e8c39bfd01b17e))

## [0.31.0](https://github.com/patricksmill/romm-cli/compare/v0.30.1...v0.31.0) (2026-05-01)


### Features

* **settings:** add "Delete Cache" option ([c6cbc43](https://github.com/patricksmill/romm-cli/commit/c6cbc439fa070d16b1e0c4a15b0e8668e05c3827))
* **tui:** implement lazyloading for uncached game list scrolling ([4c09cff](https://github.com/patricksmill/romm-cli/commit/4c09cffcaa7726e956d0e24ab39df9e15b6cea42))


### Bug Fixes

* **formatting:** ran cargo fmt ([2ba7716](https://github.com/patricksmill/romm-cli/commit/2ba771654e16b9097e4a0173751115518246ef4a))
* **tui:** retain cursor position when a list of roms is still being populated ([61af926](https://github.com/patricksmill/romm-cli/commit/61af926d0d11b090d520f9a5bef6ce0661b1a79b))
* **tui:** show correct number of distinct roms in a given console count, instead of number of files ([29dbf1b](https://github.com/patricksmill/romm-cli/commit/29dbf1b58de600a29884c5e16bd247be1ab2e519))

## [0.30.1](https://github.com/patricksmill/romm-cli/compare/v0.30.0...v0.30.1) (2026-05-01)


### Bug Fixes

* **tui:** clear "Opened in Browser" after 3 seconds, clear "Failed to open:" after 5 seconds ([6668362](https://github.com/patricksmill/romm-cli/commit/6668362a1e74d41d5e09c53bc326899321e8bf41))
* **update:** fix redundant update prompt after skipping cli update when running romm-cli tui ([6872855](https://github.com/patricksmill/romm-cli/commit/6872855a5ea05985547e943a63c65bb79dbbaabb))

## [0.30.0](https://github.com/patricksmill/romm-cli/compare/v0.29.0...v0.30.0) (2026-05-01)


### Features

* **settings:** iallow pasting into setup wizard, add reset feature to settings ([202c4a0](https://github.com/patricksmill/romm-cli/commit/202c4a0bb6705a532d63e8108207f93c50067309))
* **setup_wizard:** add user hints to the setup prcoess, rearrange the authentication method list to have the easiest method (8 character pairing) on top. ([7b5fc47](https://github.com/patricksmill/romm-cli/commit/7b5fc47ceff13ec6a4c63fa531fa3e31959fd473))
* **tui:** clear metadata footer and display tooltips after 3s, add help menu tooltip, fix cursor placement issue in settings when entering baseurl field ([7d8e230](https://github.com/patricksmill/romm-cli/commit/7d8e2300e5e68b7680148653c4d654a512af61ef))


### Bug Fixes

* cargo fmt and clippy ([bc0a636](https://github.com/patricksmill/romm-cli/commit/bc0a636dcfd7fffe143d6e938b5e82ba9cb6653e))

## [0.29.0](https://github.com/patricksmill/romm-cli/compare/v0.28.0...v0.29.0) (2026-05-01)


### Features

* **docs:** add more documentation to various parts of the source ([efdc6e3](https://github.com/patricksmill/romm-cli/commit/efdc6e3af51f9fa60d5a64f14ba2e2a3f70478b2))
* **tests:** enhance setup wizard tests with unique download directory handling ([deae08a](https://github.com/patricksmill/romm-cli/commit/deae08abb0cf5863c65149647916aa1bb8724810))
* **update:** enhance update mechanism with dynamic binary name resolution and add tests ([947db3f](https://github.com/patricksmill/romm-cli/commit/947db3f02176359b0be13a924e77b393391d999d))
* **update:** improve binary name extraction from path for better compatibility ([deae08a](https://github.com/patricksmill/romm-cli/commit/deae08abb0cf5863c65149647916aa1bb8724810))

## [0.28.0](https://github.com/patricksmill/romm-cli/compare/v0.27.0...v0.28.0) (2026-04-29)


### Features

* **auth:** add authentication command group to manage credentials ([72b9182](https://github.com/patricksmill/romm-cli/commit/72b9182b439f81e22fc58f67632a3486036a6695))

## [0.27.0](https://github.com/patricksmill/romm-cli/compare/v0.26.1...v0.27.0) (2026-04-28)


### Features

* **api:** add endpoints for tasks and system management, enhance ROM filtering options ([fcde4c8](https://github.com/patricksmill/romm-cli/commit/fcde4c857a7d2bb0a18cefa6cf5551f252b9859e))

## [0.26.1](https://github.com/patricksmill/romm-cli/compare/v0.26.0...v0.26.1) (2026-04-27)


### Bug Fixes

* **ci:** stabilize release pipeline and restore v0.26 metadata ([955d5b8](https://github.com/patricksmill/romm-cli/commit/955d5b891fbff99d2afd75a515b6de6b46924aed))

## [Unreleased]

### Features

* **update:** add startup update checks and interactive CLI/TUI update prompt with changelog shortcut

## [0.26.0](https://github.com/patricksmill/romm-cli/compare/v0.25.0...v0.26.0) (2026-04-27)


### Features

* **search:** fix search when results should yield more than 50 results ([b2d93ad](https://github.com/patricksmill/romm-cli/commit/b2d93add4dc0fb4ce21a60916780863f4b3b127a))
* **update:** add startup update checks and interactive prompt ([e601631](https://github.com/patricksmill/romm-cli/commit/e601631598ed9cf47f4d1abbc51d4fa9ec3d3706))


### Bug Fixes

* **setup_wizard:** Removed inline cursor glyph from password and API key fields in the setup wizard, ensuring they rely on the terminal cursor instead. ([34123e2](https://github.com/patricksmill/romm-cli/commit/34123e25a1eb5ca30de90b2ea2a00e4d038d116d))

## [0.25.0](https://github.com/patricksmill/romm-cli/compare/v0.24.0...v0.25.0) (2026-04-21)


### Features

* **interrupt:** implement cancellation support for long-running tasks ([e3c9ad4](https://github.com/patricksmill/romm-cli/commit/e3c9ad4b406cad3dab1879aaef3a5e681d5ed182))


### Bug Fixes

* **cli:** fix testing to reflect using --platform instead of --platform-id ([f6ef5b4](https://github.com/patricksmill/romm-cli/commit/f6ef5b47618c2b66f442d4d552559c0428842b2b))
* **interrupt:** ran cargo fmt and clippy ([7c404bb](https://github.com/patricksmill/romm-cli/commit/7c404bbaf60a2b466b5fc029566ffcd0c19226ad))
* **tui:** remove unused cover_protocol from GameDetailScreen ([b7b7679](https://github.com/patricksmill/romm-cli/commit/b7b7679bc0ba899850b330cfd389ab2fafce4e11))

## [0.24.0](https://github.com/patricksmill/romm-cli/compare/v0.23.0...v0.24.0) (2026-04-21)


### Features

* **tui:** implement path picker for file and directory selection ([6a640ed](https://github.com/patricksmill/romm-cli/commit/6a640ed8e0471b816782d2e2ab1fa45e1d40b4e2))
* **tui:** implement path picker for file and directory selection ([f07149b](https://github.com/patricksmill/romm-cli/commit/f07149b5c51c9534a593c0cc1270183301cd63e5))


### Bug Fixes

* **tui:** update GameDetailPrevious to use Box for LibraryBrowseScreen ([c966b3e](https://github.com/patricksmill/romm-cli/commit/c966b3e0294ae0b02630855b17b6109f32238bd4))

## [0.23.0](https://github.com/patricksmill/romm-cli/compare/v0.22.0...v0.23.0) (2026-04-20)


### Features

* **config:** rename download directory to ROMs directory and enhance handling ([10c62c7](https://github.com/patricksmill/romm-cli/commit/10c62c7ef7a284b0136d7c3b229bf59579a50f33))
* **cover:** enhance game detail view with cover image loading ([8ea7a0f](https://github.com/patricksmill/romm-cli/commit/8ea7a0f3bfedcfe4bb77fba674c8971ddcb4cc1c))

## [0.22.0](https://github.com/patricksmill/romm-cli/compare/v0.21.0...v0.22.0) (2026-04-19)


### Features

* **roms:** implement ROM file upload functionality ([b418c15](https://github.com/patricksmill/romm-cli/commit/b418c1556c894ae1dc9dbdbf23d84dcb8994162f))
* **roms:** implement ROM file upload functionality ([15fc288](https://github.com/patricksmill/romm-cli/commit/15fc288c1394940d46c417b0d9e9c68d329700f5))
* **scan:** add library scan functionality post-ROM upload ([e71855d](https://github.com/patricksmill/romm-cli/commit/e71855d0c8ebeffe89faf67c5f630b7333a42891))
* **scan:** enhance library scan functionality with cache management ([349e316](https://github.com/patricksmill/romm-cli/commit/349e316b1e2c6b0f5035c103de6316aa31db4163))
* **upload:** add ROM upload functionality to Library screen ([24ce155](https://github.com/patricksmill/romm-cli/commit/24ce155bc6f310a5ac90020c11d9267b99143eb6))

## [0.21.0](https://github.com/patricksmill/romm-cli/compare/v0.20.0...v0.21.0) (2026-04-19)


### Features

* **cache:** add cache management commands and functionality ([3c99a05](https://github.com/patricksmill/romm-cli/commit/3c99a05c23123326f9f9dfc53fefdfbba32ae58c))
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

* **collections:** introduce virtual and smart collections handling ([bd30eb8](https://github.com/patricksmill/romm-cli/commit/bd30eb8520871f1902008b2dd4d8853a56464254))
* **tui:** enhance keyboard navigation and help overlay ([c7f47aa](https://github.com/patricksmill/romm-cli/commit/c7f47aa252d8ed3d977f0d665d1c8113286e3182))
* **tui:** implement enhanced search functionality for library and ROM panes ([96309ae](https://github.com/patricksmill/romm-cli/commit/96309ae8ebf45089102bf3111467af98f2647289))


### Bug Fixes

* **tui:** improve filter browsing behavior and add index clamping ([b5e01e4](https://github.com/patricksmill/romm-cli/commit/b5e01e481953985e3cc9636283725fb68e32c5a2))
* **tui:** refine search behavior and results handling ([99a9f03](https://github.com/patricksmill/romm-cli/commit/99a9f03861704e241640104bd8857809f56dc5b3))

## [0.19.0](https://github.com/patricksmill/romm-cli/compare/v0.18.0...v0.19.0) (2026-04-14)


### Features

* **config:** cross-machine auth readiness ([4d9f44d](https://github.com/patricksmill/romm-cli/commit/4d9f44db9993a17281cf0d049f6cf1412768b351))
* **tui:** add pairing authentication step to setup wizard ([10fa984](https://github.com/patricksmill/romm-cli/commit/10fa9842803e2dd78bdafc2869831e051576b09b))


### Bug Fixes

* **config:** keyring warnings, persist merge, and doc accuracy ([b26cdd1](https://github.com/patricksmill/romm-cli/commit/b26cdd13f6036fa82b35bb679bfb185e9615acbf))

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
