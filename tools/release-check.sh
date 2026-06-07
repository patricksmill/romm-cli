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
  grep -E '^version = ' "$crate/Cargo.toml" | head -1 | tr -d '\r' | sed 's/version = "\(.*\)"/\1/'
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

echo "==> Checking docs/compatibility.toml"
python3 - <<'PY'
import sys
from pathlib import Path

try:
    import tomllib
except ImportError:
    err = "release-check: Python 3.11+ required for tomllib"
    print(err, file=sys.stderr)
    sys.exit(1)

path = Path("docs/compatibility.toml")
if not path.is_file():
    print("release-check: missing docs/compatibility.toml", file=sys.stderr)
    sys.exit(1)

with path.open("rb") as f:
    data = tomllib.load(f)
rows = data.get("combination")
if not rows:
    print("release-check: docs/compatibility.toml has no [[combination]] rows", file=sys.stderr)
    sys.exit(1)

latest = rows[-1]
for key in ("romm_cli", "romm_tui", "min_romm_api"):
    if key not in latest:
        print(f"release-check: latest combination missing {key}", file=sys.stderr)
        sys.exit(1)

import re

def crate_version(crate: str) -> str:
    text = Path(f"{crate}/Cargo.toml").read_text(encoding="utf-8")
    match = re.search(r'^version = "([^"]+)"', text, re.M)
    if not match:
        raise SystemExit(f"release-check: no version in {crate}/Cargo.toml")
    return match.group(1)

api = crate_version("romm-api")
cli = crate_version("romm-cli")
tui = crate_version("romm-tui")

if api == cli == tui:
    print("  OK lockstep versions; compatibility matrix not required to diverge")
    sys.exit(0)

if latest["romm_cli"] != cli:
    print(f"release-check: compatibility.romm_cli ({latest['romm_cli']}) != romm-cli ({cli})", file=sys.stderr)
    sys.exit(1)
if latest["romm_tui"] != tui:
    print(f"release-check: compatibility.romm_tui ({latest['romm_tui']}) != romm-tui ({tui})", file=sys.stderr)
    sys.exit(1)
if latest["min_romm_api"] != api:
    print(f"release-check: compatibility.min_romm_api ({latest['min_romm_api']}) != romm-api ({api})", file=sys.stderr)
    sys.exit(1)

print("  OK compatibility matrix matches diverged crate versions")
PY

api_on_crates_io() {
  local version="$1"
  local status
  status="$(curl -fsS -o /dev/null -w '%{http_code}' \
    "https://crates.io/api/v1/crates/romm-api/${version}" 2>/dev/null || true)"
  [ "${status}" = "200" ]
}

if [ "${1:-}" != "versions-only" ]; then
  api_version="$(crate_version romm-api)"
  echo "==> Publish preflight: romm-api dry-run first"
  bash ./tools/publish-crate.sh --dry-run-only romm-api

  if api_on_crates_io "${api_version}"; then
    echo "==> romm-api@${api_version} on crates.io; dry-run frontends"
    bash ./tools/publish-crate.sh --dry-run-only romm-tui
    bash ./tools/publish-crate.sh --dry-run-only romm-cli
  else
    echo "==> romm-api@${api_version} not on crates.io yet; release-check frontends via build"
    echo "    (cargo publish --dry-run for frontends requires romm-api on the index first)"
    cargo build --release -p romm-tui
    cargo build --release -p romm-cli
  fi
fi

echo "release-check: all checks passed"