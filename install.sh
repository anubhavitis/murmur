#!/bin/bash
set -euo pipefail

REPO="anubhavitis/murmur"
APP_PATH="/Applications/Murmur.app"
BIN_PATH="${APP_PATH}/Contents/MacOS/murmur"
PLIST_PATH="$HOME/Library/LaunchAgents/com.murmur.app.plist"
LOG_PATH="$HOME/.murmur/murmur.log"
OLD_BIN="$HOME/.murmur/bin/murmur"
TMP_DIR=""

cleanup() {
    [ -n "$TMP_DIR" ] && rm -rf "$TMP_DIR"
}
trap cleanup EXIT

info() { echo "  -> $1"; }
ok()   { echo "  ✓  $1"; }
err()  { echo "  ✗  $1"; }

echo ""
echo "  Murmur Installer"
echo "  ─────────────────"
echo ""

if [ "$(id -u)" -eq 0 ]; then
    err "Do not run this script with sudo."
    exit 1
fi

ARCH=$(uname -m)
if [ "$ARCH" != "arm64" ]; then
    err "Murmur currently supports Apple Silicon (arm64) only."
    err "Your architecture: $ARCH"
    exit 1
fi
ok "Architecture: arm64"

if [ "$(uname -s)" != "Darwin" ]; then
    err "Murmur currently supports macOS only."
    exit 1
fi
ok "Platform: macOS $(sw_vers -productVersion 2>/dev/null || echo 'unknown')"

echo ""

info "Fetching latest release..."
RELEASE_JSON=$(curl -sSf "https://api.github.com/repos/${REPO}/releases/latest" 2>/dev/null) || {
    err "Could not fetch latest version."
    err "GitHub API may be rate-limited. Try again in a few minutes, or download manually:"
    err "  https://github.com/${REPO}/releases/latest"
    exit 1
}
VERSION=$(echo "$RELEASE_JSON" | grep '"tag_name"' | sed 's/.*"v\(.*\)".*/\1/')
if [ -z "$VERSION" ]; then
    err "Could not parse version from GitHub release."
    exit 1
fi
ok "Latest version: v${VERSION}"

ARCHIVE="murmur-${VERSION}-aarch64-apple-darwin.zip"
URL="https://github.com/${REPO}/releases/download/v${VERSION}/${ARCHIVE}"
TMP_DIR=$(mktemp -d)

info "Downloading..."
curl -sSfL "$URL" -o "${TMP_DIR}/${ARCHIVE}" || {
    err "Download failed. Check if release v${VERSION} exists:"
    err "  https://github.com/${REPO}/releases"
    exit 1
}
ok "Downloaded ${ARCHIVE}"

info "Extracting..."
unzip -qo "${TMP_DIR}/${ARCHIVE}" -d "$TMP_DIR"

if [ ! -d "${TMP_DIR}/Murmur.app" ]; then
    err "Murmur.app not found in archive."
    exit 1
fi
ok "Extracted Murmur.app"

echo ""

# Stop existing instance
if launchctl list 2>/dev/null | grep -q "com.murmur.app"; then
    info "Stopping existing Murmur instance..."
    launchctl unload "$PLIST_PATH" 2>/dev/null || true
    sleep 1
    ok "Stopped previous instance"
fi

# Clean up old bare-binary install
if [ -f "$OLD_BIN" ]; then
    info "Removing old binary at ${OLD_BIN}..."
    rm -f "$OLD_BIN"
    rmdir "$HOME/.murmur/bin" 2>/dev/null || true
    ok "Old binary removed"
fi

# Install .app bundle
if [ -d "$APP_PATH" ] && [ ! -w "$APP_PATH" ]; then
    err "Cannot overwrite ${APP_PATH} — try: sudo rm -rf ${APP_PATH}"
    exit 1
fi
info "Installing to ${APP_PATH}..."
rm -rf "$APP_PATH"
mv "${TMP_DIR}/Murmur.app" "$APP_PATH"
xattr -cr "$APP_PATH" 2>/dev/null || true
ok "Murmur.app installed"

# Ensure config/models dir
mkdir -p "$HOME/.murmur"

# Install Launch Agent
info "Setting up Launch Agent (auto-start on login)..."
cat > "$PLIST_PATH" <<EOF
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>Label</key>
    <string>com.murmur.app</string>
    <key>Program</key>
    <string>${BIN_PATH}</string>
    <key>RunAtLoad</key>
    <true/>
    <key>KeepAlive</key>
    <dict>
        <key>SuccessfulExit</key>
        <false/>
    </dict>
    <key>ThrottleInterval</key>
    <integer>30</integer>
    <key>ProcessType</key>
    <string>Interactive</string>
    <key>StandardOutPath</key>
    <string>${LOG_PATH}</string>
    <key>StandardErrorPath</key>
    <string>${LOG_PATH}</string>
</dict>
</plist>
EOF
ok "Launch Agent installed"

info "Starting Murmur..."
launchctl load "$PLIST_PATH"
ok "Murmur is running"

echo ""
echo "  ✓  Murmur v${VERSION} installed successfully"
echo ""
echo "  ┌─────────────────────────────────────────────────────┐"
echo "  │  Murmur will automatically ask for permissions:     │"
echo "  │                                                     │"
echo "  │  1. Microphone        — click Allow when prompted   │"
echo "  │  2. Accessibility     — toggle on in Settings       │"
echo "  │  3. Input Monitoring  — toggle on in Settings       │"
echo "  │                                                     │"
echo "  │  Settings panes open automatically. Just toggle on. │"
echo "  │  Permissions persist across upgrades.               │"
echo "  └─────────────────────────────────────────────────────┘"
echo ""
echo "  Config:  ~/.murmur/config.json"
echo "  Models:  ~/.murmur/models/"
echo "  Logs:    ~/.murmur/murmur.log"
echo ""
echo "  Uninstall:"
echo "    curl -sSL https://raw.githubusercontent.com/${REPO}/main/uninstall.sh | sh"
echo ""
