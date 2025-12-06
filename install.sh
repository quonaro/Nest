#!/bin/sh
# Install script for Nest CLI tool
# Supports Linux and macOS

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Detect OS and architecture
OS="$(uname -s)"
ARCH="$(uname -m)"

case "$OS" in
    Linux*)     PLATFORM="linux" ;;
    Darwin*)    PLATFORM="macos" ;;
    *)          echo "${RED}Error: Unsupported OS: $OS${NC}" >&2; exit 1 ;;
esac

case "$ARCH" in
    x86_64)     ARCHITECTURE="x86_64" ;;
    aarch64|arm64) ARCHITECTURE="aarch64" ;;
    *)          echo "${RED}Error: Unsupported architecture: $ARCH${NC}" >&2; exit 1 ;;
esac

# Determine binary name
if [ "$PLATFORM" = "windows" ]; then
    BINARY_NAME="nest.exe"
else
    BINARY_NAME="nest"
fi

# GitHub repository
# TODO: Update this to your actual GitHub repository (e.g., "username/nest")
REPO="quonaro/nest"
VERSION="latest"

# Installation directory
INSTALL_DIR="${HOME}/.local/bin"
BINARY_PATH="${INSTALL_DIR}/${BINARY_NAME}"

echo "${GREEN}Installing Nest CLI...${NC}"
echo "Platform: ${PLATFORM}-${ARCHITECTURE}"
echo "Install directory: ${INSTALL_DIR}"

# Create install directory if it doesn't exist
mkdir -p "${INSTALL_DIR}"

# Download binary
if [ "$VERSION" = "latest" ]; then
    URL="https://github.com/${REPO}/releases/latest/download/nest-${PLATFORM}-${ARCHITECTURE}.tar.gz"
else
    URL="https://github.com/${REPO}/releases/download/v${VERSION}/nest-${PLATFORM}-${ARCHITECTURE}.tar.gz"
fi

echo "${YELLOW}Downloading from: ${URL}${NC}"

# Download with curl or wget
TEMP_DIR=$(mktemp -d)
TEMP_FILE="${TEMP_DIR}/nest-${PLATFORM}-${ARCHITECTURE}"

if command -v curl > /dev/null 2>&1; then
    curl -L -o "${TEMP_FILE}.tar.gz" "${URL}"
elif command -v wget > /dev/null 2>&1; then
    wget -O "${TEMP_FILE}.tar.gz" "${URL}"
else
    echo "${RED}Error: Neither curl nor wget found. Please install one of them.${NC}" >&2
    exit 1
fi

# Extract archive
cd "${TEMP_DIR}"
if [ "$PLATFORM" = "windows" ]; then
    echo "${RED}Error: Windows installation should use install.ps1${NC}" >&2
    exit 1
else
    tar -xzf "${TEMP_FILE}.tar.gz"
fi

# Move binary to install directory
mv "${BINARY_NAME}" "${BINARY_PATH}"

# Cleanup
rm -rf "${TEMP_DIR}"

# Make binary executable
chmod +x "${BINARY_PATH}"

# Check if ~/.local/bin is in PATH
if ! echo "${PATH}" | grep -q "${HOME}/.local/bin"; then
    echo "${YELLOW}Warning: ${INSTALL_DIR} is not in your PATH.${NC}"
    echo "Add this line to your ~/.bashrc, ~/.zshrc, or ~/.profile:"
    echo "${GREEN}export PATH=\"\${HOME}/.local/bin:\${PATH}\"${NC}"
    echo ""
    echo "Then run: source ~/.bashrc  # or ~/.zshrc / ~/.profile"
fi

echo "${GREEN}✓ Nest CLI installed successfully!${NC}"
echo "Run 'nest --version' to verify installation."

