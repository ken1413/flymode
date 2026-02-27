#!/usr/bin/env bash
# FlyMode — one-line installer (private repo, requires gh auth)
# curl -fsSL https://gist.githubusercontent.com/ken1413/756e1cd8131583561c138a33cc401984/raw/setup.sh | bash
set -euo pipefail

REPO="ken1413/flymode"
INSTALL_DIR="$HOME/app/flymode"
BIN_DIR="$HOME/.local/bin"

# ── helpers ────────────────────────────────────────────────────
RED='\033[0;31m'; GREEN='\033[0;32m'; YELLOW='\033[1;33m'; CYAN='\033[0;36m'; NC='\033[0m'

info()  { echo -e "${CYAN}[INFO]${NC}  $*"; }
ok()    { echo -e "${GREEN}[OK]${NC}    $*"; }
warn()  { echo -e "${YELLOW}[WARN]${NC}  $*"; }
fail()  { echo -e "${RED}[FAIL]${NC}  $*"; exit 1; }

need_cmd() {
    if ! command -v "$1" &>/dev/null; then
        return 1
    fi
    return 0
}

# ── detect OS ──────────────────────────────────────────────────
detect_os() {
    case "$(uname -s)" in
        Linux*)  OS="linux" ;;
        Darwin*) OS="macos" ;;
        *)       fail "Unsupported OS: $(uname -s). Only Linux and macOS are supported." ;;
    esac

    if [ "$OS" = "linux" ]; then
        if need_cmd apt-get; then
            PKG_MGR="apt"
        elif need_cmd dnf; then
            PKG_MGR="dnf"
        elif need_cmd pacman; then
            PKG_MGR="pacman"
        else
            fail "No supported package manager found (apt/dnf/pacman)."
        fi
    fi
    ok "Detected: $OS"
}

# ── system dependencies ───────────────────────────────────────
install_system_deps() {
    info "Installing system dependencies..."

    if [ "$OS" = "linux" ]; then
        case "$PKG_MGR" in
            apt)
                sudo apt-get update -qq
                sudo apt-get install -y -qq \
                    build-essential curl wget git pkg-config \
                    libgtk-3-dev libwebkit2gtk-4.1-dev \
                    libappindicator3-dev librsvg2-dev patchelf \
                    libssl-dev libsoup-3.0-dev libjavascriptcoregtk-4.1-dev
                ;;
            dnf)
                sudo dnf install -y \
                    gcc gcc-c++ make curl wget git pkg-config \
                    gtk3-devel webkit2gtk4.1-devel \
                    libappindicator-gtk3-devel librsvg2-devel \
                    openssl-devel libsoup3-devel javascriptcoregtk4.1-devel
                ;;
            pacman)
                sudo pacman -Syu --noconfirm --needed \
                    base-devel curl wget git pkg-config \
                    gtk3 webkit2gtk-4.1 \
                    libappindicator-gtk3 librsvg patchelf \
                    openssl libsoup3
                ;;
        esac
    elif [ "$OS" = "macos" ]; then
        if ! need_cmd brew; then
            info "Installing Homebrew..."
            /bin/bash -c "$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)"
        fi
        # Xcode command line tools (provides clang, git, etc.)
        xcode-select --install 2>/dev/null || true
    fi

    ok "System dependencies installed"
}

# ── Rust ───────────────────────────────────────────────────────
install_rust() {
    if need_cmd rustc && need_cmd cargo; then
        ok "Rust already installed ($(rustc --version))"
    else
        info "Installing Rust..."
        curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --default-toolchain stable
        # shellcheck source=/dev/null
        source "$HOME/.cargo/env"
        ok "Rust installed ($(rustc --version))"
    fi
}

# ── Node.js ────────────────────────────────────────────────────
install_node() {
    if need_cmd node && need_cmd npm; then
        local node_major
        node_major=$(node -v | sed 's/v\([0-9]*\).*/\1/')
        if [ "$node_major" -ge 18 ]; then
            ok "Node.js already installed ($(node -v))"
            return
        fi
        warn "Node.js $(node -v) is too old, need >= 18"
    fi

    info "Installing Node.js 22 LTS..."
    if [ "$OS" = "linux" ]; then
        curl -fsSL https://deb.nodesource.com/setup_22.x | sudo -E bash -
        sudo apt-get install -y -qq nodejs 2>/dev/null || \
        sudo dnf install -y nodejs 2>/dev/null || \
        sudo pacman -S --noconfirm nodejs npm 2>/dev/null
    elif [ "$OS" = "macos" ]; then
        brew install node@22
    fi
    ok "Node.js installed ($(node -v))"
}

# ── GitHub CLI ─────────────────────────────────────────────────
install_gh() {
    if need_cmd gh; then
        ok "GitHub CLI already installed"
    else
        info "Installing GitHub CLI..."
        if [ "$OS" = "linux" ]; then
            case "$PKG_MGR" in
                apt)
                    sudo mkdir -p -m 755 /etc/apt/keyrings
                    curl -fsSL https://cli.github.com/packages/githubcli-archive-keyring.gpg | sudo tee /etc/apt/keyrings/githubcli-archive-keyring.gpg > /dev/null
                    echo "deb [arch=$(dpkg --print-architecture) signed-by=/etc/apt/keyrings/githubcli-archive-keyring.gpg] https://cli.github.com/packages stable main" | sudo tee /etc/apt/sources.list.d/github-cli.list > /dev/null
                    sudo apt-get update -qq && sudo apt-get install -y -qq gh
                    ;;
                dnf) sudo dnf install -y gh ;;
                pacman) sudo pacman -S --noconfirm github-cli ;;
            esac
        elif [ "$OS" = "macos" ]; then
            brew install gh
        fi
        ok "GitHub CLI installed"
    fi

    # Check auth
    if ! gh auth status &>/dev/null; then
        warn "GitHub CLI not authenticated. Please log in now:"
        gh auth login
    fi
    ok "GitHub CLI authenticated"
}

# ── Tauri CLI ──────────────────────────────────────────────────
install_tauri_cli() {
    if need_cmd cargo-tauri; then
        ok "Tauri CLI already installed"
    else
        info "Installing Tauri CLI (this takes a few minutes)..."
        cargo install tauri-cli
        ok "Tauri CLI installed"
    fi
}

# ── clone repo ─────────────────────────────────────────────────
clone_repo() {
    if [ -d "$INSTALL_DIR/.git" ]; then
        info "Updating existing repo..."
        git -C "$INSTALL_DIR" pull --ff-only
    elif [ -d "$INSTALL_DIR" ]; then
        info "Directory exists but is not a git repo, cloning fresh..."
        rm -rf "$INSTALL_DIR"
        mkdir -p "$(dirname "$INSTALL_DIR")"
        gh repo clone "$REPO" "$INSTALL_DIR"
    else
        info "Cloning FlyMode..."
        mkdir -p "$(dirname "$INSTALL_DIR")"
        gh repo clone "$REPO" "$INSTALL_DIR"
    fi
    ok "Source ready at $INSTALL_DIR"
}

# ── build ──────────────────────────────────────────────────────
build_app() {
    cd "$INSTALL_DIR"

    info "Installing frontend dependencies..."
    cd src-ui && npm install --silent && cd ..

    info "Building FlyMode (release)... this may take a while"
    cargo tauri build 2>&1 | tail -5

    ok "Build complete"
}

# ── install binary ─────────────────────────────────────────────
install_binary() {
    mkdir -p "$BIN_DIR"

    local binary=""
    if [ "$OS" = "linux" ]; then
        binary="$INSTALL_DIR/target/release/flymode"
    elif [ "$OS" = "macos" ]; then
        binary="$INSTALL_DIR/target/release/flymode"
    fi

    if [ ! -f "$binary" ]; then
        fail "Binary not found at $binary"
    fi

    cp "$binary" "$BIN_DIR/flymode"
    chmod +x "$BIN_DIR/flymode"

    # Ensure ~/.local/bin is in PATH
    if ! echo "$PATH" | grep -q "$BIN_DIR"; then
        local shell_rc=""
        if [ -f "$HOME/.bashrc" ]; then
            shell_rc="$HOME/.bashrc"
        elif [ -f "$HOME/.zshrc" ]; then
            shell_rc="$HOME/.zshrc"
        fi
        if [ -n "$shell_rc" ]; then
            echo 'export PATH="$HOME/.local/bin:$PATH"' >> "$shell_rc"
            warn "Added $BIN_DIR to PATH in $shell_rc — restart shell or run: source $shell_rc"
        fi
    fi

    ok "Installed to $BIN_DIR/flymode"
}

# ── desktop entry (Linux) ─────────────────────────────────────
install_desktop_entry() {
    if [ "$OS" != "linux" ]; then return; fi

    local desktop_dir="$HOME/.local/share/applications"
    mkdir -p "$desktop_dir"

    cat > "$desktop_dir/flymode.desktop" <<DESKTOP
[Desktop Entry]
Name=FlyMode
Comment=Wireless scheduler + P2P sync + Notes
Exec=$BIN_DIR/flymode
Icon=$INSTALL_DIR/src-tauri/icons/icon.png
Terminal=false
Type=Application
Categories=Utility;Network;
StartupWMClass=flymode
DESKTOP

    ok "Desktop entry created — search 'FlyMode' in app launcher"
}

# ── main ───────────────────────────────────────────────────────
main() {
    echo ""
    echo -e "${CYAN}╔══════════════════════════════════════╗${NC}"
    echo -e "${CYAN}║        FlyMode Installer v0.2        ║${NC}"
    echo -e "${CYAN}╚══════════════════════════════════════╝${NC}"
    echo ""

    detect_os
    install_system_deps
    install_rust
    install_node
    install_gh
    clone_repo
    install_tauri_cli
    build_app
    install_binary
    install_desktop_entry

    echo ""
    echo -e "${GREEN}══════════════════════════════════════${NC}"
    echo -e "${GREEN}  FlyMode installed successfully!${NC}"
    echo -e "${GREEN}══════════════════════════════════════${NC}"
    echo ""
    echo "  Run:  flymode"
    echo "  Dir:  $INSTALL_DIR"
    echo ""
}

main "$@"
