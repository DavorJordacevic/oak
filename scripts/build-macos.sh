#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/.."

OS=$(uname -s)
case "$OS" in
    Darwin)
        echo "Building natively on macOS..."
        cargo build --release
        cargo run --release --bin gen-man -- man
        mkdir -p dist
        case "$(uname -m)" in
            x86_64) ARCH="x86_64" ;;
            arm64) ARCH="aarch64" ;;
            *)
                echo "Unsupported macOS architecture: $(uname -m)"
                exit 1
                ;;
        esac
        STAGE_DIR=$(mktemp -d)
        trap 'rm -rf "$STAGE_DIR"' EXIT
        cp target/release/oak README.md LICENSE man/oak.1 "$STAGE_DIR/"
        ARCHIVE="dist/oak-macos-${ARCH}.tar.gz"
        tar -czf "$ARCHIVE" -C "$STAGE_DIR" oak README.md LICENSE oak.1
        echo "   -> $ARCHIVE"
        ;;
    Linux)
        echo "Cross-compiling for macOS from Linux..."
        echo "Prerequisites: cross-rs or osxcross SDK"
        echo ""
        for target in x86_64-apple-darwin aarch64-apple-darwin; do
            echo "=> $target"
            if command -v cross &>/dev/null; then
                cross build --release --target "$target"
            else
                rustup target add "$target" 2>/dev/null || true
                cargo build --release --target "$target" 2>&1 || {
                    echo "  FAILED: install cross-rs: cargo install cross"
                    echo "  Then: cross build --release --target $target"
                    continue
                }
            fi
            mkdir -p dist
            cp "target/$target/release/oak" "dist/oak-${target##*-}"
            cp README.md LICENSE dist/
            arch="$( [ "$target" = "x86_64-apple-darwin" ] && echo x86_64 || echo aarch64 )"
            tar -czf "dist/oak-macos-${arch}.tar.gz" -C dist "oak-${target##*-}" README.md LICENSE
            rm "dist/oak-${target##*-}"
            echo "   -> dist/oak-macos-${arch}.tar.gz"
        done
        ;;
    *)
        echo "This script runs on Linux or macOS only."
        exit 1
        ;;
esac
