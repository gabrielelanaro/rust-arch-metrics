#!/bin/bash

set -e

BINARY_NAME="rust-arch-metrics"
BINARY_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

# Detect platform
OS=$(uname -s)
ARCH=$(uname -m)

if [[ "$OS" != "Darwin" ]] || [[ "$ARCH" != "arm64" ]]; then
    echo "Warning: This binary is built for macOS ARM64 (Apple Silicon)."
    echo "Your platform: $OS $ARCH"
    echo "Installation will proceed but the binary may not work."
    echo "Consider building from source for your platform."
    echo ""
fi

# Determine install location
if [[ -d "$HOME/.cargo/bin" ]]; then
    INSTALL_DIR="$HOME/.cargo/bin"
elif [[ -d "/usr/local/bin" ]] && [[ -w "/usr/local/bin" ]]; then
    INSTALL_DIR="/usr/local/bin"
else
    INSTALL_DIR="$HOME/.local/bin"
    mkdir -p "$INSTALL_DIR"
fi

echo "Installing $BINARY_NAME to $INSTALL_DIR..."

# Copy binary
cp "$BINARY_DIR/$BINARY_NAME" "$INSTALL_DIR/"
chmod +x "$INSTALL_DIR/$BINARY_NAME"

# Check if install dir is in PATH
if [[ ":$PATH:" != *":$INSTALL_DIR:"* ]]; then
    echo ""
    echo "Warning: $INSTALL_DIR is not in your PATH."
    echo "Add the following to your shell configuration:"
    echo "    export PATH=\"$INSTALL_DIR:\$PATH\""
fi

echo ""
echo "Installation complete!"
echo "Run '$BINARY_NAME --help' to get started."
