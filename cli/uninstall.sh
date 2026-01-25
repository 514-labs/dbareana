#!/bin/bash
# dbarena uninstallation script

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Configuration
INSTALL_DIR="${INSTALL_DIR:-/usr/local/bin}"
BINARY_NAME="dbarena"
CONFIG_DIR="${HOME}/.config/dbarena"
DATA_DIR="${HOME}/.local/share/dbarena"

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

# Main uninstallation
main() {
    echo ""
    echo "=================================="
    echo "dbarena Uninstallation Script"
    echo "=================================="
    echo ""

    # Check if dbarena is installed
    if [ ! -f "${INSTALL_DIR}/${BINARY_NAME}" ]; then
        error "dbarena is not installed at ${INSTALL_DIR}/${BINARY_NAME}"
        exit 1
    fi

    info "Found dbarena at ${INSTALL_DIR}/${BINARY_NAME}"

    # Check if we need sudo
    NEED_SUDO=false
    if [ ! -w "$INSTALL_DIR" ]; then
        NEED_SUDO=true
    fi

    # Remove binary
    info "Removing binary..."
    if [ "$NEED_SUDO" = true ]; then
        info "Administrator password required..."
        if sudo rm -f "${INSTALL_DIR}/${BINARY_NAME}"; then
            success "Removed binary"
        else
            error "Failed to remove binary"
            exit 1
        fi
    else
        if rm -f "${INSTALL_DIR}/${BINARY_NAME}"; then
            success "Removed binary"
        else
            error "Failed to remove binary"
            exit 1
        fi
    fi

    # Ask about config and data directories
    echo ""
    if [ -d "$CONFIG_DIR" ] || [ -d "$DATA_DIR" ]; then
        info "Found user data directories:"
        [ -d "$CONFIG_DIR" ] && echo "  - Config: $CONFIG_DIR"
        [ -d "$DATA_DIR" ] && echo "  - Data: $DATA_DIR"
        echo ""
        read -p "Remove user data directories? (y/N) " -n 1 -r
        echo
        if [[ $REPLY =~ ^[Yy]$ ]]; then
            [ -d "$CONFIG_DIR" ] && rm -rf "$CONFIG_DIR" && success "Removed config directory"
            [ -d "$DATA_DIR" ] && rm -rf "$DATA_DIR" && success "Removed data directory"
        else
            info "User data directories kept"
        fi
    fi

    echo ""
    echo "=================================="
    echo "Uninstallation Complete!"
    echo "=================================="
    echo ""
    success "dbarena has been uninstalled"

    if [ -d "$CONFIG_DIR" ] || [ -d "$DATA_DIR" ]; then
        echo ""
        info "To manually remove remaining data:"
        [ -d "$CONFIG_DIR" ] && echo "  rm -rf $CONFIG_DIR"
        [ -d "$DATA_DIR" ] && echo "  rm -rf $DATA_DIR"
    fi

    echo ""
}

# Run main function
main "$@"
