#!/usr/bin/env bash
# SPDX-License-Identifier: AGPL-3.0-or-later
# SPDX-FileCopyrightText: 2026 Clovercut contributors
#
# Build a .deb from the staged Linux folder CI produces (the same folder the
# AppImage is made of), using only dpkg-deb - no nfpm needed:
#
#   scripts/build-deb.sh stage/Clovercut-0.1.8-linux-x86_64 0.1.8 [out.deb]
#
# The program lives whole under /opt/clovercut - the binary finds its
# libraries in lib/ beside itself by rpath - with a launcher on the PATH,
# a desktop entry and an icon where the desktop looks for them.
set -euo pipefail

if [ "$#" -lt 2 ] || [ "$#" -gt 3 ]; then
  echo "usage: $0 STAGE VERSION [OUTPUT.deb]" >&2
  exit 2
fi
stage=$(realpath "$1")
version="$2"
for file in clovercut clovercut.png io.github.tulcoturo.clovercut.desktop LICENSE THIRD_PARTY_NOTICES.md; do
  test -f "$stage/$file" || { echo "missing stage file: $file" >&2; exit 1; }
done
test -x "$stage/clovercut" || { echo "stage binary is not executable: $stage/clovercut" >&2; exit 1; }
test -d "$stage/lib" || { echo "missing stage dir: $stage/lib" >&2; exit 1; }

case "$(uname -m)" in
  x86_64) deb_arch=amd64 ;;
  aarch64) deb_arch=arm64 ;;
  *) echo "unsupported architecture: $(uname -m)" >&2; exit 1 ;;
esac

out=${3:-"Clovercut-${version}-x86_64.deb"}
case "$out" in
  *aarch64*|*arm64*) deb_arch=arm64 ;;
  *x86_64*|*amd64*) deb_arch=amd64 ;;
esac
mkdir -p "$(dirname "$out")"
out=$(realpath -m "$out")

work=$(mktemp -d)
trap 'rm -rf "$work"' EXIT
pkg="$work/pkg"
mkdir -p "$pkg/opt/clovercut/lib" "$pkg/usr/bin" \
  "$pkg/usr/share/applications" \
  "$pkg/usr/share/icons/hicolor/256x256/apps" \
  "$pkg/usr/share/doc/clovercut" "$pkg/DEBIAN"

cp "$stage/clovercut" "$pkg/opt/clovercut/"
cp -a "$stage"/lib/. "$pkg/opt/clovercut/lib/"
printf '#!/bin/sh\nexec /opt/clovercut/clovercut "$@"\n' > "$pkg/usr/bin/clovercut"
chmod 755 "$pkg/usr/bin/clovercut"
cp "$stage/io.github.tulcoturo.clovercut.desktop" "$pkg/usr/share/applications/"
cp "$stage/clovercut.png" "$pkg/usr/share/icons/hicolor/256x256/apps/"
cp "$stage/LICENSE" "$stage/THIRD_PARTY_NOTICES.md" "$pkg/usr/share/doc/clovercut/"

installed_size=$(du -sk "$pkg/opt/clovercut" "$pkg/usr" | awk '{ s += $1 } END { print s }')
cat > "$pkg/DEBIAN/control" <<CONTROL
Package: clovercut
Version: ${version}
Section: video
Priority: optional
Architecture: ${deb_arch}
Maintainer: Clovercut contributors <https://github.com/tulcoturo/clovercut>
Homepage: https://github.com/tulcoturo/clovercut
Description: Desktop-first, open-source video editor with local AI
 Timeline editing, effects, transitions, titles and on-device captions,
 with FFmpeg underneath and hardware-accelerated preview and export.
Depends: libc6, libgtk-3-0, libfontconfig1, libfreetype6, libxkbcommon0, libxkbcommon-x11-0, libgl1, libasound2 | libasound2t64
Installed-Size: ${installed_size}
CONTROL

dpkg-deb --build "$pkg" "$out"
echo "Created $out"
