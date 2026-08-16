#!/bin/sh

set -eu

if command -v arm-kindlehf-linux-gnueabihf-gcc >/dev/null 2>&1; then
    exec "$@"
fi

if ! command -v nix >/dev/null 2>&1; then
    echo "The Kindle toolchain is unavailable and Nix is not installed." >&2
    echo "Install Nix or add arm-kindlehf-linux-gnueabihf-gcc to PATH." >&2
    exit 1
fi

echo "Starting the included Kindle development environment..."
exec nix develop path:.#kindle --command "$@"
