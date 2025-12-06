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

# Check if ~/.local/bin is in PATH and add to all existing shell configs
echo ""
PATH_EXPORT="export PATH=\"\${HOME}/.local/bin:\${PATH}\""

if ! echo "${PATH}" | grep -q "${HOME}/.local/bin"; then
    echo "${INFO} ${BOLD}Configuring PATH...${RESET}"
    
    # Add to current session
    export PATH="${HOME}/.local/bin:${PATH}"
    echo "   ${CHECK} Added to PATH for current session"
    
    # Add to all existing shell configuration files
    ADDED_TO_CONFIG=0
    CONFIG_FILES_ADDED=()
    RELOAD_CMDS=()
    
    # Check and add to .zshrc
    if [ -f "${HOME}/.zshrc" ]; then
        if ! grep -q "${HOME}/.local/bin" "${HOME}/.zshrc" 2>/dev/null; then
            echo "" >> "${HOME}/.zshrc"
            echo "# Added by Nest CLI installer" >> "${HOME}/.zshrc"
            echo "${PATH_EXPORT}" >> "${HOME}/.zshrc"
            echo "   ${CHECK} Added to ~/.zshrc"
            CONFIG_FILES_ADDED+=("~/.zshrc")
            RELOAD_CMDS+=("source ~/.zshrc")
            ADDED_TO_CONFIG=1
        else
            echo "   ${CHECK} Already in ~/.zshrc"
            ADDED_TO_CONFIG=1
        fi
    fi
    
    # Check and add to .bashrc
    if [ -f "${HOME}/.bashrc" ]; then
        if ! grep -q "${HOME}/.local/bin" "${HOME}/.bashrc" 2>/dev/null; then
            echo "" >> "${HOME}/.bashrc"
            echo "# Added by Nest CLI installer" >> "${HOME}/.bashrc"
            echo "${PATH_EXPORT}" >> "${HOME}/.bashrc"
            echo "   ${CHECK} Added to ~/.bashrc"
            CONFIG_FILES_ADDED+=("~/.bashrc")
            RELOAD_CMDS+=("source ~/.bashrc")
            ADDED_TO_CONFIG=1
        else
            echo "   ${CHECK} Already in ~/.bashrc"
            ADDED_TO_CONFIG=1
        fi
    fi
    
    # Check and add to .bash_profile (macOS sometimes uses this)
    if [ -f "${HOME}/.bash_profile" ]; then
        if ! grep -q "${HOME}/.local/bin" "${HOME}/.bash_profile" 2>/dev/null; then
            echo "" >> "${HOME}/.bash_profile"
            echo "# Added by Nest CLI installer" >> "${HOME}/.bash_profile"
            echo "${PATH_EXPORT}" >> "${HOME}/.bash_profile"
            echo "   ${CHECK} Added to ~/.bash_profile"
            CONFIG_FILES_ADDED+=("~/.bash_profile")
            RELOAD_CMDS+=("source ~/.bash_profile")
            ADDED_TO_CONFIG=1
        else
            echo "   ${CHECK} Already in ~/.bash_profile"
            ADDED_TO_CONFIG=1
        fi
    fi
    
    # Check and add to fish config
    if command -v fish > /dev/null 2>&1; then
        FISH_CONFIG_DIR="${HOME}/.config/fish"
        FISH_CONFIG_FILE="${FISH_CONFIG_DIR}/config.fish"
        if [ -d "${FISH_CONFIG_DIR}" ] || mkdir -p "${FISH_CONFIG_DIR}" 2>/dev/null; then
            if [ ! -f "${FISH_CONFIG_FILE}" ]; then
                touch "${FISH_CONFIG_FILE}"
            fi
            if ! grep -q "${HOME}/.local/bin" "${FISH_CONFIG_FILE}" 2>/dev/null; then
                echo "" >> "${FISH_CONFIG_FILE}"
                echo "# Added by Nest CLI installer" >> "${FISH_CONFIG_FILE}"
                echo "set -gx PATH \"\${HOME}/.local/bin\" \$PATH" >> "${FISH_CONFIG_FILE}"
                echo "   ${CHECK} Added to ~/.config/fish/config.fish"
                CONFIG_FILES_ADDED+=("~/.config/fish/config.fish")
                RELOAD_CMDS+=("source ~/.config/fish/config.fish")
                ADDED_TO_CONFIG=1
            else
                echo "   ${CHECK} Already in ~/.config/fish/config.fish"
                ADDED_TO_CONFIG=1
            fi
        fi
    fi
    
    # Check and add to .profile as fallback (if no other configs found)
    if [ $ADDED_TO_CONFIG -eq 0 ]; then
        PROFILE_FILE="${HOME}/.profile"
        if [ -f "${PROFILE_FILE}" ]; then
            if ! grep -q "${HOME}/.local/bin" "${PROFILE_FILE}" 2>/dev/null; then
                echo "" >> "${PROFILE_FILE}"
                echo "# Added by Nest CLI installer" >> "${PROFILE_FILE}"
                echo "${PATH_EXPORT}" >> "${PROFILE_FILE}"
                echo "   ${CHECK} Added to ~/.profile"
                CONFIG_FILES_ADDED+=("~/.profile")
                RELOAD_CMDS+=("source ~/.profile")
                ADDED_TO_CONFIG=1
            else
                echo "   ${CHECK} Already in ~/.profile"
                ADDED_TO_CONFIG=1
            fi
        else
            # Create .profile if it doesn't exist and no other configs were found
            echo "${PATH_EXPORT}" > "${PROFILE_FILE}"
            chmod 644 "${PROFILE_FILE}"
            echo "   ${CHECK} Created ~/.profile with PATH"
            CONFIG_FILES_ADDED+=("~/.profile")
            RELOAD_CMDS+=("source ~/.profile")
            ADDED_TO_CONFIG=1
        fi
    fi
    
    if [ $ADDED_TO_CONFIG -eq 1 ]; then
        echo ""
        if [ ${#CONFIG_FILES_ADDED[@]} -gt 0 ]; then
            echo "   ${INFO} PATH has been added to:"
            for config_file in "${CONFIG_FILES_ADDED[@]}"; do
                echo "      ${BOLD}${config_file}${RESET}"
            done
        fi
        echo ""
        echo "   ${INFO} Run one of these commands to reload your shell configuration:"
        for reload_cmd in "${RELOAD_CMDS[@]}"; do
            echo "      ${BOLD}${reload_cmd}${RESET}"
        done
        echo "   ${INFO} Or simply restart your terminal."
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

