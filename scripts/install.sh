#!/usr/bin/env sh
# SPDX-License-Identifier: AGPL-3.0-or-later
# SPDX-FileCopyrightText: 2026 Clovercut contributors
#
# One-line installer for Clovercut on Linux:
#
#   curl -fsSL https://raw.githubusercontent.com/tulcoturo/clovercut/main/scripts/install.sh | sh
#
# Picks the native package for the distro when one exists (.deb via apt,
# .rpm via dnf), the AUR helper on Arch, and falls back to the portable
# AppImage otherwise. Environment overrides:
#
#   VERSION=v0.1.8   release tag to install (default: latest)
#   PREFIX=~/.local install prefix for the AppImage fallback (default: /usr/local when root)
set -eu

REPO="tulcoturo/clovercut"
APP="clovercut"

die() { echo "install: $*" >&2; exit 1; }
need() { command -v "$1" >/dev/null 2>&1 || die "need '$1' to install clovercut"; }

arch="$(uname -m)"
case "$arch" in
  x86_64) release_arch="x86_64" ;;
  aarch64|arm64) release_arch="aarch64" ;;
  *) die "unsupported architecture: $arch" ;;
esac

if [ "$(uname -s)" != "Linux" ]; then
  die "this installer covers Linux; grab your build from https://github.com/${REPO}/releases"
fi

# Resolve the tag: explicit VERSION wins, otherwise the latest release.
if [ -n "${VERSION:-}" ]; then
  case "$VERSION" in
    v*) TAG="$VERSION" ;;
    *) TAG="v$VERSION" ;;
  esac
  VER="${TAG#v}"
else
  need curl
  TAG="$(curl -fsSL "https://api.github.com/repos/${REPO}/releases/latest" | sed -n 's/.*"tag_name": *"\([^"]*\)".*/\1/p' | head -1)"
  [ -n "$TAG" ] || die "could not resolve the latest release"
  VER="${TAG#v}"
fi

BASE="https://github.com/${REPO}/releases/download/${TAG}"
have() { command -v "$1" >/dev/null 2>&1; }

install_deb() {
  need curl
  tmp="$(mktemp --suffix=.deb)"
  trap 'rm -f "$tmp"' EXIT INT TERM
  echo "Downloading Clovercut ${VER} (.deb)…"
  curl -fsSL -o "$tmp" "${BASE}/Clovercut-${VER}-${release_arch}.deb"
  if have sudo && [ "$(id -u)" -ne 0 ]; then SUDO=sudo; else SUDO=""; fi
  $SUDO apt-get update -qq || true
  $SUDO apt-get install -y "$tmp"
  echo "Installed clovercut ${VER}. Run it with: clovercut"
}

install_rpm() {
  need curl
  tmp="$(mktemp --suffix=.rpm)"
  trap 'rm -f "$tmp"' EXIT INT TERM
  echo "Downloading Clovercut ${VER} (.rpm)…"
  curl -fsSL -o "$tmp" "${BASE}/Clovercut-${VER}-${release_arch}.rpm"
  if have sudo && [ "$(id -u)" -ne 0 ]; then SUDO=sudo; else SUDO=""; fi
  if have dnf; then
    $SUDO dnf install -y "$tmp"
  else
    $SUDO rpm -Uvh "$tmp"
  fi
  echo "Installed clovercut ${VER}. Run it with: clovercut"
}

install_aur() {
  if have yay; then HELPER=yay
  elif have paru; then HELPER=paru
  else
    echo "No AUR helper found (yay/paru). Installing the AppImage instead."
    echo "Or install an AUR helper, then run: ${HELPER:-yay} -S clovercut-bin"
    install_appimage
    return
  fi
  echo "Installing clovercut-bin ${VER} from the AUR with ${HELPER}…"
  $HELPER -S --noconfirm clovercut-bin
  echo "Installed clovercut ${VER}. Run it with: clovercut"
}

install_appimage() {
  need curl
  if [ -n "${PREFIX:-}" ]; then prefix="$PREFIX";
  elif [ "$(id -u)" -eq 0 ]; then prefix="/usr/local";
  else prefix="$HOME/.local"; fi
  mkdir -p "$prefix/bin"
  echo "Downloading Clovercut ${VER} (AppImage)…"
  curl -fsSL -o "$prefix/bin/$APP" "${BASE}/Clovercut-${VER}-${release_arch}.AppImage"
  chmod +x "$prefix/bin/$APP"
  case ":$PATH:" in
    *":$prefix/bin:"*) ;;
    *) echo "Note: $prefix/bin is not on your PATH yet." ;;
  esac
  echo "Installed $prefix/bin/$APP. Run it with: $APP"
}

# Distro routing: native package first, AppImage as the fallback.
if [ -f /etc/os-release ]; then
  # shellcheck disable=SC1091
  . /etc/os-release
  id_like="${ID_LIKE:-} $ID"
else
  id_like=""
fi

case "$id_like" in
  *arch*|*cachyos*|*manjaro*|*endeavouros*) install_aur ;;
  *debian*|*ubuntu*|*mint*|*pop*) install_deb ;;
  *fedora*|*rhel*|*centos*|*suse*|*opensuse*) install_rpm ;;
  *)
    if have apt-get; then install_deb
    elif have dnf; then install_rpm
    elif have pacman; then install_aur
    else install_appimage
    fi
    ;;
esac
