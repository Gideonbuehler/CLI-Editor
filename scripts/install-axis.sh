#!/bin/bash
# Axis Editor Installer
# Usage: curl -fsSL https://your-nextcloud-url/install-axis.sh | bash

set -e

INSTALL_DIR="${INSTALL_DIR:-$HOME/.local/bin}"
NEXTCLOUD_BASE="https://cloud.arcinger.com/s/fe76EZTrkgWEFHo/download"

# Detect OS and architecture
OS="$(uname -s)"
ARCH="$(uname -m)"

case "$OS" in
    Linux*)
        case "$ARCH" in
            x86_64)
                BINARY_NAME="axis-linux-x86_64"
                ;;
            aarch64|arm64)
                BINARY_NAME="axis-linux-aarch64"
                ;;
            *)
                echo "Unsupported architecture: $ARCH"
                exit 1
                ;;
        esac
        ;;
    Darwin*)
        case "$ARCH" in
            x86_64)
                BINARY_NAME="axis-macos-x86_64"
                ;;
            arm64)
                BINARY_NAME="axis-macos-arm64"
                ;;
            *)
                echo "Unsupported architecture: $ARCH"
                exit 1
                ;;
        esac
        ;;
    MINGW*|MSYS*|CYGWIN*)
        BINARY_NAME="axis.exe"
        INSTALL_DIR="${INSTALL_DIR:-$HOME/bin}"
        ;;
    *)
        echo "Unsupported OS: $OS"
        exit 1
        ;;
esac

echo "Installing Axis Editor..."
echo "  OS: $OS"
echo "  Architecture: $ARCH"
echo "  Binary: $BINARY_NAME"
echo "  Install directory: $INSTALL_DIR"
echo ""

# Create install directory if it doesn't exist
mkdir -p "$INSTALL_DIR"

# Download the binary
DOWNLOAD_URL="${NEXTCLOUD_BASE}?path=/${BINARY_NAME}"
echo "Downloading from: $DOWNLOAD_URL"

if command -v curl &> /dev/null; then
    curl -fsSL "$DOWNLOAD_URL" -o "$INSTALL_DIR/axis"
elif command -v wget &> /dev/null; then
    wget -q "$DOWNLOAD_URL" -O "$INSTALL_DIR/axis"
else
    echo "Error: Neither curl nor wget found. Please install one of them."
    exit 1
fi

# Make it executable
chmod +x "$INSTALL_DIR/axis"

echo ""
echo "Axis Editor installed successfully!"
echo ""

# Check if install dir is in PATH
if [[ ":$PATH:" != *":$INSTALL_DIR:"* ]]; then
    echo "Note: $INSTALL_DIR is not in your PATH."
    echo "Add it by running:"
    echo ""
    echo "  echo 'export PATH=\"\$PATH:$INSTALL_DIR\"' >> ~/.bashrc"
    echo "  source ~/.bashrc"
    echo ""
fi

echo "Run 'axis' to start the editor, or 'axis <filename>' to open a file."
