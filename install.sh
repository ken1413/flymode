#!/usr/bin/env bash
# FlyMode — lightweight installer for end users
# Downloads pre-built package from GitHub Releases. No Rust/Node required.
#
# Usage:
#   curl -fsSL https://raw.githubusercontent.com/ken1413/flymode/main/install.sh | bash
#   curl -fsSL https://raw.githubusercontent.com/ken1413/flymode/main/install.sh | bash -s -- --deb
set -euo pipefail

REPO="ken1413/flymode"
BIN_DIR="$HOME/.local/bin"
TMPDIR_CLEANUP=""

# ── helpers ────────────────────────────────────────────────────
RED='\033[0;31m'; GREEN='\033[0;32m'; YELLOW='\033[1;33m'; CYAN='\033[0;36m'; NC='\033[0m'

info()  { echo -e "${CYAN}[INFO]${NC}  $*"; }
ok()    { echo -e "${GREEN}[OK]${NC}    $*"; }
warn()  { echo -e "${YELLOW}[WARN]${NC}  $*"; }
fail()  { echo -e "${RED}[FAIL]${NC}  $*"; exit 1; }

need_cmd() { command -v "$1" &>/dev/null; }

# ── parse args ────────────────────────────────────────────────
USE_APPIMAGE=true
for arg in "$@"; do
  case "$arg" in
    --deb) USE_APPIMAGE=false ;;
    --help|-h)
      echo "Usage: install.sh [--deb]"
      echo ""
      echo "  Default:  AppImage install to ~/.local/bin (no sudo needed)"
      echo "  --deb     Install .deb system package instead (requires sudo)"
      echo ""
      exit 0
      ;;
  esac
done

# ── detect platform ───────────────────────────────────────────
detect_platform() {
  case "$(uname -s)" in
    Linux*)  OS="linux" ;;
    Darwin*) fail "macOS is not yet supported for pre-built install. Use setup.sh to build from source." ;;
    *)       fail "Unsupported OS: $(uname -s)" ;;
  esac

  ARCH="$(uname -m)"
  case "$ARCH" in
    x86_64)  ARCH_DEB="amd64" ;;
    aarch64) ARCH_DEB="arm64" ;;
    *)       fail "Unsupported architecture: $ARCH" ;;
  esac

  ok "Platform: Linux $ARCH_DEB"
}

# ── fetch latest release ─────────────────────────────────────
fetch_release_info() {
  info "Fetching latest release..."

  if need_cmd gh && gh auth status &>/dev/null 2>&1; then
    RELEASE_TAG=$(gh release view --repo "$REPO" --json tagName -q '.tagName' 2>/dev/null) || true
  fi

  if [ -z "${RELEASE_TAG:-}" ]; then
    # Fallback: GitHub API (works without auth for public repos)
    RELEASE_TAG=$(curl -fsSL "https://api.github.com/repos/$REPO/releases/latest" \
      | grep '"tag_name"' | head -1 | sed 's/.*: "\(.*\)".*/\1/')
  fi

  if [ -z "${RELEASE_TAG:-}" ]; then
    fail "No releases found. The project may not have published a release yet."
  fi

  VERSION="${RELEASE_TAG#v}"
  ok "Latest release: $RELEASE_TAG (v$VERSION)"
}

# ── download ──────────────────────────────────────────────────
download_and_install() {
  TMPDIR_CLEANUP=$(mktemp -d)
  local tmpdir="$TMPDIR_CLEANUP"
  trap 'rm -rf "$TMPDIR_CLEANUP"' EXIT

  if [ "$USE_APPIMAGE" = true ]; then
    install_appimage "$tmpdir"
  else
    install_deb "$tmpdir"
  fi
}

install_deb() {
  local tmpdir="$1"
  local filename="FlyMode_${VERSION}_${ARCH_DEB}.deb"
  local url="https://github.com/$REPO/releases/download/$RELEASE_TAG/$filename"

  info "Downloading $filename..."
  if ! curl -fSL --progress-bar -o "$tmpdir/$filename" "$url"; then
    warn ".deb download failed, trying AppImage..."
    install_appimage "$tmpdir"
    return
  fi

  info "Installing .deb package (requires sudo)..."
  sudo dpkg -i "$tmpdir/$filename" || {
    info "Fixing dependencies..."
    sudo apt-get install -f -y -qq
    sudo dpkg -i "$tmpdir/$filename"
  }

  ok "FlyMode installed via .deb"
}

install_appimage() {
  local tmpdir="$1"
  local filename="FlyMode_${VERSION}_${ARCH_DEB}.AppImage"
  local url="https://github.com/$REPO/releases/download/$RELEASE_TAG/$filename"

  info "Downloading $filename..."
  curl -fSL --progress-bar -o "$tmpdir/$filename" "$url" \
    || fail "Download failed: $url"

  mkdir -p "$BIN_DIR"
  cp "$tmpdir/$filename" "$BIN_DIR/flymode"
  chmod +x "$BIN_DIR/flymode"

  # Ensure ~/.local/bin is in PATH
  if ! echo "$PATH" | grep -q "$BIN_DIR"; then
    local shell_rc=""
    [ -f "$HOME/.bashrc" ] && shell_rc="$HOME/.bashrc"
    [ -f "$HOME/.zshrc" ] && shell_rc="$HOME/.zshrc"
    if [ -n "$shell_rc" ]; then
      echo 'export PATH="$HOME/.local/bin:$PATH"' >> "$shell_rc"
      warn "Added $BIN_DIR to PATH in $shell_rc — restart shell or run: source $shell_rc"
    fi
  fi

  ok "FlyMode installed as AppImage at $BIN_DIR/flymode"
}

# ── SSH server ────────────────────────────────────────────────
ensure_ssh_server() {
  if systemctl is-active --quiet sshd 2>/dev/null || systemctl is-active --quiet ssh 2>/dev/null; then
    ok "SSH server is running"
    return
  fi

  info "SSH server not running (required for P2P features)"
  read -rp "  Install and start openssh-server? [Y/n] " ans
  case "${ans:-Y}" in
    [Yy]*)
      if need_cmd apt-get; then
        sudo apt-get install -y -qq openssh-server
        sudo systemctl enable --now ssh 2>/dev/null || sudo systemctl enable --now sshd 2>/dev/null
      elif need_cmd dnf; then
        sudo dnf install -y openssh-server
        sudo systemctl enable --now sshd
      elif need_cmd pacman; then
        sudo pacman -S --noconfirm openssh
        sudo systemctl enable --now sshd
      fi
      ok "SSH server installed and started"
      ;;
    *) warn "Skipped — P2P sync/transfer will not work without SSH" ;;
  esac
}

# ── main ──────────────────────────────────────────────────────
main() {
  echo ""
  echo -e "${CYAN}╔══════════════════════════════════════╗${NC}"
  echo -e "${CYAN}║       FlyMode Quick Installer        ║${NC}"
  echo -e "${CYAN}╚══════════════════════════════════════╝${NC}"
  echo ""

  detect_platform
  fetch_release_info
  download_and_install
  ensure_ssh_server

  echo ""
  echo -e "${GREEN}══════════════════════════════════════${NC}"
  echo -e "${GREEN}  FlyMode v${VERSION} installed!${NC}"
  echo -e "${GREEN}══════════════════════════════════════${NC}"
  echo ""
  echo "  Run:  flymode"
  echo ""
  echo "  For P2P features, also install Tailscale on both machines:"
  echo "  https://tailscale.com/download"
  echo ""
}

main "$@"
