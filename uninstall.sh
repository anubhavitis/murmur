#!/bin/bash
set -euo pipefail

INSTALL_DIR="$HOME/.murmur/bin"
PLIST_PATH="$HOME/Library/LaunchAgents/com.murmur.app.plist"
MURMUR_DIR="$HOME/.murmur"

echo "Uninstalling Murmur..."

# Stop and unload Launch Agent
if launchctl list 2>/dev/null | grep -q "com.murmur.app"; then
    echo "Stopping Murmur..."
    launchctl unload "$PLIST_PATH" 2>/dev/null || true
fi
rm -f "$PLIST_PATH"

# Remove binary
rm -f "${INSTALL_DIR}/murmur"
rmdir "${INSTALL_DIR}" 2>/dev/null || true

echo "Murmur binary and Launch Agent removed."
echo ""

# Ask about config and models (handle non-interactive stdin)
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
echo "You may also want to remove 'murmur' from:"
echo "  System Settings > Privacy & Security > Input Monitoring"
echo "  System Settings > Privacy & Security > Microphone"
echo "  System Settings > Privacy & Security > Accessibility"
