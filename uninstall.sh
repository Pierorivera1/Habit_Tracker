#!/usr/bin/env bash
set -e

# Habitodo Desktop Uninstaller
echo "==> Removing Habitodo files..."

rm -f "${HOME}/.local/bin/habitodo"
rm -f "${HOME}/.local/share/applications/habitodo.desktop"
rm -f "${HOME}/.local/share/icons/hicolor/scalable/apps/habitodo.svg"

if command -v update-desktop-database >/dev/null 2>&1; then
  update-desktop-database "${HOME}/.local/share/applications" 2>/dev/null || true
fi

if command -v gtk-update-icon-cache >/dev/null 2>&1; then
  gtk-update-icon-cache -f -t "${HOME}/.local/share/icons/hicolor" 2>/dev/null || true
fi

echo "==> Habitodo has been uninstalled."
echo "(Your personal data at ~/.local/share/habitodo/ remains untouched.)"
