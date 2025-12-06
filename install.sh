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

# Check if ~/.local/bin is in PATH and add to shell config
echo ""
PATH_EXPORT="export PATH=\"\${HOME}/.local/bin:\${PATH}\""

if ! echo "${PATH}" | grep -q "${HOME}/.local/bin"; then
    echo "${INFO} ${BOLD}Configuring PATH...${RESET}"
    
    # Add to current session
    export PATH="${HOME}/.local/bin:${PATH}"
    echo "   ${CHECK} Added to PATH for current session"
    
    # Detect current shell
    # Try multiple methods to detect the shell
    CURRENT_SHELL=""
    
    # Method 1: Check $SHELL environment variable
    if [ -n "${SHELL}" ]; then
        CURRENT_SHELL=$(basename "${SHELL}")
    fi
    
    # Method 2: Check parent process (more reliable)
    PARENT_CMD=$(ps -p $PPID -o comm= 2>/dev/null | xargs basename 2>/dev/null || echo "")
    if [ -n "${PARENT_CMD}" ]; then
        case "${PARENT_CMD}" in
            zsh|bash|fish|sh|dash|ksh)
                CURRENT_SHELL="${PARENT_CMD}"
                ;;
        esac
    fi
    
    # Method 3: Check $0 (script runner)
    SCRIPT_RUNNER=$(basename "$0" 2>/dev/null || echo "sh")
    if [ "${SCRIPT_RUNNER}" = "bash" ] || [ "${SCRIPT_RUNNER}" = "zsh" ] || [ "${SCRIPT_RUNNER}" = "fish" ]; then
        CURRENT_SHELL="${SCRIPT_RUNNER}"
    fi
    
    # Default fallback
    if [ -z "${CURRENT_SHELL}" ] || [ "${CURRENT_SHELL}" = "sh" ]; then
        # Try to detect from common shells
        if command -v zsh > /dev/null 2>&1 && [ -f "${HOME}/.zshrc" ]; then
            CURRENT_SHELL="zsh"
        elif command -v bash > /dev/null 2>&1 && [ -f "${HOME}/.bashrc" ]; then
            CURRENT_SHELL="bash"
        elif command -v fish > /dev/null 2>&1; then
            CURRENT_SHELL="fish"
        else
            CURRENT_SHELL="sh"
        fi
    fi
    
    echo "   ${INFO} Detected shell: ${BOLD}${CURRENT_SHELL}${RESET}"
    
    # Add to appropriate config file based on detected shell
    ADDED_TO_CONFIG=0
    CONFIG_FILE=""
    RELOAD_CMD=""
    
    case "${CURRENT_SHELL}" in
        zsh)
            CONFIG_FILE="${HOME}/.zshrc"
            RELOAD_CMD="source ~/.zshrc"
            if [ ! -f "${CONFIG_FILE}" ]; then
                touch "${CONFIG_FILE}"
            fi
            if ! grep -q "${HOME}/.local/bin" "${CONFIG_FILE}" 2>/dev/null; then
                echo "" >> "${CONFIG_FILE}"
                echo "# Added by Nest CLI installer" >> "${CONFIG_FILE}"
                echo "${PATH_EXPORT}" >> "${CONFIG_FILE}"
                echo "   ${CHECK} Added to ~/.zshrc"
                ADDED_TO_CONFIG=1
            else
                echo "   ${CHECK} Already in ~/.zshrc"
                ADDED_TO_CONFIG=1
            fi
            ;;
        bash)
            CONFIG_FILE="${HOME}/.bashrc"
            RELOAD_CMD="source ~/.bashrc"
            if [ ! -f "${CONFIG_FILE}" ]; then
                touch "${CONFIG_FILE}"
            fi
            if ! grep -q "${HOME}/.local/bin" "${CONFIG_FILE}" 2>/dev/null; then
                echo "" >> "${CONFIG_FILE}"
                echo "# Added by Nest CLI installer" >> "${CONFIG_FILE}"
                echo "${PATH_EXPORT}" >> "${CONFIG_FILE}"
                echo "   ${CHECK} Added to ~/.bashrc"
                ADDED_TO_CONFIG=1
            else
                echo "   ${CHECK} Already in ~/.bashrc"
                ADDED_TO_CONFIG=1
            fi
            ;;
        fish)
            CONFIG_FILE="${HOME}/.config/fish/config.fish"
            RELOAD_CMD="source ~/.config/fish/config.fish"
            FISH_CONFIG_DIR="${HOME}/.config/fish"
            if [ ! -d "${FISH_CONFIG_DIR}" ]; then
                mkdir -p "${FISH_CONFIG_DIR}" 2>/dev/null || true
            fi
            if [ ! -f "${CONFIG_FILE}" ]; then
                touch "${CONFIG_FILE}"
            fi
            if ! grep -q "${HOME}/.local/bin" "${CONFIG_FILE}" 2>/dev/null; then
                echo "" >> "${CONFIG_FILE}"
                echo "# Added by Nest CLI installer" >> "${CONFIG_FILE}"
                echo "set -gx PATH \"\${HOME}/.local/bin\" \$PATH" >> "${CONFIG_FILE}"
                echo "   ${CHECK} Added to ~/.config/fish/config.fish"
                ADDED_TO_CONFIG=1
            else
                echo "   ${CHECK} Already in ~/.config/fish/config.fish"
                ADDED_TO_CONFIG=1
            fi
            ;;
        *)
            # Fallback to .profile for other shells
            CONFIG_FILE="${HOME}/.profile"
            RELOAD_CMD="source ~/.profile"
            if [ ! -f "${CONFIG_FILE}" ]; then
                echo "${PATH_EXPORT}" > "${CONFIG_FILE}"
                chmod 644 "${CONFIG_FILE}"
                echo "   ${CHECK} Created ~/.profile with PATH"
                ADDED_TO_CONFIG=1
            elif ! grep -q "${HOME}/.local/bin" "${CONFIG_FILE}" 2>/dev/null; then
                echo "" >> "${CONFIG_FILE}"
                echo "# Added by Nest CLI installer" >> "${CONFIG_FILE}"
                echo "${PATH_EXPORT}" >> "${CONFIG_FILE}"
                echo "   ${CHECK} Added to ~/.profile"
                ADDED_TO_CONFIG=1
            else
                echo "   ${CHECK} Already in ~/.profile"
                ADDED_TO_CONFIG=1
            fi
            ;;
    esac
    
    if [ $ADDED_TO_CONFIG -eq 1 ]; then
        echo ""
        echo "   ${INFO} PATH has been added to ${BOLD}${CONFIG_FILE}${RESET}"
        echo "   ${INFO} Run ${BOLD}${RELOAD_CMD}${RESET} or restart your terminal to use 'nest' command."
    else
        echo ""
        echo "   ${WARN} Could not automatically add to shell config."
        echo "   ${WARN} Please add this line manually to your shell configuration:"
        echo "   ${BOLD}${GREEN}${PATH_EXPORT}${RESET}"
    fi
    echo ""
else
    echo "${CHECK} ${BOLD}${GREEN}Already in PATH${RESET}"
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

