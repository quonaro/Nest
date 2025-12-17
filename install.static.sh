#!/bin/sh
# Install script for Nest CLI (Linux x86_64, statically linked musl binary)

set -e

# Colors (best-effort)
if [ -t 1 ] && command -v tput >/dev/null 2>&1; then
    RED=$(tput setaf 1)
    GREEN=$(tput setaf 2)
    BLUE=$(tput setaf 4)
    YELLOW=$(tput setaf 3)
    BOLD=$(tput bold)
    RESET=$(tput sgr0)
else
    RED=''; GREEN=''; BLUE=''; YELLOW=''; BOLD=''; RESET=''
fi

CHECK="${GREEN}✓${RESET}"
CROSS="${RED}✗${RESET}"
ARROW="${BLUE}→${RESET}"
INFO="${BLUE}ℹ${RESET}"

OS="$(uname -s)"
ARCH="$(uname -m)"

if [ "$OS" != "Linux" ] || [ "$ARCH" != "x86_64" ]; then
    echo "${CROSS} ${BOLD}${RED}This static installer supports only Linux x86_64${RESET}" >&2
    exit 1
fi

REPO="quonaro/nest"
PLATFORM_NAME="linux-musl"
ARCH_NAME="x86_64"

INSTALL_DIR="${HOME}/.local/bin"
BINARY_NAME="nest"
BINARY_PATH="${INSTALL_DIR}/${BINARY_NAME}"

echo ""
echo "${BOLD}${BLUE}Nest CLI Static Installer (musl, Linux x86_64)${RESET}"
echo ""
echo "${INFO} ${BOLD}Target:${RESET} ${PLATFORM_NAME}-${ARCH_NAME}"
echo "${INFO} ${BOLD}Install dir:${RESET} ${INSTALL_DIR}"
echo ""

mkdir -p "${INSTALL_DIR}"

# Version to install:
# - default: "latest" (GitHub's latest release)
# - override via NEST_VERSION env var
# - or via first CLI argument: ./install.static.sh 0.1.0
VERSION="${NEST_VERSION:-latest}"
if [ -n "$1" ]; then
    VERSION="$1"
fi
if [ "$VERSION" = "latest" ]; then
    ARCHIVE_NAME="nest-${PLATFORM_NAME}-${ARCH_NAME}.tar.gz"
    URL="https://github.com/${REPO}/releases/latest/download/${ARCHIVE_NAME}"
else
    ARCHIVE_NAME="nest-${PLATFORM_NAME}-${ARCH_NAME}.tar.gz"
    URL="https://github.com/${REPO}/releases/download/v${VERSION}/${ARCHIVE_NAME}"
fi

TMP_DIR=$(mktemp -d)
ARCHIVE_PATH="${TMP_DIR}/${ARCHIVE_NAME}"

echo "${INFO} ${BOLD}Downloading static binary...${RESET}"
echo "   ${ARROW} ${URL}"

if command -v curl >/dev/null 2>&1; then
    curl -L -o "${ARCHIVE_PATH}" "${URL}"
elif command -v wget >/dev/null 2>&1; then
    wget -O "${ARCHIVE_PATH}" "${URL}"
else
    echo "${CROSS} ${BOLD}${RED}curl or wget is required to download the binary${RESET}" >&2
    rm -rf "${TMP_DIR}"
    exit 1
fi

if [ ! -s "${ARCHIVE_PATH}" ]; then
    echo "${CROSS} ${BOLD}${RED}Downloaded file is empty, release asset may be missing${RESET}" >&2
    rm -rf "${TMP_DIR}"
    exit 1
fi

echo "${INFO} ${BOLD}Extracting archive...${RESET}"
cd "${TMP_DIR}"
tar -xzf "${ARCHIVE_PATH}"

if [ ! -f "${BINARY_NAME}" ]; then
    echo "${CROSS} ${BOLD}${RED}Binary '${BINARY_NAME}' not found in archive${RESET}" >&2
    ls -la
    rm -rf "${TMP_DIR}"
    exit 1
fi

echo "${INFO} ${BOLD}Installing binary...${RESET}"
mv "${BINARY_NAME}" "${BINARY_PATH}"
chmod +x "${BINARY_PATH}"

rm -rf "${TMP_DIR}"

echo ""
echo "${CHECK} ${BOLD}${GREEN}Static Nest (musl) installed to${RESET} ${BINARY_PATH}"
echo ""
echo "${INFO} You may need to add ${BOLD}\$HOME/.local/bin${RESET} to your PATH."
echo "${INFO} Run: ${BOLD}nest --version${RESET}"

