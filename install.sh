#!/usr/bin/env bash
# ==============================================================================
#  Nityaksha AI Agent Control Plane CLI (ntx) Universal Installer
#  Domain: simhaonline.ai
#  Usage:  curl -fsSL https://simhaonline.ai/install.sh | bash
#  Supports: Linux (x86_64, aarch64), macOS (Apple Silicon / Intel), iOS (iSH/a-Shell)
# ==============================================================================

set -euo pipefail

CYAN='\033[0;36m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
BOLD='\033[1m'
NC='\033[0m' # No Color

echo -e "${CYAN}${BOLD}"
echo "================================================================================"
echo "                   NITYAKSHA - An AI Concierge Platform                         "
echo "  Engineered for high-velocity, deterministic, and NITYAKSHA - An AI Concierge Platform in pure Rust.  "
echo "================================================================================"
echo -e "${NC}"

OS="$(uname -s | tr '[:upper:]' '[:lower:]')"
ARCH="$(uname -m)"

# Normalize architecture
case "$ARCH" in
    x86_64|amd64)
        TARGET_ARCH="x86_64"
        ;;
    aarch64|arm64)
        TARGET_ARCH="aarch64"
        ;;
    armv7l|armhf)
        TARGET_ARCH="armv7"
        ;;
    *)
        echo -e "${YELLOW}[WARN] Unknown architecture '$ARCH', defaulting to x86_64${NC}"
        TARGET_ARCH="x86_64"
        ;;
esac

# Detect iOS environments (iSH Alpine container or a-Shell on iOS)
IS_IOS=0
if [[ "$OS" == "darwin" ]] && [[ "$(uname -p 2>/dev/null)" == "iPhone"* || "$(uname -m 2>/dev/null)" == "iPad"* || -d "/private/var/mobile" ]]; then
    IS_IOS=1
elif grep -qi "ish" /proc/version 2>/dev/null || [ -f "/dev/ish" ] || [ -d "/var/mobile" ]; then
    IS_IOS=1
fi

if [ "$IS_IOS" -eq 1 ]; then
    echo -e "${GREEN}[INFO] Detected iOS Terminal Environment (iSH / a-Shell / UTM)${NC}"
    INSTALL_DIR="$HOME/bin"
else
    echo -e "${GREEN}[INFO] Detected OS: $OS ($TARGET_ARCH)${NC}"
    if [ "$(id -u)" -eq 0 ]; then
        INSTALL_DIR="/usr/local/bin"
    else
        INSTALL_DIR="$HOME/.local/bin"
    fi
fi

mkdir -p "$INSTALL_DIR"

BIN_PATH="$INSTALL_DIR/ntx"
RELEASE_URL="https://github.com/simhaonline/ntx/releases/latest/download/ntx"
RAW_URL="https://raw.githubusercontent.com/simhaonline/ntx/main/ntx"

echo -e "${CYAN}[1/4] Fetching native 'ntx' pure Rust binary...${NC}"

DOWNLOAD_SUCCESS=0

# Try direct download from repository
if curl -fsSL --connect-timeout 10 "$RAW_URL" -o "$BIN_PATH" 2>/dev/null; then
    chmod +x "$BIN_PATH"
    DOWNLOAD_SUCCESS=1
elif curl -fsSL --connect-timeout 10 "$RELEASE_URL" -o "$BIN_PATH" 2>/dev/null; then
    chmod +x "$BIN_PATH"
    DOWNLOAD_SUCCESS=1
fi

# Fallback: if cargo is installed, build from git source repository
if [ "$DOWNLOAD_SUCCESS" -eq 0 ]; then
    echo -e "${YELLOW}[INFO] Precompiled binary not reachable. Building from source via cargo...${NC}"
    if command -v cargo >/dev/null 2>&1; then
        cargo install --git https://github.com/simhaonline/ntx.git --root "$HOME/.cargo" ntx
        cp "$HOME/.cargo/bin/ntx" "$BIN_PATH"
        chmod +x "$BIN_PATH"
        DOWNLOAD_SUCCESS=1
    else
        echo -e "${RED}[ERROR] Neither prebuilt binary nor cargo build succeeded.${NC}"
        echo -e "${RED}Please check your network connection or visit https://github.com/simhaonline/ntx${NC}"
        exit 1
    fi
fi

echo -e "${CYAN}[2/4] Setting up directories and permissions...${NC}"

# Setup local policy and workspace directories
if [ "$(id -u)" -eq 0 ]; then
    mkdir -p /etc/nityaksha /var/lib/nityaksha/db /var/lib/nityaksha/cas-cache /srv/agent-workspaces
    if ! id -u nityaksha >/dev/null 2>&1; then
        useradd -r -s /usr/sbin/nologin -d /var/lib/nityaksha nityaksha 2>/dev/null || true
    fi
else
    mkdir -p "$HOME/.nityaksha/db" "$HOME/.nityaksha/cas-cache" "$HOME/.nityaksha/workspaces"
fi

echo -e "${CYAN}[3/4] Configuring shell PATH environment...${NC}"
CURRENT_SHELL="$(basename "${SHELL:-bash}")"
RC_FILE=""

case "$CURRENT_SHELL" in
    zsh)
        RC_FILE="$HOME/.zshrc"
        ;;
    bash)
        RC_FILE="$HOME/.bashrc"
        ;;
    *)
        RC_FILE="$HOME/.profile"
        ;;
esac

if [[ ":$PATH:" != *":$INSTALL_DIR:"* ]]; then
    if [ -n "$RC_FILE" ] && [ -f "$RC_FILE" ]; then
        echo "export PATH=\"$INSTALL_DIR:\$PATH\"" >> "$RC_FILE"
        echo -e "${GREEN}[INFO] Appended $INSTALL_DIR to $RC_FILE${NC}"
    fi
    export PATH="$INSTALL_DIR:$PATH"
fi

echo -e "${CYAN}[4/4] Verifying installation...${NC}"
if "$BIN_PATH" --version; then
    echo ""
    echo -e "${GREEN}${BOLD}✓ Nityaksha Control Plane (ntx) installed successfully!${NC}"
    echo -e "Binary installed to: ${BOLD}$BIN_PATH${NC}"
    echo ""
    echo "Quick Start Commands:"
    echo "  ntx doctor            - Verify system invariants, memory stack, and sandbox"
    echo "  ntx status            - View framework health, runtimes, and catalog stats"
    echo "  ntx skill list        - Explore registered proprietary skills"
    echo "  ntx harness status    - Inspect AI Harness and HITL safety gates"
    echo "  ntx ui --port 8080    - Launch workstation WebUI"
    echo "  ntx repl              - Enter interactive shell"
    echo ""
    echo -e "${CYAN}Documentation & Source: https://simhaonline.ai | https://github.com/simhaonline/ntx${NC}"
else
    echo -e "${RED}[ERROR] Verification of $BIN_PATH failed.${NC}"
    exit 1
fi
