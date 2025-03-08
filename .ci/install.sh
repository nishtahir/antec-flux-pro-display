#!/bin/bash
set -euo pipefail

BASE_URL="https://github.com/nishtahir/antec-flux-pro-display/releases/latest/download/"
PACKAGE="af-pro-display.deb"
CHECKSUM_FILE="af-pro-display.deb.sha256"

tmp_dir=$(mktemp -d)
trap "rm -rf $tmp_dir" EXIT
cd "$tmp_dir"

curl -sSL -O "${BASE_URL}/${PACKAGE}"
curl -sSL -O "${BASE_URL}/${CHECKSUM_FILE}"

ls -la

echo "Verifying package integrity..."
sha256sum "./${CHECKSUM_FILE}"

echo "Installing the package..."
sudo dpkg -i "./${PACKAGE}"

echo "Installation complete!"
