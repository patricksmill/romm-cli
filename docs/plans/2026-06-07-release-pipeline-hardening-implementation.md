# Release pipeline hardening — implementation spec

**Date:** 2026-06-07  
**Status:** Implemented

## Goal

Eliminate crates.io publish races by moving ordered publish into [`release-please.yml`](../../.github/workflows/release-please.yml), leaving [`release-artifacts.yml`](../../.github/workflows/release-artifacts.yml) for binaries only.

## Files to create

### `tools/publish-crate.sh`

Idempotent single-crate publish: dry-run, then `cargo publish`; treat `already exists on crates.io index` as success.

### `tools/wait-for-crates-io.sh`

Poll `https://crates.io/api/v1/crates/<crate>/<version>` until HTTP 200 (default 300s timeout).

### `tools/publish-workspace.sh`

Publish order: `romm-api` first, then `romm-tui` and `romm-cli` in parallel.

Flags:

- `--if-created` — read `PUBLISH_API`, `PUBLISH_TUI`, `PUBLISH_CLI` (`true`/`false`) from env
- `--crates romm-api …` — explicit subset, still ordered

Before frontend publish, wait for required `romm-api` version on crates.io when API was not published in the same run.

### `docs/compatibility.toml`

```toml
[[combination]]
romm_cli = "1.0.0"
romm_tui = "1.0.0"
min_romm_api = "1.0.0"
notes = "Workspace fresh start at 1.0.0"
```

## Files to modify

### `.github/workflows/release-please.yml`

- Add `workflow_dispatch` with inputs: `ref` (default `main`), `publish_romm_api`, `publish_romm_tui`, `publish_romm_cli` (booleans)
- Add job outputs from `steps.release.outputs['romm-*--release_created']`
- Add `publish-crates-io` job:
  - Runs on push when any component release was created
  - Runs on `workflow_dispatch` for manual recovery
  - Calls `./tools/publish-workspace.sh --if-created` with env from outputs or dispatch inputs

### `.github/workflows/release-artifacts.yml`

- Remove `publish-crates-io` job
- Remove unused `crate` output from `resolve-tag`
- Update `workflow_dispatch` description: binaries and checksums only

### `.github/workflows/ci.yml`

Add job:

```yaml
clippy-no-default-features:
  name: Clippy (no default features)
  runs-on: ubuntu-latest
  steps:
    - uses: actions/checkout@v6
    - uses: Swatinem/rust-cache@v2
    - uses: dtolnay/rust-toolchain@stable
      with:
        components: clippy
    - run: cargo clippy --workspace --all-targets --no-default-features -- -D warnings
```

### `tools/release-check.sh`

After version checks, add compatibility validation:

- Parse latest `[[combination]]` from `docs/compatibility.toml` (Python `tomllib` or simple parser)
- If `romm-api`, `romm-cli`, `romm-tui` versions all equal → pass
- If any differ → latest row must match current `romm_cli`, `romm_tui`, `min_romm_api` versions

Optional: delegate dry-runs to `publish-crate.sh --dry-run-only` (keep current behavior if simpler).

### `docs/releases.md`

- Update overview diagram: crates.io publish via Release Please workflow
- Release checklist: add `docs/compatibility.toml` row when versions diverge
- Hotfix: `workflow_dispatch` on Release Please for ordered publish; Release Artifacts for binaries only

### `docs/plans/2026-06-06-release-coordination-design.md`

Add **Publish orchestration** section documenting the race fix and starship-style chaining.

## Verification

```bash
cargo fmt && cargo fmt --check
cargo clippy --all-targets --all-features -- -D warnings
cargo clippy --all-targets --no-default-features -- -D warnings
./tools/release-check.sh
bash -n tools/publish-crate.sh tools/wait-for-crates-io.sh tools/publish-workspace.sh
```

## `release-please.yml` target shape

```yaml
name: Release Please

on:
  push:
    branches: [main, master]
  workflow_dispatch:
    inputs:
      ref:
        description: "Git ref to publish from"
        required: true
        default: main
        type: string
      publish_romm_api:
        description: "Publish romm-api"
        type: boolean
        default: true
      publish_romm_tui:
        description: "Publish romm-tui"
        type: boolean
        default: true
      publish_romm_cli:
        description: "Publish romm-cli"
        type: boolean
        default: true

permissions:
  contents: read

concurrency:
  group: release-please-${{ github.ref }}
  cancel-in-progress: false

jobs:
  release-please:
    if: github.event_name == 'push'
    runs-on: ubuntu-latest
    permissions:
      contents: write
      pull-requests: write
    outputs:
      api_created: ${{ steps.release.outputs['romm-api--release_created'] }}
      cli_created: ${{ steps.release.outputs['romm-cli--release_created'] }}
      tui_created: ${{ steps.release.outputs['romm-tui--release_created'] }}
    steps:
      - uses: googleapis/release-please-action@v4
        id: release
        with:
          token: ${{ secrets.GITHUB_TOKEN }}
          config-file: release-please-config.json
          manifest-file: .release-please-manifest.json

  publish-crates-io:
    needs: release-please
    if: >-
      github.event_name == 'workflow_dispatch' ||
      (needs.release-please.result == 'success' &&
        (needs.release-please.outputs.api_created == 'true' ||
         needs.release-please.outputs.cli_created == 'true' ||
         needs.release-please.outputs.tui_created == 'true'))
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v6
        with:
          ref: ${{ github.event_name == 'workflow_dispatch' && inputs.ref || github.sha }}
      - uses: Swatinem/rust-cache@v2
      - uses: dtolnay/rust-toolchain@stable
      - name: Publish to crates.io (ordered)
        env:
          CARGO_REGISTRY_TOKEN: ${{ secrets.CARGO_REGISTRY_TOKEN }}
          PUBLISH_API: ${{ github.event_name == 'workflow_dispatch' && inputs.publish_romm_api || needs.release-please.outputs.api_created }}
          PUBLISH_TUI: ${{ github.event_name == 'workflow_dispatch' && inputs.publish_romm_tui || needs.release-please.outputs.tui_created }}
          PUBLISH_CLI: ${{ github.event_name == 'workflow_dispatch' && inputs.publish_romm_cli || needs.release-please.outputs.cli_created }}
        run: bash ./tools/publish-workspace.sh --if-created
```
