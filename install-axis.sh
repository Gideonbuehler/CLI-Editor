#!/bin/bash
INSTALL_DIR="$HOME/.local/bin"
BIN_URL="https://cloud.arcinger.com/s/JmXgj5MXer5P7PR/download?path=/axis"

mkdir -p "$INSTALL_DIR"
curl -L "$BIN_URL" -o "$INSTALL_DIR/axis"
chmod +x "$INSTALL_DIR/axis"

# Add to PATH if needed
if [[ ":$PATH:" != *":$INSTALL_DIR:"* ]]; then
    echo 'export PATH="$HOME/.local/bin:$PATH"' >> ~/.bashrc
fi

echo "Axis installed! Run 'axis' or restart terminal"
