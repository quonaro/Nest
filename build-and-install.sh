#!/bin/bash
# Build and install script for Nest CLI
# Reads version from Cargo.toml, builds, installs, and increments version

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
else
    CHECK="${GREEN}[OK]${RESET}"
    ARROW="${BLUE}=>${RESET}"
    INFO="${BLUE}[i]${RESET}"
fi

# Get project root directory
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR"

# File paths
CARGO_TOML="Cargo.toml"
INSTALL_DIR="/usr/local/bin"
BINARY_NAME="nest"
BINARY_PATH="${INSTALL_DIR}/${BINARY_NAME}"

# Default options
CUSTOM_VERSION=""
CUSTOM_INSTALL_DIR=""
STATIC_BUILD=false
CLEAN_BUILD=false

OS_NAME="$(uname -s)"

# Function to run command with sudo if needed
run_elevated() {
    local target_path="$1"
    shift
    local target_dir="$(dirname "$target_path")"
    
    # We need sudo if:
    # 1. The directory is not writable (for creating/renaming files)
    # 2. OR the file exists and is not writable
    if [ ! -w "$target_dir" ] || ([ -e "$target_path" ] && [ ! -w "$target_path" ]); then
        echo "   ${YELLOW}Elevation required to access ${target_path}${RESET}"
        sudo "$@"
    else
        "$@"
    fi
}

# Function to show usage
show_usage() {
    echo "Usage: $0 [OPTIONS]"
    echo ""
    echo "Options:"
    echo "  --version VERSION       Use specific version instead of reading from Cargo.toml"
    echo "  --install-dir PATH      Install binary to custom directory (default: ~/.local/bin)"
    echo "  --static                Build static binary (Linux only)"
    echo "  --clean                 Clean build artifacts before building"
    echo "  --help                  Show this help message"
    echo ""
    echo "Examples:"
    echo "  $0                                    # Build and install with version from Cargo.toml"
    echo "  $0 --version 1.0.0                    # Build and install with specific version"
    echo "  $0 --install-dir /usr/local/bin       # Install to custom directory"
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
        --static)
            STATIC_BUILD=true
            shift
            ;;
        --clean)
            CLEAN_BUILD=true
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

# Function to read version from Cargo.toml
read_version() {
    if [ ! -f "$CARGO_TOML" ]; then
        echo "${RED}Error: ${CARGO_TOML} not found${RESET}" >&2
        exit 1
    fi
    # Extract version from Cargo.toml
    grep -E '^version\s*=' "$CARGO_TOML" | sed -E 's/^version\s*=\s*"([^"]+)".*/\1/' | tr -d '[:space:]'
}



# Function to update version in Cargo.toml
update_cargo_version() {
    local version="$1"
    local files=("$CARGO_TOML")
    
    # Add all crate Cargo.toml files
    if [ -d "crates" ]; then
        for crate_toml in crates/*/Cargo.toml; do
            if [ -f "$crate_toml" ]; then
                files+=("$crate_toml")
            fi
        done
    fi

    echo "   ${INFO} Updating version in: ${files[*]}"

    for file in "${files[@]}"; do
        # Use sed to update version in Cargo.toml
        if [[ "$OSTYPE" == "darwin"* ]]; then
            # macOS uses BSD sed
            sed -i '' "s/^version = \".*\"/version = \"${version}\"/" "$file"
        else
            # Linux uses GNU sed
            sed -i "s/^version = \".*\"/version = \"${version}\"/" "$file"
        fi
    done
}


# Print header
echo ""
if [ "$HAS_COLORS" = "1" ]; then
    echo "${BOLD}${BLUE}╔════════════════════════════════════════╗${RESET}"
    echo "${BOLD}${BLUE}║${RESET}  ${BOLD}Nest Build & Install${RESET}                  ${BOLD}${BLUE}║${RESET}"
    echo "${BOLD}${BLUE}╚════════════════════════════════════════╝${RESET}"
else
    echo "${BOLD}Nest Build & Install${RESET}"
    echo "========================================"
fi
echo ""

# Read current version
echo "${INFO} ${BOLD}Reading version...${RESET}"
if [ -n "$CUSTOM_VERSION" ]; then
    CURRENT_VERSION="$CUSTOM_VERSION"
    echo "   ${ARROW} Using custom version: ${BOLD}${CURRENT_VERSION}${RESET}"
    # Update Cargo.toml with custom version
    echo ""
    echo "${INFO} ${BOLD}Updating Cargo.toml...${RESET}"
    update_cargo_version "$CURRENT_VERSION"
    echo "   ${CHECK} Version updated to ${BOLD}${CURRENT_VERSION}${RESET}"
else
    CURRENT_VERSION=$(read_version)
    echo "   ${ARROW} Current version: ${BOLD}${CURRENT_VERSION}${RESET}"
fi

# Configure build type (static or default)
BUILD_TARGET_DIR="target/release"
BUILD_CMD="cargo build --release"

if [ "$STATIC_BUILD" = true ]; then
    if [ "$OS_NAME" != "Linux" ]; then
        echo "${RED}Error: --static is only supported on Linux hosts${RESET}" >&2
        exit 1
    fi
    TARGET_TRIPLE="x86_64-unknown-linux-musl"
    BUILD_TARGET_DIR="target/${TARGET_TRIPLE}/release"
    echo ""
    echo "${INFO} ${BOLD}Enabling static Linux build (musl)...${RESET}"
    echo "   ${ARROW} Target: ${BOLD}${TARGET_TRIPLE}${RESET}"
    if command -v rustup > /dev/null 2>&1; then
        echo "   ${ARROW} Ensuring Rust target ${BOLD}${TARGET_TRIPLE}${RESET} is installed..."
        rustup target add "${TARGET_TRIPLE}" >/dev/null 2>&1 || true
    else
        echo "   ${YELLOW}Warning: rustup not found, assuming musl target is already installed${RESET}"
    fi
    BUILD_CMD="cargo build --release --target ${TARGET_TRIPLE}"
    BUILD_CMD="cargo build --release --target ${TARGET_TRIPLE}"
fi

# Force rebuild if requested or always ensures binaries are fresh
if [ "$CLEAN_BUILD" = true ]; then
    echo ""
    echo "${INFO} ${BOLD}Cleaning build artifacts...${RESET}"
    cargo clean
else
    # Touch main source files to force cargo to rebuild/relink
    # even if no changes were made.
    if [ -f "crates/nest-cli/src/main.rs" ]; then
        touch "crates/nest-cli/src/main.rs"
    fi
    if [ -f "crates/nest-ui/src/main.rs" ]; then
        touch "crates/nest-ui/src/main.rs"
    fi
fi

# Build project
echo ""
echo "${INFO} ${BOLD}Building project...${RESET}"
echo "   ${ARROW} Command: ${BOLD}${BUILD_CMD}${RESET}"
if eval "${BUILD_CMD}"; then
    echo "   ${CHECK} Build successful"
else
    echo "   ${RED}✗ Build failed${RESET}" >&2
    exit 1
fi

# Create install directory if it doesn't exist
echo ""
echo "${INFO} ${BOLD}Preparing installation...${RESET}"
if [ ! -d "$INSTALL_DIR" ]; then
    run_elevated "$INSTALL_DIR" mkdir -p "$INSTALL_DIR"
fi
echo "   ${CHECK} Install directory ready: ${BOLD}${INSTALL_DIR}${RESET}"

# Install binary
echo ""
echo "${INFO} ${BOLD}Installing binary...${RESET}"
RELEASE_BINARY="${BUILD_TARGET_DIR}/${BINARY_NAME}"
if [ ! -f "$RELEASE_BINARY" ]; then
    echo "   ${RED}✗ Binary not found at ${RELEASE_BINARY}${RESET}" >&2
    exit 1
fi

if command -v install >/dev/null 2>&1; then
    run_elevated "$BINARY_PATH" install -m 755 "$RELEASE_BINARY" "$BINARY_PATH"
else
    # Fallback if install is missing (rare)
    run_elevated "$BINARY_PATH" cp -f "$RELEASE_BINARY" "$BINARY_PATH"
    run_elevated "$BINARY_PATH" chmod +x "$BINARY_PATH"
fi
echo "   ${CHECK} Binary installed to ${BOLD}${BINARY_PATH}${RESET}"

NESTUI_BINARY_NAME="nestui"
RELEASE_NESTUI="${BUILD_TARGET_DIR}/${NESTUI_BINARY_NAME}"
NESTUI_BINARY_PATH="${INSTALL_DIR}/${NESTUI_BINARY_NAME}"

if [ -f "$RELEASE_NESTUI" ]; then
    echo ""
    echo "${INFO} ${BOLD}Installing nestui binary...${RESET}"
    if command -v install >/dev/null 2>&1; then
        run_elevated "$NESTUI_BINARY_PATH" install -m 755 "$RELEASE_NESTUI" "$NESTUI_BINARY_PATH"
    else
        run_elevated "$NESTUI_BINARY_PATH" cp -f "$RELEASE_NESTUI" "$NESTUI_BINARY_PATH"
        run_elevated "$NESTUI_BINARY_PATH" chmod +x "$NESTUI_BINARY_PATH"
    fi
    echo "   ${CHECK} Nest UI installed to ${BOLD}${NESTUI_BINARY_PATH}${RESET}"
else
    echo "   ${YELLOW}Warning: nestui binary not found at ${RELEASE_NESTUI}${RESET}"
fi

# Success message
echo ""
if [ "$HAS_COLORS" = "1" ]; then
    echo "${BOLD}${GREEN}╔════════════════════════════════════════╗${RESET}"
    echo "${BOLD}${GREEN}║${RESET}  ${CHECK} ${BOLD}Build and install successful!${RESET}     ${BOLD}${GREEN}║${RESET}"
    echo "${BOLD}${GREEN}╚════════════════════════════════════════╝${RESET}"
else
    echo "${CHECK} ${BOLD}Build and install successful!${RESET}"
    echo "========================================"
fi
echo ""
echo "   Installed version: ${BOLD}${CURRENT_VERSION}${RESET}"
echo "   Run ${BOLD}nest --version${RESET} to verify installation."
echo ""
