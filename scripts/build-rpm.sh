#!/usr/bin/env bash
# SPDX-License-Identifier: AGPL-3.0-or-later
# SPDX-FileCopyrightText: 2026 Clovercut contributors
#
# Build an .rpm from the staged Linux folder CI produces (the same folder
# the AppImage is made of), using only rpmbuild:
#
#   scripts/build-rpm.sh stage/Clovercut-0.1.8-linux-x86_64 0.1.8 [out.rpm]
#
# Layout mirrors the .deb: everything under /opt/clovercut, a launcher on
# the PATH, a desktop entry and an icon where the desktop looks for them.
set -euo pipefail

if [ "$#" -lt 2 ] || [ "$#" -gt 3 ]; then
  echo "usage: $0 STAGE VERSION [OUTPUT.rpm]" >&2
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
  x86_64) rpm_arch=x86_64 ;;
  aarch64) rpm_arch=aarch64 ;;
  *) echo "unsupported architecture: $(uname -m)" >&2; exit 1 ;;
esac

out=${3:-"Clovercut-${version}-x86_64.rpm"}
case "$out" in
  *aarch64*) rpm_arch=aarch64 ;;
  *x86_64*) rpm_arch=x86_64 ;;
esac
mkdir -p "$(dirname "$out")"
out=$(realpath -m "$out")

work=$(mktemp -d)
trap 'rm -rf "$work"' EXIT
topdir="$work/rpm"
mkdir -p "$topdir"/{BUILD,RPMS,SOURCES,SPECS,SRPMS}
root="$work/root"
mkdir -p "$root/opt/clovercut/lib" "$root/usr/bin" \
  "$root/usr/share/applications" \
  "$root/usr/share/icons/hicolor/256x256/apps" \
  "$root/usr/share/doc/clovercut"

cp "$stage/clovercut" "$root/opt/clovercut/"
cp -a "$stage"/lib/. "$root/opt/clovercut/lib/"
printf '#!/bin/sh\nexec /opt/clovercut/clovercut "$@"\n' > "$root/usr/bin/clovercut"
chmod 755 "$root/usr/bin/clovercut"
cp "$stage/io.github.tulcoturo.clovercut.desktop" "$root/usr/share/applications/"
cp "$stage/clovercut.png" "$root/usr/share/icons/hicolor/256x256/apps/"
cp "$stage/LICENSE" "$stage/THIRD_PARTY_NOTICES.md" "$root/usr/share/doc/clovercut/"

cat > "$topdir/SPECS/clovercut.spec" <<SPEC
Name: clovercut
Version: ${version}
Release: 1%{?dist}
Summary: Desktop-first, open-source video editor with local AI
License: AGPL-3.0-or-later
URL: https://github.com/tulcoturo/clovercut
BuildArch: ${rpm_arch}
Requires: gtk3, fontconfig, freetype, libxkbcommon, libxkbcommon-x11, mesa-libGL, alsa-lib
%description
Timeline editing, effects, transitions, titles and on-device captions,
with FFmpeg underneath and hardware-accelerated preview and export.
%files
/opt/clovercut/
/usr/bin/clovercut
/usr/share/applications/io.github.tulcoturo.clovercut.desktop
/usr/share/icons/hicolor/256x256/apps/clovercut.png
/usr/share/doc/clovercut/
SPEC

rpmbuild -bb --define "_topdir $topdir" \
  --define "_buildrootdir $work/root" \
  --buildroot "$root" \
  "$topdir/SPECS/clovercut.spec"

built=$(find "$topdir/RPMS" -name '*.rpm' | head -1)
test -n "$built" || { echo "rpmbuild produced no .rpm" >&2; exit 1; }
cp "$built" "$out"
echo "Created $out"
