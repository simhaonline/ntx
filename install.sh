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

# Early PATH export so cargo and subshells immediately recognize paths
export PATH="$INSTALL_DIR:$HOME/.cargo/bin:/root/.cargo/bin:$PATH"

BIN_PATH="$INSTALL_DIR/ntx"
RELEASE_URL="https://github.com/simhaonline/ntx/releases/latest/download/ntx"
RAW_URL="https://raw.githubusercontent.com/simhaonline/ntx/main/ntx"

echo -e "${CYAN}[1/4] Fetching native 'ntx' pure Rust binary...${NC}"

# Prevent 'Text file busy' by stopping active daemon and terminating old processes
if command -v systemctl >/dev/null 2>&1; then
    systemctl stop nityaksha-webui.service >/dev/null 2>&1 || true
fi
pkill -x ntx >/dev/null 2>&1 || true

DOWNLOAD_SUCCESS=0
TEMP_BIN="$(mktemp "${INSTALL_DIR}/.ntx.XXXXXX" 2>/dev/null || mktemp /tmp/ntx.XXXXXX)"

# 1. Prefer existing local release build if available
if [ -f "/root/install/nityaksha-core/target/release/ntx" ]; then
    cp "/root/install/nityaksha-core/target/release/ntx" "$TEMP_BIN"
    chmod +x "$TEMP_BIN"
    mv -f "$TEMP_BIN" "$BIN_PATH"
    DOWNLOAD_SUCCESS=1
# 2. Try direct download from raw GitHub repository
elif curl -fsSL --connect-timeout 10 "$RAW_URL" -o "$TEMP_BIN" 2>/dev/null; then
    chmod +x "$TEMP_BIN"
    mv -f "$TEMP_BIN" "$BIN_PATH"
    DOWNLOAD_SUCCESS=1
# 3. Try GitHub release artifact
elif curl -fsSL --connect-timeout 10 "$RELEASE_URL" -o "$TEMP_BIN" 2>/dev/null; then
    chmod +x "$TEMP_BIN"
    mv -f "$TEMP_BIN" "$BIN_PATH"
    DOWNLOAD_SUCCESS=1
fi

# 4. Fallback: build from git source repository using cargo
if [ "$DOWNLOAD_SUCCESS" -eq 0 ]; then
    echo -e "${YELLOW}[INFO] Precompiled binary not reachable. Building from source via cargo...${NC}"
    if command -v cargo >/dev/null 2>&1; then
        cargo install --quiet --git https://github.com/simhaonline/ntx.git --root "$HOME/.cargo" ntx
        cp "$HOME/.cargo/bin/ntx" "$TEMP_BIN"
        chmod +x "$TEMP_BIN"
        mv -f "$TEMP_BIN" "$BIN_PATH"
        DOWNLOAD_SUCCESS=1
    else
        rm -f "$TEMP_BIN"
        echo -e "${RED}[ERROR] Neither prebuilt binary nor cargo build succeeded.${NC}"
        echo -e "${RED}Please check your network connection or visit https://github.com/simhaonline/ntx${NC}"
        exit 1
    fi
fi

rm -f "$TEMP_BIN"

echo -e "${CYAN}[2/4] Initializing Nityaksha Native Agent Memory Stack (Hybrid Vectors, BM25, Multimodal CAS, WAL)...${NC}"

# Setup local policy, database, and workspace directories
if [ "$(id -u)" -eq 0 ]; then
    mkdir -p /etc/nityaksha /var/lib/nityaksha/db /var/lib/nityaksha/cas-cache /srv/agent-workspaces /opt/nityaksha/bin
    if ! id -u nityaksha >/dev/null 2>&1; then
        useradd -r -s /usr/sbin/nologin -d /var/lib/nityaksha nityaksha 2>/dev/null || true
    fi

    # Write pure Rust native policy without PostgreSQL / pgvector
    cat > /etc/nityaksha/policy.yaml <<'EOF_POLICY'
apiVersion: nityaksha.simha/v1
kind: SecurityPolicy
metadata:
  namespace: default
  name: default-enterprise-policy
  version: "1.0.0"

service:
  user: nityaksha

storage:
  source_of_truth: native_rust_agent_memory
  local_backend: native_rust_sqlite_wal
  local_db_path: "/var/lib/nityaksha/db"
  vector_backend: native_rust_simd_vector
  multimodal_cas: "/var/lib/nityaksha/cas-cache"
  artifact_store: local_cas

security:
  untrusted_sources_default: catalog_only
  allow_host_docker_socket: false
  allow_production_trading_mutation: false
  require_approval_for_elevated: true
  max_parallel_agents: 8

workspaces:
  root_dir: "/srv/agent-workspaces"
  retention_days: 30
EOF_POLICY

    # Write native Rust Agent Memory Stack DDL (WAL, Vectors, BM25, CAS)
    cat > /etc/nityaksha/schema.sql <<'EOF_SCHEMA'
-- NITYAKSHA - An AI Concierge Platform (Native Pure Rust Agent Memory Stack)
-- Relational WAL + Dense SIMD Vectors + Sparse BM25 + Multimodal CAS + MVCC Versioning
CREATE TABLE IF NOT EXISTS capabilities (
  id TEXT PRIMARY KEY,
  kind TEXT NOT NULL,
  name TEXT NOT NULL,
  origin_repo TEXT NOT NULL DEFAULT 'nityaksha/core',
  original_name TEXT NOT NULL,
  version TEXT NOT NULL DEFAULT '1.0.0',
  description TEXT NOT NULL,
  trust_tier TEXT NOT NULL DEFAULT 'approved',
  manifest_path TEXT NOT NULL,
  content_sha256 TEXT NOT NULL,
  created_at TEXT NOT NULL DEFAULT (datetime('now')),
  UNIQUE (kind, name)
);

CREATE TABLE IF NOT EXISTS vector_cache (
  id TEXT PRIMARY KEY,
  dimension INTEGER NOT NULL,
  embedding BLOB NOT NULL,
  metadata TEXT,
  updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS bm25_inverted_index (
  term TEXT NOT NULL,
  doc_id TEXT NOT NULL,
  term_frequency INTEGER NOT NULL,
  doc_length INTEGER NOT NULL,
  PRIMARY KEY (term, doc_id)
);

CREATE TABLE IF NOT EXISTS multimodal_cas (
  sha256_hash TEXT PRIMARY KEY,
  mime_type TEXT NOT NULL,
  byte_size INTEGER NOT NULL,
  file_path TEXT NOT NULL,
  created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS mvcc_state_commits (
  commit_hash TEXT PRIMARY KEY,
  parent_hash TEXT,
  snapshot_blob BLOB NOT NULL,
  author TEXT NOT NULL DEFAULT 'Simha (simhaonline)',
  created_at TEXT NOT NULL DEFAULT (datetime('now'))
);
EOF_SCHEMA
    chmod 0644 /etc/nityaksha/policy.yaml /etc/nityaksha/schema.sql
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

CARGO_BIN="$HOME/.cargo/bin"
for p in "$INSTALL_DIR" "$CARGO_BIN"; do
    if [ -d "$p" ] && [[ ":$PATH:" != *":$p:"* ]]; then
        if [ -n "$RC_FILE" ] && [ -f "$RC_FILE" ]; then
            if ! grep -q "$p" "$RC_FILE" 2>/dev/null; then
                echo "export PATH=\"$p:\$PATH\"" >> "$RC_FILE"
                echo -e "${GREEN}[INFO] Appended $p to $RC_FILE${NC}"
            fi
        fi
        export PATH="$p:$PATH"
    fi
done

# Restart service if systemd service unit exists
if [ -f "/etc/systemd/system/nityaksha-webui.service" ]; then
    systemctl daemon-reload >/dev/null 2>&1 || true
    systemctl restart nityaksha-webui.service >/dev/null 2>&1 || true
fi

echo -e "${CYAN}[4/4] Verifying installation...${NC}"
if "$BIN_PATH" --version; then
    echo ""
    echo -e "${GREEN}${BOLD}✓ NITYAKSHA - An AI Concierge Platform (ntx) installed successfully!${NC}"
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
