#!/usr/bin/env bash

# This script is used to zip up the built binary for release,
# and is used in the GitHub workflow to create a release artifact

# A step that fails stops the script: a release must not be built from half an archive
set -euo pipefail

BIN_NAME="luneblox"
BIN_EXT=""
CWD="$PWD"

# We should have gotten TARGET_TRIPLE as the first arg to this script
TARGET_TRIPLE="${1:-}"
if [ -z "$TARGET_TRIPLE" ]; then
    echo "Usage: $0 <TARGET_TRIPLE>"
    exit 1
fi
TARGET_DIR="target/$TARGET_TRIPLE/release"
if [ ! -d "$TARGET_DIR" ]; then
    echo "Target directory '$TARGET_DIR' does not exist"
    exit 1
fi

# Use exe extension on windows
OS="$(uname -s | tr '[:upper:]' '[:lower:]')"
case "$OS" in
    darwin) OS="macos" ;;
    linux) OS="linux" ;;
    cygwin*|mingw*|msys*) OS="windows" ;;
    *)
        echo "Unsupported OS: $OS" >&2
        exit 1 ;;
esac
if [ "$OS" = "windows" ]; then
    BIN_EXT=".exe"
fi

# Clean up any previous artifacts and dirs
rm -rf staging
rm -rf release.zip

# Create new staging dir to work in and copy the binary into that.
# The license travels with the binary, so the terms reach whoever unpacks the zip.
mkdir -p staging
cp "$TARGET_DIR/$BIN_NAME$BIN_EXT" staging/
cp "LICENSE.txt" staging/
cd staging

# Zip the staging dir up
if [ "$OS" = "windows" ]; then
	7z a ../release.zip *
else
	chmod +x "$BIN_NAME"
	zip ../release.zip *
fi

# Go back to cwd and clean up staging dir
cd "$CWD"
rm -rf staging
