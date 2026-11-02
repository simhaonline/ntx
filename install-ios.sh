#!/usr/bin/env sh
# ==============================================================================
#  Nityaksha AI Agent Control Plane CLI (ntx) iOS Terminal Installer
#  Domain: simhaonline.ai
#  Usage:  curl -fsSL https://simhaonline.ai/install-ios.sh | sh
#  Supports: iOS / iPadOS terminals (iSH Shell Alpine Linux, a-Shell Darwin, UTM)
# ==============================================================================

set -e

printf "\033[0;36m"
printf "================================================================================\n"
printf "                   NITYAKSHA - An AI Concierge Platform                         \n"
printf "  Engineered for high-velocity, deterministic, and NITYAKSHA - An AI Concierge Platform in pure Rust.  \n"
printf "================================================================================\n"
printf "\033[0m"

INSTALL_DIR="$HOME/bin"
mkdir -p "$INSTALL_DIR"
mkdir -p "$HOME/.nityaksha/db"
mkdir -p "$HOME/.nityaksha/cas-cache"
mkdir -p "$HOME/.nityaksha/workspaces"

BIN_PATH="$INSTALL_DIR/ntx"
RAW_URL="https://raw.githubusercontent.com/simhaonline/ntx/main/ntx"

printf "\033[0;36m[1/3] Downloading ntx binary to %s...\033[0m\n" "$BIN_PATH"

if command -v curl >/dev/null 2>&1; then
    curl -fsSL --connect-timeout 15 "$RAW_URL" -o "$BIN_PATH"
elif command -v wget >/dev/null 2>&1; then
    wget -qO "$BIN_PATH" "$RAW_URL"
else
    printf "\033[0;31m[ERROR] Neither curl nor wget found. Please install curl via apk or pkg.\033[0m\n"
    exit 1
fi

chmod +x "$BIN_PATH"

printf "\033[0;36m[2/3] Configuring shell environment...\033[0m\n"

for rc in "$HOME/.profile" "$HOME/.bashrc" "$HOME/.ashrc" "$HOME/.zshrc"; do
    if [ -f "$rc" ]; then
        if ! grep -q "$INSTALL_DIR" "$rc" 2>/dev/null; then
            printf "\nexport PATH=\"%s:\$PATH\"\n" "$INSTALL_DIR" >> "$rc"
            printf "\033[0;32m[INFO] Appended PATH to %s\033[0m\n" "$rc"
        fi
    fi
done

export PATH="$INSTALL_DIR:$PATH"

printf "\033[0;36m[3/3] Verifying installation on iOS...\033[0m\n"

if "$BIN_PATH" --version >/dev/null 2>&1; then
    "$BIN_PATH" --version
    printf "\n\033[0;32m✓ Nityaksha Control Plane (ntx) successfully installed on iOS!\033[0m\n"
    printf "Binary Location: %s\n\n" "$BIN_PATH"
    printf "Run 'ntx help' or 'ntx status' to get started.\n"
    printf "WebUI: run 'ntx ui --port 8080' and open http://localhost:8080 in Safari.\n"
else
    printf "\033[1;33m[NOTE] Binary downloaded to %s.\033[0m\n" "$BIN_PATH"
    printf "If running inside iSH or a-Shell, ensure execution permissions with: chmod +x %s\n" "$BIN_PATH"
fi
