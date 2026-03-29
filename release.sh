#!/bin/bash
set -euo pipefail

VERSION=$(grep '^version' Cargo.toml | head -1 | sed 's/.*"\(.*\)".*/\1/')
BINARY="target/release/murmur"
ARCHIVE="murmur-${VERSION}-aarch64-apple-darwin.tar.gz"

echo "Building murmur v${VERSION} (release)..."
cargo build --release

echo "Stripping binary..."
strip "$BINARY"

echo "Creating archive..."
tar -czf "$ARCHIVE" -C target/release murmur

SHA=$(shasum -a 256 "$ARCHIVE" | awk '{print $1}')

echo ""
echo "=== Release Ready ==="
echo "Version:  ${VERSION}"
echo "Archive:  ${ARCHIVE}"
echo "SHA256:   ${SHA}"
echo "Size:     $(du -h "$ARCHIVE" | awk '{print $1}')"
echo ""

# Generate cask formula
CASK_FILE="dist/murmur.rb"
sed -e "s/__VERSION__/${VERSION}/g" -e "s/__SHA256__/${SHA}/g" dist/murmur.rb.template > "$CASK_FILE"
echo "Cask formula generated: ${CASK_FILE}"
echo ""

if command -v gh &>/dev/null; then
    echo "GitHub CLI found. To create a release:"
    echo "  gh release create v${VERSION} ${ARCHIVE} --title \"v${VERSION}\" --notes \"Release v${VERSION}\""
else
    echo "Upload ${ARCHIVE} to: https://github.com/anubhavitis/murmur/releases/new?tag=v${VERSION}"
fi

echo ""
echo "Then copy ${CASK_FILE} to homebrew-murmur/Casks/murmur.rb and push."
