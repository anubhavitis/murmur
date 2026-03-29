#!/bin/bash
set -euo pipefail

VERSION=$(sed -n '/^\[package\]/,/^\[/p' Cargo.toml | grep '^version' | head -1 | sed 's/.*"\(.*\)".*/\1/')
if [ -z "$VERSION" ]; then
    echo "Error: Could not parse version from Cargo.toml"
    exit 1
fi

BINARY="target/release/murmur"
ARCHIVE="murmur-${VERSION}-aarch64-apple-darwin.zip"
TAG="v${VERSION}"

if git rev-parse "$TAG" >/dev/null 2>&1; then
    echo "Warning: Tag ${TAG} already exists. Bump version in Cargo.toml first."
    read -p "Continue anyway? [y/N] " -n 1 -r
    echo ""
    if [[ ! $REPLY =~ ^[Yy]$ ]]; then
        exit 1
    fi
fi

if ! command -v swift &>/dev/null; then
    echo "Error: Swift not found. Install Xcode Command Line Tools."
    exit 1
fi

echo "Building murmur v${VERSION} (release, with FluidAudio)..."
cargo build --release --features fluid_audio

echo "Stripping binary..."
strip "$BINARY" 2>/dev/null || echo "Warning: strip not available, skipping"

echo "Creating .app bundle..."
STAGING=$(mktemp -d)
APP_DIR="${STAGING}/Murmur.app/Contents"
mkdir -p "${APP_DIR}/MacOS" "${APP_DIR}/Resources"
cp "$BINARY" "${APP_DIR}/MacOS/murmur"
cp dist/AppIcon.icns "${APP_DIR}/Resources/AppIcon.icns"
sed "s/__VERSION__/${VERSION}/g" dist/Info.plist > "${APP_DIR}/Info.plist"

echo "Creating archive..."
cd "$STAGING" && zip -rq "$OLDPWD/$ARCHIVE" Murmur.app && cd "$OLDPWD"
rm -rf "$STAGING"

SHA=$(shasum -a 256 "$ARCHIVE" | awk '{print $1}')

CASK_FILE="dist/murmur.rb"
sed -e "s/__VERSION__/${VERSION}/g" -e "s/__SHA256__/${SHA}/g" dist/murmur.rb.template > "$CASK_FILE"

echo ""
echo "=== Release Ready ==="
echo "Version:  ${VERSION}"
echo "Archive:  ${ARCHIVE}"
echo "SHA256:   ${SHA}"
echo "Size:     $(du -h "$ARCHIVE" | awk '{print $1}')"
echo "Cask:     ${CASK_FILE}"
echo ""

if ! git rev-parse "$TAG" >/dev/null 2>&1; then
    echo "Creating git tag ${TAG}..."
    git tag -a "$TAG" -m "Release ${TAG}"
    echo "Push tag with: git push origin ${TAG}"
else
    echo "Tag ${TAG} already exists, skipping tag creation."
fi

echo ""
if command -v gh &>/dev/null; then
    echo "To create GitHub release:"
    echo "  git push origin ${TAG}"
    echo "  gh release create ${TAG} ${ARCHIVE} --title \"${TAG}\" --notes \"Release ${TAG}\""
else
    echo "1. Push the tag:  git push origin ${TAG}"
    echo "2. Upload ${ARCHIVE} to: https://github.com/anubhavitis/murmur/releases/new?tag=${TAG}"
fi

echo ""
echo "Then copy ${CASK_FILE} to homebrew-murmur/Casks/murmur.rb and push."
