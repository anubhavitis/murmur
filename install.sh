#!/bin/bash
set -euo pipefail

REPO="anubhavitis/murmur"
INSTALL_DIR="$HOME/.murmur/bin"
PLIST_PATH="$HOME/Library/LaunchAgents/com.murmur.app.plist"
LOG_PATH="$HOME/.murmur/murmur.log"

echo "Installing Murmur..."

# Check architecture
ARCH=$(uname -m)
if [ "$ARCH" != "arm64" ]; then
    echo "Error: Murmur currently supports Apple Silicon (arm64) only."
    echo "Your architecture: $ARCH"
    exit 1
fi

# Check macOS
if [ "$(uname -s)" != "Darwin" ]; then
    echo "Error: Murmur currently supports macOS only."
    exit 1
fi

# Get latest version
echo "Fetching latest version..."
VERSION=$(curl -sSf "https://api.github.com/repos/${REPO}/releases/latest" | grep '"tag_name"' | sed 's/.*"v\(.*\)".*/\1/' || "")
if [ -z "$VERSION" ]; then
    echo "Error: Could not fetch latest version. Check your internet connection."
    exit 1
fi
echo "Latest version: v${VERSION}"

# Download
ARCHIVE="murmur-${VERSION}-aarch64-apple-darwin.tar.gz"
URL="https://github.com/${REPO}/releases/download/v${VERSION}/${ARCHIVE}"
TMP_DIR=$(mktemp -d)

echo "Downloading..."
curl -sSfL "$URL" -o "${TMP_DIR}/${ARCHIVE}"

echo "Extracting..."
tar -xzf "${TMP_DIR}/${ARCHIVE}" -C "$TMP_DIR"

# Install binary
mkdir -p "$INSTALL_DIR"
mv "${TMP_DIR}/murmur" "${INSTALL_DIR}/murmur"
chmod +x "${INSTALL_DIR}/murmur"
xattr -cr "${INSTALL_DIR}/murmur" 2>/dev/null || true

# Clean up temp
rm -rf "$TMP_DIR"

# Unload existing Launch Agent if present
if launchctl list | grep -q "com.murmur.app" 2>/dev/null; then
    echo "Stopping existing Murmur instance..."
    launchctl unload "$PLIST_PATH" 2>/dev/null || true
fi

# Install Launch Agent
cat > "$PLIST_PATH" <<EOF
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>Label</key>
    <string>com.murmur.app</string>
    <key>Program</key>
    <string>${INSTALL_DIR}/murmur</string>
    <key>RunAtLoad</key>
    <true/>
    <key>KeepAlive</key>
    <dict>
        <key>SuccessfulExit</key>
        <false/>
    </dict>
    <key>ProcessType</key>
    <string>Interactive</string>
    <key>StandardOutPath</key>
    <string>${LOG_PATH}</string>
    <key>StandardErrorPath</key>
    <string>${LOG_PATH}</string>
</dict>
</plist>
EOF

# Load Launch Agent
launchctl load "$PLIST_PATH"

echo ""
echo "=== Murmur v${VERSION} installed ==="
echo ""
echo "Murmur is now running in your menubar."
echo ""
echo "IMPORTANT: Grant these permissions in System Settings > Privacy & Security:"
echo "  1. Input Monitoring  — for hotkey detection"
echo "  2. Microphone        — for audio capture"
echo "  3. Accessibility     — for paste-at-cursor"
echo ""
echo "Look for 'murmur' in each permission list and toggle it on."
echo "You may need to restart Murmur after granting permissions."
echo ""
echo "To uninstall: curl -sSL https://raw.githubusercontent.com/${REPO}/main/uninstall.sh | sh"
