#!/bin/bash
# dbarena installation script
# Usage: curl -sSL https://raw.githubusercontent.com/514-labs/dbareana/main/install.sh | bash

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Configuration
REPO="514-labs/dbareana"
VERSION="${DBARENA_VERSION:-v0.3.0}"
INSTALL_DIR="${INSTALL_DIR:-/usr/local/bin}"
BINARY_NAME="dbarena"

# Helper functions
info() {
    echo -e "${BLUE}ℹ${NC} $1"
}

success() {
    echo -e "${GREEN}✓${NC} $1"
}

error() {
    echo -e "${RED}✗${NC} $1"
}

warn() {
    echo -e "${YELLOW}⚠${NC} $1"
}

# Detect OS
detect_os() {
    case "$(uname -s)" in
        Darwin*)
            OS="darwin"
            ;;
        Linux*)
            OS="linux"
            ;;
        *)
            error "Unsupported operating system: $(uname -s)"
            exit 1
            ;;
    esac
}

# Detect architecture
detect_arch() {
    ARCH="$(uname -m)"
    case "$ARCH" in
        x86_64)
            ARCH="x86_64"
            ;;
        arm64|aarch64)
            ARCH="aarch64"
            ;;
        *)
            error "Unsupported architecture: $ARCH"
            exit 1
            ;;
    esac
}

# Check if command exists
command_exists() {
    command -v "$1" >/dev/null 2>&1
}

# Download file
download() {
    local url="$1"
    local output="$2"

    if command_exists curl; then
        curl -sSL -o "$output" "$url"
    elif command_exists wget; then
        wget -q -O "$output" "$url"
    else
        error "Neither curl nor wget is available. Please install one of them."
        exit 1
    fi
}

# Main installation
main() {
    echo ""
    echo "=================================="
    echo "dbarena Installation Script"
    echo "=================================="
    echo ""

    # Detect system
    info "Detecting system..."
    detect_os
    detect_arch
    success "Detected: $OS-$ARCH"

    # Construct download URL
    # Note: Currently only macOS ARM64 binary is available
    # Format: dbarena (direct binary name on release)
    DOWNLOAD_URL="https://github.com/${REPO}/releases/download/${VERSION}/${BINARY_NAME}"
    CHECKSUM_URL="https://github.com/${REPO}/releases/download/${VERSION}/${BINARY_NAME}-${VERSION}-${ARCH}-apple-${OS}.sha256"

    # Check if this platform is supported
    if [ "$OS" = "darwin" ] && [ "$ARCH" = "aarch64" ]; then
        success "Platform is supported"
    else
        warn "Pre-built binary not available for $OS-$ARCH"
        info "Available platforms:"
        echo "  - macOS Apple Silicon (darwin-aarch64)"
        echo ""
        info "You can build from source:"
        echo "  git clone https://github.com/${REPO}.git"
        echo "  cd dbareana"
        echo "  cargo build --release"
        echo "  sudo cp target/release/dbarena ${INSTALL_DIR}/"
        exit 1
    fi

    # Create temporary directory
    TMP_DIR="$(mktemp -d)"
    trap "rm -rf '$TMP_DIR'" EXIT

    # Download binary
    info "Downloading dbarena ${VERSION}..."
    if ! download "$DOWNLOAD_URL" "$TMP_DIR/$BINARY_NAME"; then
        error "Failed to download binary from $DOWNLOAD_URL"
        exit 1
    fi
    success "Downloaded binary"

    # Download checksum (optional, may not exist)
    info "Downloading checksum..."
    if download "$CHECKSUM_URL" "$TMP_DIR/checksum.sha256" 2>/dev/null; then
        success "Downloaded checksum"

        # Verify checksum
        info "Verifying checksum..."
        cd "$TMP_DIR"
        if echo "$(cat checksum.sha256 | awk '{print $1}')  $BINARY_NAME" | shasum -a 256 -c - >/dev/null 2>&1; then
            success "Checksum verified"
        else
            warn "Checksum verification failed, but continuing anyway"
        fi
        cd - >/dev/null
    else
        warn "Checksum file not available, skipping verification"
    fi

    # Make binary executable
    chmod +x "$TMP_DIR/$BINARY_NAME"

    # Check if we need sudo
    NEED_SUDO=false
    if [ ! -w "$INSTALL_DIR" ]; then
        NEED_SUDO=true
    fi

    # Install binary
    info "Installing to $INSTALL_DIR..."
    if [ "$NEED_SUDO" = true ]; then
        if ! command_exists sudo; then
            error "sudo is required to install to $INSTALL_DIR but is not available"
            exit 1
        fi

        info "Administrator password required..."
        if sudo cp "$TMP_DIR/$BINARY_NAME" "$INSTALL_DIR/$BINARY_NAME"; then
            success "Installed to $INSTALL_DIR/$BINARY_NAME"
        else
            error "Failed to install binary"
            exit 1
        fi
    else
        if cp "$TMP_DIR/$BINARY_NAME" "$INSTALL_DIR/$BINARY_NAME"; then
            success "Installed to $INSTALL_DIR/$BINARY_NAME"
        else
            error "Failed to install binary"
            exit 1
        fi
    fi

    # Verify installation
    info "Verifying installation..."
    if command_exists dbarena; then
        INSTALLED_VERSION="$(dbarena --version | awk '{print $2}')"
        success "dbarena ${INSTALLED_VERSION} installed successfully!"
    else
        error "Installation completed but dbarena is not in PATH"
        warn "You may need to add ${INSTALL_DIR} to your PATH"
        warn "Add this to your ~/.bashrc or ~/.zshrc:"
        echo "  export PATH=\"${INSTALL_DIR}:\$PATH\""
        exit 1
    fi

    echo ""
    echo "=================================="
    echo "Installation Complete!"
    echo "=================================="
    echo ""
    info "Get started:"
    echo "  dbarena --help"
    echo "  dbarena create postgres"
    echo "  dbarena config init > dbarena.toml"
    echo ""
    info "Documentation:"
    echo "  https://github.com/${REPO}/blob/main/README.md"
    echo ""
    info "Examples:"
    echo "  https://github.com/${REPO}/tree/main/examples"
    echo ""
}

# Run main function
main "$@"
