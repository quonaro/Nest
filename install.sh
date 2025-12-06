#!/bin/sh
# Install script for Nest CLI tool
# Supports Linux and macOS

set -e

# Detect if colors are supported
if [ -t 1 ] && command -v tput > /dev/null 2>&1; then
    RED=$(tput setaf 1)
    GREEN=$(tput setaf 2)
    YELLOW=$(tput setaf 3)
    BLUE=$(tput setaf 4)
    BOLD=$(tput bold)
    RESET=$(tput sgr0)
    HAS_COLORS=1
else
    RED=''
    GREEN=''
    YELLOW=''
    BLUE=''
    BOLD=''
    RESET=''
    HAS_COLORS=0
fi

# Detect Unicode support
if [ "$HAS_COLORS" = "1" ] && [ -n "$LANG" ] && echo "$LANG" | grep -q "UTF-8\|utf8"; then
    CHECK="${GREEN}✓${RESET}"
    CROSS="${RED}✗${RESET}"
    ARROW="${BLUE}→${RESET}"
    INFO="${BLUE}ℹ${RESET}"
    WARN="${YELLOW}⚠${RESET}"
    NEST_ICON="🪺"
else
    CHECK="${GREEN}[OK]${RESET}"
    CROSS="${RED}[FAIL]${RESET}"
    ARROW="${BLUE}=>${RESET}"
    INFO="${BLUE}[i]${RESET}"
    WARN="${YELLOW}[!]${RESET}"
    NEST_ICON="Nest"
fi

# Detect OS and architecture
OS="$(uname -s)"
ARCH="$(uname -m)"

case "$OS" in
    Linux*)     PLATFORM="linux" ;;
    Darwin*)    PLATFORM="macos" ;;
    *)          echo "${CROSS} ${BOLD}${RED}Error: Unsupported OS: $OS${RESET}" >&2; exit 1 ;;
esac

case "$ARCH" in
    x86_64)     ARCHITECTURE="x86_64" ;;
    aarch64|arm64) ARCHITECTURE="aarch64" ;;
    *)          echo "${CROSS} ${BOLD}${RED}Error: Unsupported architecture: $ARCH${RESET}" >&2; exit 1 ;;
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

# Print header
echo ""
if [ "$HAS_COLORS" = "1" ]; then
    echo "${BOLD}${BLUE}╔════════════════════════════════════════╗${RESET}"
    echo "${BOLD}${BLUE}║${RESET}  ${BOLD}${NEST_ICON} Nest CLI Installer${RESET}              ${BOLD}${BLUE}║${RESET}"
    echo "${BOLD}${BLUE}╚════════════════════════════════════════╝${RESET}"
else
    echo "${BOLD}${NEST_ICON} Nest CLI Installer${RESET}"
    echo "========================================"
fi
echo ""

# Print system information
echo "${INFO} ${BOLD}Detected system:${RESET}"
echo "   ${ARROW} Platform: ${BOLD}${PLATFORM}-${ARCHITECTURE}${RESET}"
echo "   ${ARROW} Install directory: ${BOLD}${INSTALL_DIR}${RESET}"
echo ""

# Create install directory if it doesn't exist
echo "${INFO} ${BOLD}Preparing installation...${RESET}"
mkdir -p "${INSTALL_DIR}"
echo "   ${CHECK} Created install directory"

# Download binary
if [ "$VERSION" = "latest" ]; then
    URL="https://github.com/${REPO}/releases/latest/download/nest-${PLATFORM}-${ARCHITECTURE}.tar.gz"
else
    URL="https://github.com/${REPO}/releases/download/v${VERSION}/nest-${PLATFORM}-${ARCHITECTURE}.tar.gz"
fi

# Download with curl or wget
TEMP_DIR=$(mktemp -d)
TEMP_FILE="${TEMP_DIR}/nest-${PLATFORM}-${ARCHITECTURE}.tar.gz"

echo ""
echo "${INFO} ${BOLD}Downloading Nest CLI...${RESET}"
echo "   ${ARROW} ${URL}"

# Download and check for errors
if command -v curl > /dev/null 2>&1; then
    HTTP_CODE=$(curl -L -w "%{http_code}" -o "${TEMP_FILE}" "${URL}" -s -S --show-error)
    if [ "$HTTP_CODE" != "200" ]; then
        echo ""
        echo "${CROSS} ${BOLD}${RED}Error: Failed to download binary (HTTP $HTTP_CODE)${RESET}" >&2
        echo "   ${WARN} The release may not exist yet. Please check:" >&2
        echo "      https://github.com/${REPO}/releases" >&2
        rm -rf "${TEMP_DIR}"
        exit 1
    fi
    echo "   ${CHECK} Download completed"
elif command -v wget > /dev/null 2>&1; then
    if ! wget -O "${TEMP_FILE}" "${URL}" 2>&1 | grep -q "200 OK"; then
        echo ""
        echo "${CROSS} ${BOLD}${RED}Error: Failed to download binary${RESET}" >&2
        echo "   ${WARN} The release may not exist yet. Please check:" >&2
        echo "      https://github.com/${REPO}/releases" >&2
        rm -rf "${TEMP_DIR}"
        exit 1
    fi
    echo "   ${CHECK} Download completed"
else
    echo ""
    echo "${CROSS} ${BOLD}${RED}Error: Neither curl nor wget found${RESET}" >&2
    echo "   ${WARN} Please install curl or wget to continue" >&2
    exit 1
fi

# Verify downloaded file is a valid archive
echo "${INFO} ${BOLD}Verifying download...${RESET}"
if ! file "${TEMP_FILE}" | grep -q "gzip\|archive"; then
    echo ""
    echo "${CROSS} ${BOLD}${RED}Error: Downloaded file is not a valid archive${RESET}" >&2
    echo "   ${WARN} The release may not exist yet. Please check:" >&2
    echo "      https://github.com/${REPO}/releases" >&2
    rm -rf "${TEMP_DIR}"
    exit 1
fi
echo "   ${CHECK} Archive verified"

# Extract archive
echo "${INFO} ${BOLD}Extracting archive...${RESET}"
cd "${TEMP_DIR}"
if ! tar -xzf "${TEMP_FILE}" 2>/dev/null; then
    echo ""
    echo "${CROSS} ${BOLD}${RED}Error: Failed to extract archive${RESET}" >&2
    echo "   ${WARN} The downloaded file may be corrupted" >&2
    rm -rf "${TEMP_DIR}"
    exit 1
fi
echo "   ${CHECK} Archive extracted"

# Check if binary exists
if [ ! -f "${BINARY_NAME}" ]; then
    echo ""
    echo "${CROSS} ${BOLD}${RED}Error: Binary '${BINARY_NAME}' not found in archive${RESET}" >&2
    echo "   ${WARN} Archive contents:" >&2
    ls -la "${TEMP_DIR}" >&2
    rm -rf "${TEMP_DIR}"
    exit 1
fi

# Install binary
echo "${INFO} ${BOLD}Installing binary...${RESET}"
mv "${BINARY_NAME}" "${BINARY_PATH}"
chmod +x "${BINARY_PATH}"
echo "   ${CHECK} Binary installed to ${BINARY_PATH}"

# Cleanup
rm -rf "${TEMP_DIR}"

# Check if ~/.local/bin is in PATH
echo ""
if ! echo "${PATH}" | grep -q "${HOME}/.local/bin"; then
    echo "${WARN} ${BOLD}${YELLOW}Note: ${INSTALL_DIR} is not in your PATH${RESET}"
    echo ""
    echo "   To use 'nest' command, add this to your shell config:"
    echo ""
    echo "   ${BOLD}${GREEN}export PATH=\"\${HOME}/.local/bin:\${PATH}\"${RESET}"
    echo ""
    # Detect shell and suggest the right file
    SHELL_NAME=$(basename "${SHELL:-sh}")
    case "$SHELL_NAME" in
        zsh)  CONFIG_FILE="~/.zshrc" ;;
        bash) CONFIG_FILE="~/.bashrc" ;;
        *)    CONFIG_FILE="~/.profile" ;;
    esac
    echo "   Add it to ${BOLD}${CONFIG_FILE}${RESET} and run:"
    echo "   ${BOLD}source ${CONFIG_FILE}${RESET}"
    echo ""
fi

# Success message
echo ""
if [ "$HAS_COLORS" = "1" ]; then
    echo "${BOLD}${GREEN}╔════════════════════════════════════════╗${RESET}"
    echo "${BOLD}${GREEN}║${RESET}  ${CHECK} ${BOLD}Nest CLI installed successfully!${RESET}  ${BOLD}${GREEN}║${RESET}"
    echo "${BOLD}${GREEN}╚════════════════════════════════════════╝${RESET}"
else
    echo "${CHECK} ${BOLD}Nest CLI installed successfully!${RESET}"
    echo "========================================"
fi
echo ""
echo "   Run ${BOLD}nest --version${RESET} to verify installation."
echo ""

