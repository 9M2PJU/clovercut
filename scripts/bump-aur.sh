#!/usr/bin/env bash
# SPDX-License-Identifier: AGPL-3.0-or-later
# SPDX-FileCopyrightText: 2026 Clovercut contributors
#
# Bump the AUR package (assets/arch/PKGBUILD + .SRCINFO) to a release:
#
#   scripts/bump-aur.sh 0.1.9          bump to v0.1.9 (updates sums, .SRCINFO)
#   scripts/bump-aur.sh 0.1.9 --check  exit 0 if already on it, 1 otherwise
#
# Why a script and not a floating version: AUR packages pin pkgver and
# sha256sums by design - pacman verifies the downloaded bytes against them.
# A "-bin" package cannot track "latest" on its own; instead
# .github/workflows/aur.yml runs this script on every published release so
# `yay -S clovercut-bin` / `yay -Syu` always resolves to the newest release
# with hashes taken from that release's SHA256SUMS file.
set -euo pipefail

if [ "$#" -lt 1 ] || [ "$#" -gt 2 ]; then
  echo "usage: $0 VERSION [--check]" >&2
  exit 2
fi
ver="${1#v}"
check=false
[ "${2:-}" = "--check" ] && check=true

repo=$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)
pkgbuild="$repo/assets/arch/PKGBUILD"
current=$(sed -n 's/^pkgver=//p' "$pkgbuild" | head -1)

if [ "$current" = "$ver" ]; then
  # Sums may still have drifted; verify against the release file.
  sums_url="https://github.com/tulcoturo/clovercut/releases/download/v${ver}/SHA256SUMS"
  sums=$(curl -fsSL "$sums_url" 2>/dev/null || true)
  if [ -n "$sums" ]; then
    x86=$(awk '/x86_64\.AppImage$/ { print $1 }' <<<"$sums")
    arm=$(awk '/aarch64\.AppImage$/ { print $1 }' <<<"$sums")
    if grep -q "$x86" "$pkgbuild" && grep -q "$arm" "$pkgbuild"; then
      echo "bump-aur: already on $ver with matching sums"
      exit 0
    fi
  else
    echo "bump-aur: already on $ver (SHA256SUMS unreachable, sums unchecked)"
    exit 0
  fi
fi
if $check; then
  echo "bump-aur: PKGBUILD is on $current, asked for $ver" >&2
  exit 1
fi

echo "bump-aur: $current -> $ver"
sums_url="https://github.com/tulcoturo/clovercut/releases/download/v${ver}/SHA256SUMS"
sums=$(curl -fsSL "$sums_url") || { echo "bump-aur: cannot fetch $sums_url" >&2; exit 1; }
x86=$(awk '/x86_64\.AppImage$/ { print $1 }' <<<"$sums")
arm=$(awk '/aarch64\.AppImage$/ { print $1 }' <<<"$sums")
[ -n "$x86" ] && [ -n "$arm" ] || { echo "bump-aur: AppImage sums missing in $sums_url" >&2; exit 1; }

sed -i "s/^pkgver=.*/pkgver=${ver}/" "$pkgbuild"
python3 - "$pkgbuild" "$x86" "$arm" <<'EOF'
import re, sys
path, x86, arm = sys.argv[1], sys.argv[2], sys.argv[3]
text = open(path).read()
text = re.sub(r"(?m)^sha256sums_x86_64=\('[^']*'\)$",
              f"sha256sums_x86_64=('{x86}')", text, count=1)
text = re.sub(r"(?m)^sha256sums_aarch64=\('[^']*'\)$",
              f"sha256sums_aarch64=('{arm}')", text, count=1)
open(path, 'w').write(text)
EOF
grep -q "$x86" "$pkgbuild" && grep -q "$arm" "$pkgbuild" \
  || { echo "bump-aur: sum replacement failed" >&2; exit 1; }

# .SRCINFO must always match the PKGBUILD; AUR rejects mismatches.
if command -v makepkg >/dev/null 2>&1; then
  (cd "$(dirname "$pkgbuild")" && makepkg --printsrcinfo > .SRCINFO)
else
  echo "bump-aur: makepkg not found, .SRCINFO left stale - regenerate before pushing" >&2
  exit 1
fi
echo "bump-aur: now on $ver"
grep -n "^pkgver" "$pkgbuild"
