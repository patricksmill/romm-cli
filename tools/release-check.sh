#!/usr/bin/env bash
# Version consistency and publish-order preflight for maintainers and CI.
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

err() {
  echo "release-check: $*" >&2
  exit 1
}

crate_version() {
  local crate="$1"
  grep -E '^version = ' "$crate/Cargo.toml" | head -1 | sed 's/version = "\(.*\)"/\1/'
}

manifest_version() {
  local crate="$1"
  python3 -c "import json; print(json.load(open('.release-please-manifest.json'))['$crate'])"
}

echo "==> Checking .release-please-manifest.json matches crate versions"
for crate in romm-api romm-cli romm-tui; do
  cv="$(crate_version "$crate")"
  mv="$(manifest_version "$crate")"
  if [ "$cv" != "$mv" ]; then
    err "$crate Cargo.toml version ($cv) != manifest ($mv)"
  fi
  echo "  OK $crate $cv"
done

echo "==> Checking root Cargo.toml has no workspace.package.version"
if grep -qE '^version = ' Cargo.toml; then
  err "root Cargo.toml must not set [workspace.package] version"
fi

echo "==> Checking workspace.dependencies version pins"
api_pin="$(grep 'romm-api = ' Cargo.toml | sed -n 's/.*version = "\([^"]*\)".*/\1/p')"
api_major_minor="$(crate_version romm-api | sed -E 's/(\.[0-9]+)$//')"
if [ "$api_pin" != "$api_major_minor" ]; then
  err "workspace romm-api pin ($api_pin) should match romm-api major.minor ($api_major_minor)"
fi
echo "  OK workspace pins"

if [ "${1:-}" != "versions-only" ]; then
  echo "==> Topological publish dry-run (romm-api -> romm-tui -> romm-cli)"
  cargo publish -p romm-api --dry-run
  cargo publish -p romm-tui --dry-run
  cargo publish -p romm-cli --dry-run
fi

echo "release-check: all checks passed"
