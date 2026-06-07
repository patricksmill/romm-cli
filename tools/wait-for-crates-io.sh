#!/usr/bin/env bash
# Wait until a crate version is visible on crates.io.
set -euo pipefail

usage() {
  echo "usage: $0 <crate> <version> [timeout_seconds]" >&2
  exit 1
}

crate="${1:-}"
version="${2:-}"
timeout="${3:-300}"

if [ -z "$crate" ] || [ -z "$version" ]; then
  usage
fi

url="https://crates.io/api/v1/crates/${crate}/${version}"
deadline=$((SECONDS + timeout))

echo "==> Waiting for ${crate}@${version} on crates.io (timeout ${timeout}s)"

while [ "$SECONDS" -lt "$deadline" ]; do
  status="$(curl -fsS -o /dev/null -w '%{http_code}' "$url" 2>/dev/null || true)"
  if [ "$status" = "200" ]; then
    echo "==> ${crate}@${version} is available on crates.io"
    exit 0
  fi
  sleep 5
done

echo "wait-for-crates-io: timed out waiting for ${crate}@${version}" >&2
exit 1