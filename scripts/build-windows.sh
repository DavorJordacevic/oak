#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/.."

echo "Building Oak for Windows"

target="x86_64-pc-windows-msvc"
archive="oak-windows-x86_64.zip"
mkdir -p dist

if [ "$(uname -s)" = "Windows" ] || [ -n "${MSYSTEM:-}" ]; then
    echo "=> Building natively on Windows..."
    cargo build --release
    cp target/release/oak.exe dist/
else
    echo "=> Cross-compiling from $(uname -s)..."
    if command -v cross &>/dev/null; then
        cross build --release --target "$target"
    else
        echo "Trying with cargo directly..."
        rustup target add "$target" 2>/dev/null || true
        cargo build --release --target "$target" 2>&1 || {
            echo
            echo "FAILED. You need a Windows cross-compiler."
            echo "Install cross-rs:  cargo install cross"
            echo "Then: cross build --release --target $target"
            echo
            echo "Or install MSVC toolchain:"
            echo "  Linux: apt install mingw-w64"
            echo "  Then:  rustup target add x86_64-pc-windows-gnu"
            echo "         cargo build --release --target x86_64-pc-windows-gnu"
            exit 1
        }
    fi
    cp "target/$target/release/oak.exe" dist/
fi

cp README.md LICENSE dist/ 2>/dev/null || true
(cd dist && zip -q "$archive" oak.exe README.md LICENSE)

echo "=> dist/$archive"
ls -lh "dist/$archive"
