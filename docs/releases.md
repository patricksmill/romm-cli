# Release runbook

This workspace ships three independently versioned crates plus prebuilt desktop binaries. See [release coordination design](plans/2026-06-06-release-coordination-design.md) for rationale and industry benchmarks.

## Overview

```mermaid
flowchart TB
  subgraph crates [Workspace crates]
    API[romm-api]
    CLI[romm-cli]
    TUI[romm-tui]
  end

  subgraph tags [GitHub tags]
    TAPI["romm-api-vA.B.C"]
    TCLI["romm-cli-vX.Y.Z"]
    TTUI["romm-tui-vP.Q.R"]
  end

  API --> TAPI
  CLI --> TCLI
  TUI --> TTUI

  TAPI --> CratesApi[crates.io romm-api]
  TCLI --> CratesCli[crates.io romm-cli]
  TCLI --> BinDual[GitHub dual-binary archives]
  TTUI --> CratesTui[crates.io romm-tui]
  TTUI --> BinTui[GitHub single-binary archives]
```

| Crate | Tag format | Changelog | Binaries | crates.io |
|-------|------------|-----------|----------|-----------|
| `romm-api` | `romm-api-v*` | [romm-api/CHANGELOG.md](../romm-api/CHANGELOG.md) | — | Yes |
| `romm-cli` | `romm-cli-v*` | [romm-cli/CHANGELOG.md](../romm-cli/CHANGELOG.md) | `romm-cli` + `romm-tui` | Yes |
| `romm-tui` | `romm-tui-v*` | [romm-tui/CHANGELOG.md](../romm-tui/CHANGELOG.md) | `romm-tui` only | Yes |

Automation: [Release Please](../.github/workflows/release-please.yml) opens release PRs; [Release Artifacts](../.github/workflows/release-artifacts.yml) builds assets and publishes to crates.io.

## Day-to-day development

1. Use [Conventional Commits](https://www.conventionalcommits.org/) with scopes that match the crate:
   - `feat(api):` / changes under `romm-api/` → bumps `romm-api`
   - `feat(cli):` / `romm-cli/` → bumps `romm-cli`
   - `feat(tui):` / `romm-tui/` → bumps `romm-tui`
2. Merge feature PRs to `main`. Release Please opens a combined **release PR** only for components with qualifying commits.
3. Review the release PR for version bumps, dependency pin updates, and changelog entries.

Root-only doc edits (`docs/`, `README.md`) do not trigger a release unless you add a scoped commit (for example `docs(release): note …` with the target component).

## Release checklist

Before merging a release PR:

- [ ] CI is green (fmt, clippy, tests, publish dry-runs, version consistency)
- [ ] Dependent `Cargo.toml` files show updated `romm-api` / `romm-tui` pins where applicable
- [ ] Per-crate changelogs look correct

After merging:

- [ ] Component tag(s) exist (`romm-<crate>-v*`)
- [ ] [Release Artifacts](../.github/workflows/release-artifacts.yml) completed for frontend tags
- [ ] crates.io shows the new version(s) — publish order: **romm-api → romm-tui → romm-cli**
- [ ] Smoke test: download an archive, run `romm-cli --version` and `romm-tui --version`
- [ ] Optional: `romm-cli update --help` / TUI update flow on a test machine

Local preflight:

```bash
./tools/release-check.sh
```

## Breaking `romm-api` releases

When `romm-api` has a breaking semver bump:

1. Ensure Release Please also bumps or patch-bumps `romm-cli` and `romm-tui` in the same release window (dependency cascade).
2. Publish **romm-api** to crates.io before frontends.
3. Cut frontend releases the same day so `cargo install` never resolves an unpublished API version.

## Hotfix / manual recovery

Use **workflow_dispatch** on [Release Artifacts](../.github/workflows/release-artifacts.yml) with the full tag (for example `romm-cli-v0.40.1`).

For crates.io-only recovery, publish manually in order:

```bash
cargo publish -p romm-api
cargo publish -p romm-tui
cargo publish -p romm-cli
```

## Self-update and binary layout

- **`romm-cli-v*` archives** contain both `romm-cli` and `romm-tui`. Self-update from either binary can refresh siblings when updating via a CLI distribution release.
- **`romm-tui-v*` archives** contain only `romm-tui`.
- Legacy unified tags (`v0.x.y`) are still recognized for `romm-cli` during transition.

Changelog URLs point to per-crate changelogs; the [root index](../CHANGELOG.md) links to all three.

## Compatibility matrix (template)

Record published combinations when versions diverge:

| `romm-cli` tag | `romm-tui` tag | Min `romm-api` | Notes |
|----------------|----------------|----------------|-------|
| `romm-cli-v0.40.0` | `romm-tui-v0.40.0` | `0.40.0` | Last unified-era bootstrap |
| | | | |

## Android (future)

When `android/` lands:

| Artifact | Versioning | Channel |
|----------|------------|---------|
| UniFFI `.so` | Matches `romm-api` semver | Bundled in APK/AAB |
| Android app | Independent `versionName` / `versionCode` | Play Store / sideload |

CI contract (see [android-release.yml](../.github/workflows/android-release.yml)):

- `rommApiVersion` injected from `romm-api/Cargo.toml` at build time
- App manifest declares `minRommApiFfiVersion`
- Release notes cite the compatible `romm-api` range

## Tag migration (one-time)

Historical releases used unified `v0.x.y` tags. New releases use component tags. To bootstrap from the last unified release at `0.40.0`:

```bash
./tools/bootstrap-component-tags.sh 0.40.0 <commit-sha>
```

See [release coordination design](plans/2026-06-06-release-coordination-design.md) for full migration notes.
