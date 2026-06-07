#!/usr/bin/env bash
# Bootstrap component release tags from a unified-era version (one-time maintainer step).
# Usage: ./tools/bootstrap-component-tags.sh 0.40.0 [commit-sha]
set -euo pipefail

VERSION="${1:-}"
SHA="${2:-HEAD}"

if [ -z "$VERSION" ]; then
  echo "Usage: $0 <version> [commit-sha]" >&2
  echo "Example: $0 0.40.0 v0.40.0" >&2
  exit 1
fi

for component in romm-api romm-cli romm-tui; do
  tag="${component}-v${VERSION}"
  echo "Creating annotated tag ${tag} at ${SHA}"
  git tag -a "$tag" -m "${component} ${VERSION} (bootstrap from unified release)" "$SHA"
done

echo ""
echo "Tags created locally. Push when ready:"
echo "  git push origin romm-api-v${VERSION} romm-cli-v${VERSION} romm-tui-v${VERSION}"
