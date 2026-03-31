#!/bin/sh
set -e

# Eidra installer
# Usage: curl -sf eidra.dev/install | sh

REPO="hanabi-jpn/eidra"
INSTALL_DIR="/usr/local/bin"

# Detect OS and architecture
OS="$(uname -s)"
ARCH="$(uname -m)"

case "$OS" in
    Linux)  OS="linux" ;;
    Darwin) OS="darwin" ;;
    *)      echo "Unsupported OS: $OS"; exit 1 ;;
esac

case "$ARCH" in
    x86_64)  ARCH="x86_64" ;;
    aarch64) ARCH="aarch64" ;;
    arm64)   ARCH="aarch64" ;;
    *)       echo "Unsupported architecture: $ARCH"; exit 1 ;;
esac

TARGET="${OS}-${ARCH}"

echo "Installing Eidra for ${TARGET}..."

# Get latest release
LATEST=$(curl -sf "https://api.github.com/repos/${REPO}/releases/latest" | grep '"tag_name"' | sed -E 's/.*"([^"]+)".*/\1/')

if [ -z "$LATEST" ]; then
    echo "Could not determine latest version. Building from source..."
    echo ""
    echo "  cargo install eidra-core"
    echo ""
    exit 1
fi

URL="https://github.com/${REPO}/releases/download/${LATEST}/eidra-${TARGET}"

echo "Downloading ${URL}..."
curl -sfL "$URL" -o /tmp/eidra

chmod +x /tmp/eidra

if [ -w "$INSTALL_DIR" ]; then
    mv /tmp/eidra "$INSTALL_DIR/eidra"
else
    echo "Need sudo to install to $INSTALL_DIR"
    sudo mv /tmp/eidra "$INSTALL_DIR/eidra"
fi

echo ""
echo "✓ Eidra ${LATEST} installed to ${INSTALL_DIR}/eidra"
echo ""
echo "Get started:"
echo "  eidra init"
echo "  eidra start"
echo "  eidra dashboard"
echo ""
