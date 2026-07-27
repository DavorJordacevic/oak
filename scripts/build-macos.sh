#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/.."

OS=$(uname -s)
case "$OS" in
    Darwin)
        echo "Building natively on macOS..."
        cargo build --release
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
