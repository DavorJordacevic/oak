#!/usr/bin/env sh
set -eu

REPO="DavorJordacevic/oak"
BIN="oak"

OS=$(uname -s)
ARCH=$(uname -m)

# -------- detect platform --------
case "$OS" in
Linux)
    case "$ARCH" in
    x86_64)  TARGET="oak-linux-x86_64.tar.gz" ;;
    aarch64) TARGET="oak-linux-aarch64.tar.gz" ;;
    *)       echo "Unsupported Linux arch: $ARCH"; exit 1 ;;
    esac
    EXT="tar.gz"
    IS_WINDOWS=0
    ;;
Darwin)
    case "$ARCH" in
    x86_64)  TARGET="oak-macos-x86_64.tar.gz" ;;
    arm64)   TARGET="oak-macos-aarch64.tar.gz" ;;
    *)       echo "Unsupported macOS arch: $ARCH"; exit 1 ;;
    esac
    EXT="tar.gz"
    IS_WINDOWS=0
    ;;
MINGW* | MSYS* | CYGWIN*)
    case "$ARCH" in
    x86_64) TARGET="oak-windows-x86_64.zip" ;;
    *)      echo "Unsupported Windows arch: $ARCH"; exit 1 ;;
    esac
    EXT="zip"
    IS_WINDOWS=1
    BIN="oak.exe"
    ;;
*)
    echo "Unsupported OS: $OS"
    exit 1
    ;;
esac

# -------- check helper tools --------
require_cmd() {
    command -v "$1" >/dev/null 2>&1 || {
        echo "Error: '$1' is required but not found. Install it and try again."
        exit 1
    }
}
require_cmd curl

if [ "$EXT" = "zip" ]; then
    require_cmd unzip
fi

# -------- get download URL from latest GitHub release --------
echo "Fetching latest release..."
RELEASE_URL="https://api.github.com/repos/$REPO/releases/latest"

# GitHub API requires a User-Agent header
LATEST_URL=$(curl -sL -H "Accept: application/vnd.github+json" \
    -H "User-Agent: oak-installer" \
    "$RELEASE_URL" \
    | grep -o "\"browser_download_url\": *\"[^\"]*$TARGET\"" \
    | head -1 \
    | cut -d '"' -f 4)

if [ -z "$LATEST_URL" ]; then
    echo "Could not find download URL for $TARGET"
    echo "Check https://github.com/$REPO/releases/latest"
    exit 1
fi

# -------- download --------
TMPDIR=$(mktemp -d)
trap 'rm -rf "$TMPDIR"' EXIT

echo "Downloading $TARGET..."
curl -sL "$LATEST_URL" -o "$TMPDIR/$TARGET"

# -------- extract --------
echo "Extracting..."
if [ "$EXT" = "zip" ]; then
    unzip -qo "$TMPDIR/$TARGET" -d "$TMPDIR"
else
    tar -xzf "$TMPDIR/$TARGET" -C "$TMPDIR"
fi

# -------- pick install dir --------
if [ "$IS_WINDOWS" -eq 1 ]; then
    INSTALL_DIR="$HOME/bin"
    mkdir -p "$INSTALL_DIR"
else
    if [ -d "$HOME/.local/bin" ] && echo "$PATH" | tr ':' '\n' | grep -qFx "$HOME/.local/bin"; then
        INSTALL_DIR="$HOME/.local/bin"
    elif [ -d "/usr/local/bin" ] && [ -w "/usr/local/bin" ]; then
        INSTALL_DIR="/usr/local/bin"
    else
        INSTALL_DIR="$HOME/.local/bin"
        mkdir -p "$INSTALL_DIR"
    fi
fi

# -------- install --------
echo "Installing to $INSTALL_DIR/$BIN..."
cp "$TMPDIR/$BIN" "$INSTALL_DIR/$BIN"
chmod +x "$INSTALL_DIR/$BIN"

echo "=> Oak installed successfully!"

# -------- PATH reminder --------
case ":$PATH:" in
*":$INSTALL_DIR:"*) ;;
*)
    echo
    echo "Note: $INSTALL_DIR is not in your PATH."
    echo "Add it to your shell profile to run 'oak' from anywhere:"
    if [ "$IS_WINDOWS" -eq 1 ]; then
        echo "  echo 'export PATH=\"\$HOME/bin:\$PATH\"' >> ~/.bashrc"
    else
        case "${SHELL##*/}" in
        zsh)  echo "  echo 'export PATH=\"\$HOME/.local/bin:\$PATH\"' >> ~/.zshrc" ;;
        fish) echo "  fish_add_path $INSTALL_DIR" ;;
        *)    echo "  echo 'export PATH=\"$INSTALL_DIR:\$PATH\"' >> ~/.bashrc" ;;
        esac
    fi
    ;;
esac

echo
echo "Run 'oak --help' to get started."
