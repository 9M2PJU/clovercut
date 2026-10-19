#!/usr/bin/env bash
# Clovercut release notes are reviewed alongside the code.
# Usage: release-notes.sh <since-ref> <until-ref> <version>
set -euo pipefail
repo=$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)
version=${3:?version required}
[[ "$version" =~ ^[0-9]+\.[0-9]+\.[0-9]+(-[0-9A-Za-z.]+)?$ ]] || exit 2
cat "$repo/docs/releases/${version%%-*}.md"
