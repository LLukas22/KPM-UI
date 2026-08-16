#!/bin/sh

set -eu

if command -v cargo >/dev/null 2>&1 && command -v pkg-config >/dev/null 2>&1 && pkg-config --exists gtk+-2.0; then
    exec "$@"
fi

if ! command -v nix >/dev/null 2>&1; then
    echo "GTK 2 development libraries are unavailable and Nix is not installed." >&2
    echo "Install Nix or add GTK 2 and pkg-config to the local development environment." >&2
    exit 1
fi

echo "Starting the included desktop development environment..."
exec nix develop path:.#desktop --command "$@"
