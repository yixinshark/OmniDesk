#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_DIR="$(cd "$SCRIPT_DIR/../.." && pwd)"
DEB_PATH="${1:-$PROJECT_DIR/src-tauri/target/release/bundle/deb/tauri-app_0.1.0_amd64.deb}"

if [ ! -f "$DEB_PATH" ]; then
  echo "deb package not found: $DEB_PATH" >&2
  exit 1
fi

WORK_DIR="$(mktemp -d)"
trap 'rm -rf "$WORK_DIR"' EXIT

PKG_DIR="$WORK_DIR/package"
dpkg-deb -R "$DEB_PATH" "$PKG_DIR"

# OmniDesk replaces the desktop layer and should not appear as a normal launcher.
rm -f "$PKG_DIR/usr/share/applications/tauri-app.desktop"
rmdir --ignore-fail-on-non-empty "$PKG_DIR/usr/share/applications" 2>/dev/null || true

WANTS_DIR="$PKG_DIR/usr/lib/systemd/user/dde-session-core.target.wants"
mkdir -p "$WANTS_DIR"
chmod 755 "$WANTS_DIR"
rm -f "$WANTS_DIR/tauri-app.service"
ln -s ../tauri-app.service "$WANTS_DIR/tauri-app.service"

(
  cd "$PKG_DIR"
  find . -path './DEBIAN' -prune -o -type f -printf '%P\0' \
    | sort -z \
    | xargs -0 md5sum > DEBIAN/md5sums
)

TMP_DEB="$WORK_DIR/$(basename "$DEB_PATH")"
dpkg-deb --root-owner-group -b "$PKG_DIR" "$TMP_DEB"
mv "$TMP_DEB" "$DEB_PATH"

echo "postprocessed deb package: $DEB_PATH"
