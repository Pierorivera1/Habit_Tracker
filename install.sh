#!/usr/bin/env bash
set -e

# Habitodo Desktop Installer
# Installs binary, desktop entry, and icon for current user

echo "==> Building Habitodo in release mode..."
cargo build --release

BIN_DIR="${HOME}/.local/bin"
APP_DIR="${HOME}/.local/share/applications"
ICON_DIR="${HOME}/.local/share/icons/hicolor/scalable/apps"

echo "==> Creating destination directories..."
mkdir -p "${BIN_DIR}"
mkdir -p "${APP_DIR}"
mkdir -p "${ICON_DIR}"

echo "==> Installing binary to ${BIN_DIR}/habitodo..."
rm -f "${BIN_DIR}/habitodo"
install -m 755 target/release/habitodo "${BIN_DIR}/habitodo"

echo "==> Installing application icon to ${ICON_DIR}/habitodo.svg..."
cp assets/habitodo.svg "${ICON_DIR}/habitodo.svg"

echo "==> Installing desktop entry to ${APP_DIR}/habitodo.desktop..."
cp assets/habitodo.desktop "${APP_DIR}/habitodo.desktop"

# Ensure ~/.local/bin is in PATH notice
if [[ ":$PATH:" != *":${HOME}/.local/bin:"* ]]; then
  echo ""
  echo "NOTE: ${HOME}/.local/bin is not in your current PATH."
  echo "Add it to your shell config (~/.bashrc or ~/.zshrc):"
  echo "  export PATH=\"\$HOME/.local/bin:\$PATH\""
  echo ""
fi

# Refresh desktop database if tool is installed
if command -v update-desktop-database >/dev/null 2>&1; then
  update-desktop-database "${APP_DIR}" 2>/dev/null || true
fi

# Refresh icon cache if tool is installed
if command -v gtk-update-icon-cache >/dev/null 2>&1; then
  gtk-update-icon-cache -f -t "${HOME}/.local/share/icons/hicolor" 2>/dev/null || true
fi

echo "==> Successfully installed Habitodo!"
echo "You can now launch Habitodo from your system application menu or by running 'habitodo' in your terminal."
