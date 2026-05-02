#!/bin/bash
# onesource Network Installer for macOS / Linux
# Run with: curl -sSL https://raw.githubusercontent.com/TW-RF54732/onesource/main/install.sh | bash

REPO="TW-RF54732/onesource"
INSTALL_DIR="/usr/local/bin"

# 色彩設定
CYAN='\033[0;36m'
GREEN='\033[0;32m'
GRAY='\033[1;30m'
NC='\033[0m' # No Color

echo -e "${CYAN}"
echo "=========================================================="
echo "  ____  _   _ _____   ____   ___  _   _ ____   ____ _____ "
echo " / __ \| \ | | ____| / ___| / _ \| | | |  _ \ / ___| ____|"
echo "| |  | |  \| |  _|   \___ \| | | | | | | |_) | |   |  _|  "
echo "| |__| | |\  | |___   ___) | |_| | |_| |  _ <| |___| |___ "
echo " \____/|_| \_|_____| |____/ \___/ \___/|_| \_\\\\____|_____|"
echo "                          "
echo " >> onesource Network Installer | Unix Edition <<"
echo "=========================================================="
echo -e "${NC}"

# 1. 判斷 OS
OS="$(uname -s)"
if [ "$OS" = "Linux" ]; then
    ASSET_NAME="onesource-linux"
elif [ "$OS" = "Darwin" ]; then
    ASSET_NAME="onesource-macos"
else
    echo "Unsupported OS: $OS"
    exit 1
fi

# 2. 取得下載連結
echo -e "[1/3] Fetching latest release info from GitHub..."
DOWNLOAD_URL=$(curl -s "https://api.github.com/repos/$REPO/releases/latest" | grep "browser_download_url.*$ASSET_NAME" | cut -d '"' -f 4)

if [ -z "$DOWNLOAD_URL" ]; then
    echo "Error: Could not find download URL for $ASSET_NAME"
    exit 1
fi

# 3. 下載檔案
echo -e "[2/3] Downloading $ASSET_NAME..."
curl -# -L -o onesource "$DOWNLOAD_URL"
chmod +x onesource

# 4. 安裝至全域路徑
echo -e "[3/3] Installing to $INSTALL_DIR..."
if [ -w "$INSTALL_DIR" ]; then
    mv onesource "$INSTALL_DIR/onesource"
else
    echo -e "${GRAY}      Requesting sudo privileges to move file to $INSTALL_DIR...${NC}"
    sudo mv onesource "$INSTALL_DIR/onesource"
fi

echo -e "\n${CYAN}==========================================================${NC}"
echo -e "${GREEN}  INSTALLATION COMPLETE!${NC}"
echo -e "  Location: $INSTALL_DIR/onesource"
echo -e "  You can now use the '${GREEN}onesource${NC}' command anywhere."
echo -e "${CYAN}==========================================================${NC}"