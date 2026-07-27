#!/usr/bin/env bash
set -euo pipefail

cd "$(dirname "$0")/.."

case "$(uname -m)" in
    x86_64|amd64) ARCH="x86_64" ;;
    aarch64|arm64) ARCH="aarch64" ;;
    *)
        echo "Unsupported Linux architecture: $(uname -m)"
        exit 1
        ;;
esac

mkdir -p dist man

echo "Building Oak for Linux ($ARCH)..."
cargo build --release
cargo run --release --bin gen-man -- man

STAGE_DIR=$(mktemp -d)
trap 'rm -rf "$STAGE_DIR"' EXIT
cp target/release/oak README.md LICENSE man/oak.1 "$STAGE_DIR/"

ARCHIVE="dist/oak-linux-${ARCH}.tar.gz"
tar -czf "$ARCHIVE" -C "$STAGE_DIR" oak README.md LICENSE oak.1

echo "=> $ARCHIVE"
