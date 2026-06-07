#!/usr/bin/env bash
# Publish workspace crates to crates.io: romm-api first, then romm-tui and romm-cli in parallel.
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

PUBLISH_SCRIPT="${ROOT}/tools/publish-crate.sh"
WAIT_SCRIPT="${ROOT}/tools/wait-for-crates-io.sh"

usage() {
  cat >&2 <<USAGE
usage: publish-workspace.sh [--if-created] [--crates crate ...]

  --if-created   Use PUBLISH_API, PUBLISH_TUI, PUBLISH_CLI env vars (true/false).
  --crates       Publish only listed crates (api first; frontends in parallel).

With no filter flags, all three crates are candidates for publish.
USAGE
  exit 1
}

should_publish() {
  local crate="$1"
  case "$crate" in
    romm-api)
      [ "${PUBLISH_API:-true}" = "true" ]
      ;;
    romm-tui)
      [ "${PUBLISH_TUI:-true}" = "true" ]
      ;;
    romm-cli)
      [ "${PUBLISH_CLI:-true}" = "true" ]
      ;;
    *)
      return 1
      ;;
  esac
}

if_created="false"
declare -a crate_filter=()

while [ $# -gt 0 ]; do
  case "$1" in
    --if-created)
      if_created="true"
      shift
      ;;
    --crates)
      shift
      while [ $# -gt 0 ] && [[ "$1" != --* ]]; do
        crate_filter+=("$1")
        shift
      done
      ;;
    -h | --help)
      usage
      ;;
    *)
      echo "publish-workspace: unknown argument: $1" >&2
      usage
      ;;
  esac
done

if [ "$if_created" = "true" ]; then
  :
elif [ "${#crate_filter[@]}" -gt 0 ]; then
  PUBLISH_API="false"
  PUBLISH_TUI="false"
  PUBLISH_CLI="false"
  for crate in "${crate_filter[@]}"; do
    case "$crate" in
      romm-api) PUBLISH_API="true" ;;
      romm-tui) PUBLISH_TUI="true" ;;
      romm-cli) PUBLISH_CLI="true" ;;
      *)
        echo "publish-workspace: unknown crate: $crate" >&2
        exit 1
        ;;
    esac
  done
else
  PUBLISH_API="true"
  PUBLISH_TUI="true"
  PUBLISH_CLI="true"
fi

api_version="$(grep -E '^version = ' romm-api/Cargo.toml | head -1 | tr -d '\r' | sed 's/version = "\(.*\)"/\1/')"
needs_frontend="false"
if should_publish romm-tui || should_publish romm-cli; then
  needs_frontend="true"
fi

if should_publish romm-api; then
  "${PUBLISH_SCRIPT}" romm-api
elif [ "$needs_frontend" = "true" ]; then
  echo "==> Skipping romm-api publish; waiting for existing ${api_version} on crates.io"
  "${WAIT_SCRIPT}" romm-api "${api_version}"
fi

publish_frontend() {
  local crate="$1"
  if ! should_publish romm-api; then
    "${WAIT_SCRIPT}" romm-api "${api_version}"
  fi
  "${PUBLISH_SCRIPT}" "${crate}"
}

declare -a frontend_pids=()

if should_publish romm-tui; then
  publish_frontend romm-tui &
  frontend_pids+=("$!")
fi

if should_publish romm-cli; then
  publish_frontend romm-cli &
  frontend_pids+=("$!")
fi

if [ "${#frontend_pids[@]}" -gt 0 ]; then
  fail=0
  for pid in "${frontend_pids[@]}"; do
    wait "${pid}" || fail=1
  done
  if [ "${fail}" -ne 0 ]; then
    exit 1
  fi
fi

if ! should_publish romm-api && ! should_publish romm-tui && ! should_publish romm-cli; then
  echo "publish-workspace: no crates selected; nothing to do"
fi

echo "publish-workspace: done"
