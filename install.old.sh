#!/bin/bash
# Install script for old systems with static linking (musl)
# Creates a statically linked binary that works on any Linux system
# Can be used remotely: curl -fsSL https://raw.githubusercontent.com/quonaro/nest/main/install.old.sh | bash

set -e

# Colors for output
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

# Unicode symbols
if [ "$HAS_COLORS" = "1" ] && [ -n "$LANG" ] && echo "$LANG" | grep -q "UTF-8\|utf8"; then
    CHECK="${GREEN}✓${RESET}"
    ARROW="${BLUE}→${RESET}"
    INFO="${BLUE}ℹ${RESET}"
    WARN="${YELLOW}⚠${RESET}"
else
    CHECK="${GREEN}[OK]${RESET}"
    ARROW="${BLUE}=>${RESET}"
    INFO="${BLUE}[i]${RESET}"
    WARN="${YELLOW}[!]${RESET}"
fi

# Configuration
REPO="${REPO:-quonaro/nest}"
INSTALL_DIR="${HOME}/.local/bin"
BINARY_NAME="nest"
BINARY_PATH="${INSTALL_DIR}/${BINARY_NAME}"

# Detect if running remotely (no Cargo.toml in current directory)
# or locally (Cargo.toml exists)
if [ -f "Cargo.toml" ]; then
    # Running locally - use current directory
    SCRIPT_DIR="$(pwd)"
    REMOTE_MODE=false
    CLEANUP_TEMP=false
else
    # Running remotely - clone repository to temp directory
    REMOTE_MODE=true
    CLEANUP_TEMP=true
    SCRIPT_DIR=$(mktemp -d)
    trap "rm -rf '$SCRIPT_DIR'" EXIT INT TERM
fi

OUTPUT_DIR="${SCRIPT_DIR}/target/x86_64-unknown-linux-musl/release"

# Default options
SKIP_INSTALL=false
CUSTOM_VERSION=""
CUSTOM_INSTALL_DIR=""

# Function to show usage
show_usage() {
    echo "Usage: $0 [OPTIONS]"
    echo ""
    echo "Options:"
    echo "  --version VERSION        Use specific version instead of reading from Cargo.toml"
    echo "  --install-dir PATH       Install binary to custom directory (default: ~/.local/bin)"
    echo "  --skip-install           Don't install binary after build"
    echo "  --help                   Show this help message"
    echo ""
    echo "Examples:"
    echo "  $0                                    # Build with default settings"
    echo "  $0 --skip-install                     # Build without installing"
    echo ""
    echo "Prerequisites:"
    echo "  - Rust toolchain installed"
    echo "  - musl target: rustup target add x86_64-unknown-linux-musl"
    echo "  - musl-gcc or musl-tools (depending on your distro)"
}

# Parse command line arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        --version)
            if [ -z "$2" ]; then
                echo "${RED}Error: --version requires a version number${RESET}" >&2
                exit 1
            fi
            CUSTOM_VERSION="$2"
            shift 2
            ;;
        --install-dir)
            if [ -z "$2" ]; then
                echo "${RED}Error: --install-dir requires a path${RESET}" >&2
                exit 1
            fi
            CUSTOM_INSTALL_DIR="$2"
            shift 2
            ;;
        --skip-install)
            SKIP_INSTALL=true
            shift
            ;;
        --help|-h)
            show_usage
            exit 0
            ;;
        *)
            echo "${RED}Error: Unknown option: $1${RESET}" >&2
            echo ""
            show_usage
            exit 1
            ;;
    esac
done

# Apply custom install directory if provided
if [ -n "$CUSTOM_INSTALL_DIR" ]; then
    INSTALL_DIR="$CUSTOM_INSTALL_DIR"
    BINARY_PATH="${INSTALL_DIR}/${BINARY_NAME}"
fi

# If running remotely, clone repository
if [ "$REMOTE_MODE" = true ]; then
    echo "${INFO} ${BOLD}Cloning repository...${RESET}"
    if ! command -v git > /dev/null 2>&1; then
        echo "${RED}Error: git is not installed${RESET}" >&2
        echo "Please install git to use this script remotely"
        exit 1
    fi
    
    echo "   ${ARROW} Cloning https://github.com/${REPO}.git"
    if git clone --depth 1 "https://github.com/${REPO}.git" "$SCRIPT_DIR" > /dev/null 2>&1; then
        echo "   ${CHECK} Repository cloned"
    else
        echo "   ${RED}✗ Failed to clone repository${RESET}" >&2
        exit 1
    fi
fi

# Change to project directory
cd "$SCRIPT_DIR"

# Function to read version from Cargo.toml
read_version() {
    if [ ! -f "Cargo.toml" ]; then
        echo "${RED}Error: Cargo.toml not found${RESET}" >&2
        exit 1
    fi
    grep -E '^version\s*=' "Cargo.toml" | sed -E 's/^version\s*=\s*"([^"]+)".*/\1/' | tr -d '[:space:]'
}

# Check prerequisites
check_prerequisites() {
    echo "${INFO} ${BOLD}Checking prerequisites...${RESET}"
    
    # Check Rust
    if ! command -v rustc > /dev/null 2>&1; then
        echo "   ${RED}✗ Rust is not installed${RESET}" >&2
        echo "   ${ARROW} Installing Rust..."
        if curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y > /dev/null 2>&1; then
            # Source cargo env
            if [ -f "${HOME}/.cargo/env" ]; then
                source "${HOME}/.cargo/env"
                echo "   ${CHECK} Rust installed"
            else
                echo "   ${RED}✗ Failed to install Rust${RESET}" >&2
                exit 1
            fi
        else
            echo "   ${RED}✗ Failed to install Rust${RESET}" >&2
            echo "   ${ARROW} Please install manually: curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh"
            exit 1
        fi
    else
        echo "   ${CHECK} Rust found: $(rustc --version)"
    fi
    
    # Source cargo env if available (for cases where Rust was just installed)
    if [ -f "${HOME}/.cargo/env" ]; then
        source "${HOME}/.cargo/env" 2>/dev/null || true
    fi
    
    # Check musl target
    if ! rustup target list --installed 2>/dev/null | grep -q "x86_64-unknown-linux-musl"; then
        echo "   ${WARN} musl target not installed${RESET}"
        echo "   ${ARROW} Installing musl target..."
        if rustup target add x86_64-unknown-linux-musl > /dev/null 2>&1; then
            echo "   ${CHECK} musl target installed"
        else
            echo "   ${RED}✗ Failed to install musl target${RESET}" >&2
            exit 1
        fi
    else
        echo "   ${CHECK} musl target installed"
    fi
    
    # Check for musl-gcc or musl-tools
    MUSL_AVAILABLE=false
    if command -v musl-gcc > /dev/null 2>&1 || command -v musl-clang > /dev/null 2>&1; then
        MUSL_AVAILABLE=true
        echo "   ${CHECK} musl compiler found"
    elif [ -f "/usr/include/x86_64-linux-musl" ] || [ -d "/usr/x86_64-linux-musl" ] || [ -d "/usr/lib/x86_64-linux-musl" ]; then
        MUSL_AVAILABLE=true
        echo "   ${CHECK} musl headers found"
    else
        echo "   ${WARN} musl development tools may not be installed${RESET}"
        echo "   ${ARROW} On Debian/Ubuntu: sudo apt-get install musl-tools"
        echo "   ${ARROW} On Fedora: sudo dnf install musl-gcc"
        echo "   ${ARROW} On Arch: sudo pacman -S musl"
        echo ""
        echo "   ${INFO} Continuing anyway, but build may fail if musl is not properly configured..."
        MUSL_AVAILABLE=false
    fi
    echo ""
}

# Print header
echo ""
if [ "$HAS_COLORS" = "1" ]; then
    echo "${BOLD}${BLUE}╔════════════════════════════════════════╗${RESET}"
    echo "${BOLD}${BLUE}║${RESET}  ${BOLD}Nest Static Build (musl)${RESET}            ${BOLD}${BLUE}║${RESET}"
    echo "${BOLD}${BLUE}╚════════════════════════════════════════╝${RESET}"
else
    echo "${BOLD}Nest Static Build (musl)${RESET}"
    echo "========================================"
fi
echo ""

# Check prerequisites
check_prerequisites

# Read current version
echo "${INFO} ${BOLD}Reading version...${RESET}"
if [ -n "$CUSTOM_VERSION" ]; then
    CURRENT_VERSION="$CUSTOM_VERSION"
    echo "   ${ARROW} Using custom version: ${BOLD}${CURRENT_VERSION}${RESET}"
else
    CURRENT_VERSION=$(read_version)
    echo "   ${ARROW} Current version: ${BOLD}${CURRENT_VERSION}${RESET}"
fi

# Build project
echo ""
echo "${INFO} ${BOLD}Building project with musl (static linking)...${RESET}"
echo "   ${ARROW} Target: x86_64-unknown-linux-musl"

# Set environment variables for musl build
export RUSTFLAGS="-C target-feature=+crt-static"

if cargo build --release --target x86_64-unknown-linux-musl; then
    echo "   ${CHECK} Build successful"
else
    echo "   ${RED}✗ Build failed${RESET}" >&2
    echo ""
    echo "   ${INFO} Troubleshooting:"
    echo "   ${ARROW} Make sure musl-tools is installed"
    echo "   ${ARROW} Try: sudo apt-get install musl-tools (Debian/Ubuntu)"
    echo "   ${ARROW} Or: sudo dnf install musl-gcc (Fedora)"
    exit 1
fi

# Verify binary
if [ ! -f "$OUTPUT_DIR/nest" ]; then
    echo "   ${RED}✗ Binary not found at ${OUTPUT_DIR}/nest${RESET}" >&2
    exit 1
fi

echo ""
echo "${INFO} ${BOLD}Binary information:${RESET}"
if command -v file > /dev/null 2>&1; then
    echo "   ${ARROW} Type: $(file "$OUTPUT_DIR/nest" | cut -d: -f2-)"
fi
if command -v ldd > /dev/null 2>&1; then
    if ldd "$OUTPUT_DIR/nest" > /dev/null 2>&1; then
        echo "   ${ARROW} Dependencies:"
        ldd "$OUTPUT_DIR/nest" 2>&1 | sed 's/^/      /'
    else
        echo "   ${CHECK} Statically linked (no dynamic dependencies)"
    fi
fi

# Install binary (if not skipped)
if [ "$SKIP_INSTALL" = false ]; then
    echo ""
    echo "${INFO} ${BOLD}Installing binary...${RESET}"
    mkdir -p "$INSTALL_DIR"
    cp "$OUTPUT_DIR/nest" "$BINARY_PATH"
    chmod +x "$BINARY_PATH"
    echo "   ${CHECK} Binary installed to ${BOLD}${BINARY_PATH}${RESET}"
    
    # Check if install directory is in PATH
    if ! echo "${PATH}" | grep -q "${INSTALL_DIR}"; then
        echo ""
        echo "   ${WARN} ${INSTALL_DIR} is not in your PATH${RESET}"
        echo "   ${ARROW} Add this line to your shell config (~/.bashrc, ~/.zshrc, etc.):"
        echo "      ${BOLD}export PATH=\"\${HOME}/.local/bin:\${PATH}\"${RESET}"
    fi
fi

# Success message
echo ""
if [ "$HAS_COLORS" = "1" ]; then
    echo "${BOLD}${GREEN}╔════════════════════════════════════════╗${RESET}"
    echo "${BOLD}${GREEN}║${RESET}  ${CHECK} ${BOLD}Build successful!${RESET}              ${BOLD}${GREEN}║${RESET}"
    echo "${BOLD}${GREEN}╚════════════════════════════════════════╝${RESET}"
else
    echo "${CHECK} ${BOLD}Build successful!${RESET}"
    echo "========================================"
fi
echo ""
echo "   Version: ${BOLD}${CURRENT_VERSION}${RESET}"
if [ "$SKIP_INSTALL" = false ]; then
    echo "   Installed to: ${BOLD}${BINARY_PATH}${RESET}"
    echo "   ${INFO} Run ${BOLD}nest --version${RESET} to verify installation"
else
    echo "   Binary location: ${BOLD}${OUTPUT_DIR}/nest${RESET}"
fi
echo ""
echo "   ${CHECK} This binary is statically linked and should work on any Linux system"
echo "   ${INFO} No GLIBC dependencies - works on very old systems"
if [ "$SKIP_INSTALL" = false ]; then
    echo "   ${INFO} Test: ${BOLD}nest --version${RESET}"
else
    echo "   ${INFO} Test: ${BOLD}${OUTPUT_DIR}/nest --version${RESET}"
fi
echo ""
