#!/bin/bash
set -euo pipefail

APP_NAME="mz-hyprnurs"
BIN_DIR="$HOME/.local/bin"
DESKTOP_DIR="$HOME/.local/share/applications"
ICON_DIR="$HOME/.local/share/icons/hicolor/256x256/apps"

echo "Uninstalling $APP_NAME..."

rm -f "$BIN_DIR/$APP_NAME"
rm -f "$DESKTOP_DIR/$APP_NAME.desktop"
rm -f "$ICON_DIR/$APP_NAME.png"

# Caches aktualisieren, damit der Eintrag aus den Launchern verschwindet
command -v update-desktop-database >/dev/null 2>&1 && update-desktop-database "$DESKTOP_DIR" >/dev/null 2>&1 || true
command -v gtk-update-icon-cache  >/dev/null 2>&1 && gtk-update-icon-cache -t "$HOME/.local/share/icons/hicolor" >/dev/null 2>&1 || true

echo "Done! $APP_NAME removed."
