#!/usr/bin/env sh
set -eu

REPO="DavorJordacevic/oak"
BIN="oak"

OS=$(uname -s)
ARCH=$(uname -m)

case "$OS" in
  Linux)
    case "$ARCH" in
      x86_64|amd64) TARGET="oak-linux-x86_64.tar.gz" ;;
      aarch64|arm64) TARGET="oak-linux-aarch64.tar.gz" ;;
      *) echo "Unsupported architecture: $ARCH"; exit 1 ;;
    esac
    ;;
  Darwin)
    case "$ARCH" in
      x86_64|amd64) TARGET="oak-macos-x86_64.tar.gz" ;;
      arm64|aarch64) TARGET="oak-macos-aarch64.tar.gz" ;;
      *) echo "Unsupported architecture: $ARCH"; exit 1 ;;
    esac
    ;;
  *)
    echo "Unsupported OS: $OS"
    exit 1
    ;;
esac

echo "Fetching latest release..."
LATEST_URL=$(curl -sL "https://api.github.com/repos/$REPO/releases/latest" | sed -n 's/.*"browser_download_url": "\([^"]*'"$TARGET"'\)".*/\1/p')

if [ -z "$LATEST_URL" ]; then
  echo "Could not find download URL for $TARGET"
  exit 1
fi

TMPDIR=$(mktemp -d)
trap 'rm -rf "$TMPDIR"' EXIT

echo "Downloading $TARGET..."
curl -sL "$LATEST_URL" -o "$TMPDIR/$TARGET"

echo "Extracting..."
tar -xzf "$TMPDIR/$TARGET" -C "$TMPDIR"

if [ -d "$HOME/.local/bin" ] && echo "$PATH" | grep -q "$HOME/.local/bin"; then
  INSTALL_DIR="$HOME/.local/bin"
elif [ -d "/usr/local/bin" ] && [ -w "/usr/local/bin" ]; then
  INSTALL_DIR="/usr/local/bin"
else
  INSTALL_DIR="$HOME/.local/bin"
  mkdir -p "$INSTALL_DIR"
fi

echo "Installing to $INSTALL_DIR/$BIN..."
cp "$TMPDIR/$BIN" "$INSTALL_DIR/$BIN"
chmod +x "$INSTALL_DIR/$BIN"

echo "✓ Oak installed successfully!"
echo "Run 'oak --help' to get started."
