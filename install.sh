#!/usr/bin/env bash
# RShare Installer for Linux
# Run: curl -fsSL https://raw.githubusercontent.com/ronakgh97/r-share/main/install.sh | bash

set -e

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
CYAN='\033[0;36m'
NC='\033[0m'

echo -e "${CYAN}Installing RShare...${NC}"

# Configuration
REPO="ronakgh97/r-share"
VERSION="v1.0.0-beta"
BINARY_NAME="rs-linux"
INSTALL_DIR="$HOME/.local/bin"
INSTALL_NAME="rs"

# Download URL (from GitHub Release)
DOWNLOAD_URL="https://github.com/$REPO/releases/download/$VERSION/$BINARY_NAME"

echo -e "${YELLOW}✓ Downloading RShare $VERSION...${NC}"

# Create install directory
mkdir -p "$INSTALL_DIR"

# Download binary
BINARY_PATH="$INSTALL_DIR/$INSTALL_NAME"
if command -v curl &> /dev/null; then
    curl -fsSL -o "$BINARY_PATH" "$DOWNLOAD_URL"
elif command -v wget &> /dev/null; then
    wget -q -O "$BINARY_PATH" "$DOWNLOAD_URL"
else
    echo -e "${RED}✓ Neither curl nor wget found${NC}"
    exit 1
fi

chmod +x "$BINARY_PATH"
echo -e "${GREEN}✓ Downloaded successfully${NC}"

# Add to PATH
echo -e "${YELLOW} Checking PATH...${NC}"

SHELL_RC=""
case "$SHELL" in
    */bash)
        SHELL_RC="$HOME/.bashrc"
        ;;
    */zsh)
        SHELL_RC="$HOME/.zshrc"
        ;;
    */fish)
        SHELL_RC="$HOME/.config/fish/config.fish"
        ;;
esac

if [ -n "$SHELL_RC" ] && [ -f "$SHELL_RC" ]; then
    if ! grep -q "$INSTALL_DIR" "$SHELL_RC"; then
        echo "export PATH=\"\$PATH:$INSTALL_DIR\"" >> "$SHELL_RC"
        echo -e "${GREEN}✓ Added to PATH in $SHELL_RC${NC}"
    else
        echo -e "${GREEN}✓ Already in PATH${NC}"
    fi
fi

echo ""
echo -e "${GREEN}✓ Installation complete!${NC}"
echo ""