#!/bin/bash
# Install script for old systems with older GLIBC versions
# Uses Docker to build on an older system image for compatibility
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
else
    CHECK="${GREEN}[OK]${RESET}"
    ARROW="${BLUE}=>${RESET}"
    INFO="${BLUE}[i]${RESET}"
fi

# Get project root directory
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR"

# Configuration
DOCKER_IMAGE="${DOCKER_IMAGE:-ubuntu:20.04}"
RUST_VERSION="${RUST_VERSION:-stable}"
INSTALL_DIR="${HOME}/.local/bin"
BINARY_NAME="nest"
BINARY_PATH="${INSTALL_DIR}/${BINARY_NAME}"
OUTPUT_DIR="${SCRIPT_DIR}/target/old-system-release"

# Default options
SKIP_INSTALL=false
CUSTOM_VERSION=""
CUSTOM_INSTALL_DIR=""
CUSTOM_OUTPUT_DIR=""

# Function to show usage
show_usage() {
    echo "Usage: $0 [OPTIONS]"
    echo ""
    echo "Options:"
    echo "  --docker-image IMAGE     Docker image to use (default: ubuntu:20.04)"
    echo "  --rust-version VERSION   Rust version to use (default: stable)"
    echo "  --version VERSION        Use specific version instead of reading from Cargo.toml"
    echo "  --install-dir PATH      Install binary to custom directory (default: ~/.local/bin)"
    echo "  --output-dir PATH        Output directory for binary (default: target/old-system-release)"
    echo "  --skip-install           Don't install binary after build"
    echo "  --help                   Show this help message"
    echo ""
    echo "Examples:"
    echo "  $0                                    # Build with default settings"
    echo "  $0 --docker-image debian:bullseye    # Use Debian Bullseye"
    echo "  $0 --skip-install                     # Build without installing"
    echo ""
    echo "Environment variables:"
    echo "  DOCKER_IMAGE             Docker image to use"
    echo "  RUST_VERSION             Rust version to use"
}

# Parse command line arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        --docker-image)
            if [ -z "$2" ]; then
                echo "${RED}Error: --docker-image requires an image name${RESET}" >&2
                exit 1
            fi
            DOCKER_IMAGE="$2"
            shift 2
            ;;
        --rust-version)
            if [ -z "$2" ]; then
                echo "${RED}Error: --rust-version requires a version${RESET}" >&2
                exit 1
            fi
            RUST_VERSION="$2"
            shift 2
            ;;
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
        --output-dir)
            if [ -z "$2" ]; then
                echo "${RED}Error: --output-dir requires a path${RESET}" >&2
                exit 1
            fi
            CUSTOM_OUTPUT_DIR="$2"
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

# Apply custom directories if provided
if [ -n "$CUSTOM_INSTALL_DIR" ]; then
    INSTALL_DIR="$CUSTOM_INSTALL_DIR"
    BINARY_PATH="${INSTALL_DIR}/${BINARY_NAME}"
fi

if [ -n "$CUSTOM_OUTPUT_DIR" ]; then
    OUTPUT_DIR="$CUSTOM_OUTPUT_DIR"
fi

# Check if Docker is available
if ! command -v docker > /dev/null 2>&1; then
    echo "${RED}Error: Docker is not installed or not in PATH${RESET}" >&2
    echo "Please install Docker to use this script"
    exit 1
fi

# Function to read version from Cargo.toml
read_version() {
    if [ ! -f "Cargo.toml" ]; then
        echo "${RED}Error: Cargo.toml not found${RESET}" >&2
        exit 1
    fi
    grep -E '^version\s*=' "Cargo.toml" | sed -E 's/^version\s*=\s*"([^"]+)".*/\1/' | tr -d '[:space:]'
}

# Print header
echo ""
if [ "$HAS_COLORS" = "1" ]; then
    echo "${BOLD}${BLUE}╔════════════════════════════════════════╗${RESET}"
    echo "${BOLD}${BLUE}║${RESET}  ${BOLD}Nest Build for Old Systems${RESET}          ${BOLD}${BLUE}║${RESET}"
    echo "${BOLD}${BLUE}╚════════════════════════════════════════╝${RESET}"
else
    echo "${BOLD}Nest Build for Old Systems${RESET}"
    echo "========================================"
fi
echo ""

# Read current version
echo "${INFO} ${BOLD}Reading version...${RESET}"
if [ -n "$CUSTOM_VERSION" ]; then
    CURRENT_VERSION="$CUSTOM_VERSION"
    echo "   ${ARROW} Using custom version: ${BOLD}${CURRENT_VERSION}${RESET}"
else
    CURRENT_VERSION=$(read_version)
    echo "   ${ARROW} Current version: ${BOLD}${CURRENT_VERSION}${RESET}"
fi

# Show build configuration
echo ""
echo "${INFO} ${BOLD}Build configuration:${RESET}"
echo "   ${ARROW} Docker image: ${BOLD}${DOCKER_IMAGE}${RESET}"
echo "   ${ARROW} Rust version: ${BOLD}${RUST_VERSION}${RESET}"
echo "   ${ARROW} Output directory: ${BOLD}${OUTPUT_DIR}${RESET}"

# Create output directory
mkdir -p "$OUTPUT_DIR"

# Create Dockerfile for build
echo ""
echo "${INFO} ${BOLD}Preparing Docker build environment...${RESET}"

DOCKERFILE=$(cat <<EOF
FROM ${DOCKER_IMAGE}

# Install dependencies
RUN apt-get update && apt-get install -y \\
    curl \\
    build-essential \\
    pkg-config \\
    libssl-dev \\
    ca-certificates \\
    && rm -rf /var/lib/apt/lists/*

# Install Rust
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --default-toolchain ${RUST_VERSION}
ENV PATH="/root/.cargo/bin:\$PATH"

# Set working directory
WORKDIR /build

# Copy project files
COPY Cargo.toml Cargo.lock ./
COPY src ./src

# Build project
RUN cargo build --release

# The binary will be at /build/target/release/nest
EOF
)

echo "$DOCKERFILE" > "$OUTPUT_DIR/Dockerfile"
echo "   ${CHECK} Dockerfile created"

# Build Docker image
echo ""
echo "${INFO} ${BOLD}Building Docker image...${RESET}"
DOCKER_IMAGE_NAME="nest-build-$(echo "$DOCKER_IMAGE" | tr '/:' '-')"
if docker build -t "$DOCKER_IMAGE_NAME" -f "$OUTPUT_DIR/Dockerfile" . > /dev/null 2>&1; then
    echo "   ${CHECK} Docker image built successfully"
else
    echo "   ${RED}✗ Docker build failed${RESET}" >&2
    echo ""
    echo "Running build with output for debugging..."
    docker build -t "$DOCKER_IMAGE_NAME" -f "$OUTPUT_DIR/Dockerfile" .
    exit 1
fi

# Extract binary from container
echo ""
echo "${INFO} ${BOLD}Extracting binary from container...${RESET}"
CONTAINER_ID=$(docker create "$DOCKER_IMAGE_NAME")
docker cp "$CONTAINER_ID:/build/target/release/nest" "$OUTPUT_DIR/nest"
docker rm "$CONTAINER_ID" > /dev/null 2>&1

if [ -f "$OUTPUT_DIR/nest" ]; then
    chmod +x "$OUTPUT_DIR/nest"
    echo "   ${CHECK} Binary extracted to ${BOLD}${OUTPUT_DIR}/nest${RESET}"
    
    # Show binary info
    echo ""
    echo "${INFO} ${BOLD}Binary information:${RESET}"
    if command -v file > /dev/null 2>&1; then
        echo "   ${ARROW} Type: $(file "$OUTPUT_DIR/nest" | cut -d: -f2-)"
    fi
    if command -v ldd > /dev/null 2>&1 && ldd "$OUTPUT_DIR/nest" > /dev/null 2>&1; then
        echo "   ${ARROW} GLIBC version:"
        ldd --version 2>&1 | head -1 | sed 's/^/      /'
        echo "   ${ARROW} Dependencies:"
        ldd "$OUTPUT_DIR/nest" 2>&1 | grep -E 'libc\.so|GLIBC' | sed 's/^/      /' || echo "      (static or no GLIBC dependencies)"
    fi
else
    echo "   ${RED}✗ Failed to extract binary${RESET}" >&2
    exit 1
fi

# Install binary (if not skipped)
if [ "$SKIP_INSTALL" = false ]; then
    echo ""
    echo "${INFO} ${BOLD}Installing binary...${RESET}"
    mkdir -p "$INSTALL_DIR"
    cp "$OUTPUT_DIR/nest" "$BINARY_PATH"
    echo "   ${CHECK} Binary installed to ${BOLD}${BINARY_PATH}${RESET}"
fi

# Cleanup Dockerfile (optional, can be kept for debugging)
# rm -f "$OUTPUT_DIR/Dockerfile"

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
echo "   Binary location: ${BOLD}${OUTPUT_DIR}/nest${RESET}"
if [ "$SKIP_INSTALL" = false ]; then
    echo "   Installed to: ${BOLD}${BINARY_PATH}${RESET}"
fi
echo ""
echo "   ${INFO} This binary should work on systems with older GLIBC versions"
echo "   ${INFO} Test on target system: ${BOLD}${OUTPUT_DIR}/nest --version${RESET}"
echo ""

