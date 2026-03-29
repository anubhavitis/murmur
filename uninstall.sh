#!/bin/bash
set -euo pipefail

APP_PATH="/Applications/Murmur.app"
OLD_BIN="$HOME/.murmur/bin/murmur"
PLIST_PATH="$HOME/Library/LaunchAgents/com.murmur.app.plist"
MURMUR_DIR="$HOME/.murmur"

echo "Uninstalling Murmur..."

# Stop and unload Launch Agent
if launchctl list 2>/dev/null | grep -q "com.murmur.app"; then
    echo "Stopping Murmur..."
    launchctl unload "$PLIST_PATH" 2>/dev/null || true
fi
rm -f "$PLIST_PATH"

# Remove .app bundle
rm -rf "$APP_PATH"

# Remove old bare-binary install if present
rm -f "$OLD_BIN"
rmdir "$HOME/.murmur/bin" 2>/dev/null || true

echo "Murmur.app and Launch Agent removed."
echo ""

if [ -t 0 ]; then
    read -p "Remove config and models too? (~/.murmur/) [y/N] " -n 1 -r
    echo ""
else
    REPLY="n"
    echo "Non-interactive mode: keeping config and models at ${MURMUR_DIR}"
    echo "To remove manually: rm -rf ${MURMUR_DIR}"
fi

if [[ $REPLY =~ ^[Yy]$ ]]; then
    rm -rf "$MURMUR_DIR"
    echo "Config and models removed."
else
    echo "Config and models kept at ${MURMUR_DIR}"
fi

echo ""
echo "Murmur uninstalled."
echo "You may also want to remove 'Murmur' from:"
echo "  System Settings > Privacy & Security > Input Monitoring"
echo "  System Settings > Privacy & Security > Microphone"
echo "  System Settings > Privacy & Security > Accessibility"
