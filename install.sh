#!/usr/bin/env bash
set -e

# Habit Tracker Desktop Installer
# Installs binary, desktop entry, and icon for current user

echo "==> Building Habit Tracker in release mode..."
cargo build --release

BIN_DIR="${HOME}/.local/bin"
APP_DIR="${HOME}/.local/share/applications"
ICON_DIR="${HOME}/.local/share/icons/hicolor/scalable/apps"

echo "==> Creating destination directories..."
mkdir -p "${BIN_DIR}"
mkdir -p "${APP_DIR}"
mkdir -p "${ICON_DIR}"

echo "==> Installing binary to ${BIN_DIR}/habit-tracker..."
rm -f "${BIN_DIR}/habit-tracker"
install -m 755 target/release/habitodo "${BIN_DIR}/habit-tracker"

# Clean up legacy habitodo desktop files if present
rm -f "${APP_DIR}/habitodo.desktop"

echo "==> Installing application icon to ${ICON_DIR}/habit-tracker.svg..."
cp assets/habit-tracker.svg "${ICON_DIR}/habit-tracker.svg"

echo "==> Installing desktop entry to ${APP_DIR}/habit-tracker.desktop..."
cp assets/habit-tracker.desktop "${APP_DIR}/habit-tracker.desktop"

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

echo "==> Successfully installed Habit Tracker!"
echo "You can now launch Habit Tracker from your system application menu or by running 'habit-tracker' in your terminal."
