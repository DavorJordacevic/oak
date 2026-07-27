#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PROJECT_DIR="$(dirname "$SCRIPT_DIR")"
OUTDIR="$PROJECT_DIR/dist"

rm -rf "$OUTDIR"
mkdir -p "$OUTDIR"

OS=$(uname -s)
ARCH=$(uname -m)

# -------- select native target --------
case "$OS-$ARCH" in
    Linux-x86_64)   NATIVE="x86_64-unknown-linux-gnu" ;;
    Linux-aarch64)  NATIVE="aarch64-unknown-linux-gnu" ;;
    Darwin-x86_64)  NATIVE="x86_64-apple-darwin" ;;
    Darwin-arm64)   NATIVE="aarch64-apple-darwin" ;;
    *)
        echo "Unsupported host: $OS-$ARCH"
        exit 1
        ;;
esac

build_native() {
    echo "=> building native: $NATIVE"
    cd "$PROJECT_DIR"
    cargo build --release

    local bin="oak"
    local archive
    case "$NATIVE" in
        *linux*)   archive="oak-linux-${NATIVE##*-}.tar.gz" ;;
        *darwin*)  archive="oak-macos-${NATIVE##*-}.tar.gz" ;;
    esac
    cp "target/release/$bin" "$OUTDIR/$bin"
    cp README.md LICENSE "$OUTDIR/"
    tar -czf "$OUTDIR/$archive" -C "$OUTDIR" "$bin" README.md LICENSE
    rm "$OUTDIR/$bin"
    echo "   -> dist/$archive"
}

build_target() {
    local target="$1"
    if [ "$target" = "$NATIVE" ]; then
        return
    fi

    echo "=> building cross: $target"

    cd "$PROJECT_DIR"

    if command -v cross &>/dev/null; then
        cross build --release --target "$target"
    else
        rustup target add "$target" 2>/dev/null || true
        cargo build --release --target "$target"
    fi

    local bin="oak"
    local archive
    case "$target" in
        *windows*) bin="oak.exe"; archive="oak-windows-x86_64.zip" ;;
        x86_64*linux*)  archive="oak-linux-x86_64.tar.gz" ;;
        aarch64*linux*) archive="oak-linux-aarch64.tar.gz" ;;
        x86_64*darwin*) archive="oak-macos-x86_64.tar.gz" ;;
        aarch64*darwin*) archive="oak-macos-aarch64.tar.gz" ;;
    esac

    cp "target/$target/release/$bin" "$OUTDIR/$bin"
    cp README.md LICENSE "$OUTDIR/"

    case "$target" in
        *windows*)
            (cd "$OUTDIR" && zip -q "$archive" "$bin" README.md LICENSE)
            ;;
        *)
            tar -czf "$OUTDIR/$archive" -C "$OUTDIR" "$bin" README.md LICENSE
            ;;
    esac
    rm "$OUTDIR/$bin"
    echo "   -> dist/$archive"
}

echo "== Oak build-all =="
echo "Host: $OS-$ARCH ($NATIVE)"
echo

# Always build native
build_native

# Attempt cross targets
build_target "x86_64-unknown-linux-gnu"
build_target "aarch64-unknown-linux-gnu"
build_target "x86_64-apple-darwin"
build_target "aarch64-apple-darwin"
build_target "x86_64-pc-windows-msvc"

echo
echo "Done. Builds in dist/"
ls -lh "$OUTDIR"/*.{tar.gz,zip} 2>/dev/null || true
