#!/usr/bin/env bash
# SPDX-License-Identifier: AGPL-3.0-or-later
# SPDX-FileCopyrightText: 2026 Clovercut contributors
# Package a self-contained stage (clovercut, lib/, desktop entry and icon).
# Usage: scripts/build-appimage.sh STAGE x86_64|aarch64 OUTPUT.AppImage
set -euo pipefail
if [ "$#" -ne 3 ]; then
  echo "usage: $0 STAGE x86_64|aarch64 OUTPUT.AppImage" >&2
  exit 2
fi
repo=$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)
stage=$(realpath "$1")
export ARCH="$2"
case "$ARCH" in x86_64|aarch64) ;; *) echo "unsupported architecture: $ARCH" >&2; exit 2 ;; esac
mkdir -p "$(dirname "$3")"
output=$(realpath -m "$3")
for file in clovercut clovercut.png io.github.tulcoturo.clovercut.desktop LICENSE THIRD_PARTY_NOTICES.md; do
  test -f "$stage/$file" || { echo "missing stage file: $file" >&2; exit 1; }
done
test -x "$stage/clovercut"
test -d "$stage/lib"
work=$(mktemp -d)
trap 'rm -rf "$work"' EXIT
appdir="$work/Clovercut.AppDir"
mkdir -p "$appdir/usr/bin" "$appdir/usr/share/applications" \
  "$appdir/usr/share/icons/hicolor/256x256/apps" "$appdir/usr/share/doc/clovercut"
cp "$stage/clovercut" "$appdir/usr/bin/"
cp -a "$stage/lib" "$appdir/usr/bin/"
# Keep the X11 keyboard libraries from the same build together.
for lib in libxkbcommon.so.0 libxkbcommon-x11.so.0 libxcb-xkb.so.1; do
  path=$(ldconfig -p | awk -v lib="$lib" '$1 == lib && !found { found=1; path=$NF } END { print path }')
  test -n "$path" || { echo "missing system library: $lib" >&2; exit 1; }
  cp -L "$path" "$appdir/usr/bin/lib/$lib"
done
cp "$stage/io.github.tulcoturo.clovercut.desktop" "$appdir/"
cp "$stage/io.github.tulcoturo.clovercut.desktop" "$appdir/usr/share/applications/"
cp "$stage/clovercut.png" "$appdir/"
cp "$stage/clovercut.png" "$appdir/usr/share/icons/hicolor/256x256/apps/"
cp "$stage/LICENSE" "$stage/THIRD_PARTY_NOTICES.md" "$appdir/usr/share/doc/clovercut/"
ln -s clovercut.png "$appdir/.DirIcon"
ln -s usr/bin/clovercut "$appdir/AppRun"
"$repo/scripts/ci/fetch.sh" "appimagetool-$ARCH" "$work/appimagetool"
"$repo/scripts/ci/fetch.sh" "appimage-runtime-$ARCH" "$work/appimage-runtime"
chmod +x "$work/appimagetool"
"$work/appimagetool" --appimage-extract-and-run --runtime-file "$work/appimage-runtime" "$appdir" "$output"
chmod +x "$output"
echo "Created $output"
