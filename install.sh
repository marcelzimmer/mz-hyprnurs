#!/bin/bash
set -euo pipefail

APP_NAME="mz-hyprnurs"
BIN_DIR="$HOME/.local/bin"
DESKTOP_DIR="$HOME/.local/share/applications"
ICON_DIR="$HOME/.local/share/icons/hicolor/256x256/apps"
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"

echo "Installing $APP_NAME..."

# Prüfen ob eine fertige Binary neben diesem Skript liegt (Release-Zip)
# Falls ja, diese direkt nutzen — kein Cargo-Build notwendig
if [ -f "$SCRIPT_DIR/$APP_NAME" ]; then
    echo "Pre-built binary found, skipping build..."
    BINARY="$SCRIPT_DIR/$APP_NAME"
else
    echo "No pre-built binary found, building from source..."
    cargo build --release --manifest-path "$SCRIPT_DIR/Cargo.toml"
    BINARY="$SCRIPT_DIR/target/release/$APP_NAME"
fi

# Ordner erstellen
mkdir -p "$BIN_DIR" "$DESKTOP_DIR" "$ICON_DIR"

# Binary installieren
cp "$BINARY" "$BIN_DIR/$APP_NAME"
chmod +x "$BIN_DIR/$APP_NAME"

# Icon installieren (im Release-Zip liegt es neben dem Skript, im Source-Checkout unter assets/)
if [ -f "$SCRIPT_DIR/icon.png" ]; then
    ICON_SRC="$SCRIPT_DIR/icon.png"
elif [ -f "$SCRIPT_DIR/assets/icon.png" ]; then
    ICON_SRC="$SCRIPT_DIR/assets/icon.png"
else
    echo "Warnung: icon.png nicht gefunden, Icon wird nicht installiert"
    ICON_SRC=""
fi
if [ -n "$ICON_SRC" ]; then
    cp "$ICON_SRC" "$ICON_DIR/$APP_NAME.png"
fi

# Desktop-Datei erstellen
cat > "$DESKTOP_DIR/$APP_NAME.desktop" << EOF
[Desktop Entry]
Name=MZ-HyprNurs
Comment=Desktop-App für die pflegerische Übergabe im Krankenhaus
Exec=$BIN_DIR/$APP_NAME
Icon=$ICON_DIR/$APP_NAME.png
Type=Application
Categories=Office;MedicalSoftware;
Terminal=false
EOF

# Caches aktualisieren, damit Icon und Desktop-Eintrag sofort sichtbar sind
command -v update-desktop-database >/dev/null 2>&1 && update-desktop-database "$DESKTOP_DIR" >/dev/null 2>&1 || true
command -v gtk-update-icon-cache  >/dev/null 2>&1 && gtk-update-icon-cache -t "$HOME/.local/share/icons/hicolor" >/dev/null 2>&1 || true

echo "Done! $APP_NAME installed to $BIN_DIR/$APP_NAME"
echo "Desktop entry created. App should appear in your launcher."
