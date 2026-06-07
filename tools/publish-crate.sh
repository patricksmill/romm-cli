#!/usr/bin/env bash
# Publish one workspace crate to crates.io (idempotent on duplicate version).
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

usage() {
  echo "usage: $0 [--dry-run-only] <crate>" >&2
  exit 1
}

dry_run_only="false"
crate=""

while [ $# -gt 0 ]; do
  case "$1" in
    --dry-run-only)
      dry_run_only="true"
      shift
      ;;
    -h | --help)
      usage
      ;;
    *)
      if [ -n "$crate" ]; then
        usage
      fi
      crate="$1"
      shift
      ;;
  esac
done

if [ -z "$crate" ]; then
  usage
fi

case "$crate" in
  romm-api | romm-cli | romm-tui) ;;
  *)
    echo "publish-crate: unknown crate: $crate" >&2
    exit 1
    ;;
esac

echo "==> Dry-run publish ${crate}"
cargo publish -p "${crate}" --dry-run --allow-dirty

if [ "$dry_run_only" = "true" ]; then
  echo "publish-crate: dry-run only; skipping real publish"
  exit 0
fi

if [ -z "${CARGO_REGISTRY_TOKEN:-}" ]; then
  echo "publish-crate: CARGO_REGISTRY_TOKEN is not set" >&2
  exit 1
fi

echo "==> Publishing ${crate}"
set +e
output="$(cargo publish -p "${crate}" 2>&1)"
status=$?
echo "${output}"
if [ "${status}" -ne 0 ]; then
  if echo "${output}" | grep -q "already exists on crates.io index"; then
    echo "publish-crate: ${crate} version already published; treating as success."
    exit 0
  fi
  exit "${status}"
fi