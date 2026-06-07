#!/usr/bin/env bash
# Create aligned component tags and GitHub releases for a workspace version.
# Usage: ./tools/cut-workspace-release.sh 1.0.0 [commit-sha]
#
# After tags exist, publish crates.io (ordered) and frontend binaries:
#   gh workflow run release-please.yml -f ref=<sha-or-main> \
#     -f publish_romm_api=true -f publish_romm_tui=true -f publish_romm_cli=true
#   gh workflow run release-artifacts.yml -f tag=romm-cli-v<version>
#   gh workflow run release-artifacts.yml -f tag=romm-tui-v<version>
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

VERSION="${1:-}"
SHA="${2:-HEAD}"

if [ -z "$VERSION" ]; then
  echo "Usage: $0 <version> [commit-sha]" >&2
  exit 1
fi

RESOLVED="$(git rev-parse "$SHA")"
echo "==> Cutting workspace release ${VERSION} at ${RESOLVED}"

for component in romm-api romm-cli romm-tui; do
  tag="${component}-v${VERSION}"
  if git rev-parse "$tag" >/dev/null 2>&1; then
    existing="$(git rev-parse "$tag")"
    if [ "$existing" != "$RESOLVED" ]; then
      echo "Tag ${tag} exists at ${existing} (wanted ${RESOLVED})."
      echo "Delete and recreate locally, then push:"
      echo "  git tag -d ${tag}"
      echo "  git push origin :refs/tags/${tag}"
      echo "  git tag -a ${tag} -m \"${component} ${VERSION}\" ${RESOLVED}"
      echo "  git push origin ${tag}"
      exit 1
    fi
    echo "  OK tag ${tag} already at ${RESOLVED}"
  else
    echo "  Creating tag ${tag}"
    git tag -a "$tag" -m "${component} ${VERSION}" "$RESOLVED"
    git push origin "$tag"
  fi
done

notes_for() {
  local crate="$1"
  local changelog="${crate}/CHANGELOG.md"
  if [ ! -f "$changelog" ]; then
    echo "${crate} ${VERSION}"
    return
  fi
  awk -v ver="$VERSION" '
    $0 ~ "^## \\[" ver "\\]" { found=1; next }
    found && /^## \[/ { exit }
    found { print }
  ' "$changelog" | sed '/^$/d' | head -40
}

for component in romm-api romm-cli romm-tui; do
  tag="${component}-v${VERSION}"
  if gh release view "$tag" >/dev/null 2>&1; then
    echo "  OK GitHub release ${tag} exists"
  else
    echo "  Creating GitHub release ${tag}"
    notes="$(notes_for "$component")"
    gh release create "$tag" --title "${component} v${VERSION}" --notes "$notes"
  fi
done

echo ""
echo "==> Next: publish to crates.io (ordered)"
echo "gh workflow run release-please.yml -f ref=${RESOLVED} \\"
echo "  -f publish_romm_api=true -f publish_romm_tui=true -f publish_romm_cli=true"
echo ""
echo "==> Next: build frontend binaries"
echo "gh workflow run release-artifacts.yml -f tag=romm-cli-v${VERSION}"
echo "gh workflow run release-artifacts.yml -f tag=romm-tui-v${VERSION}"
