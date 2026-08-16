#!/bin/sh

set -eu

target=arm-unknown-linux-gnueabihf
binary=target/$target/release/kpm-ui
cargo_command=cargo

if command -v cargo-kindle >/dev/null 2>&1; then
    cargo_command=cargo-kindle
fi

if ! command -v arm-kindlehf-linux-gnueabihf-gcc >/dev/null 2>&1; then
    echo "The kindlehf koxtoolchain is not on PATH." >&2
    exit 1
fi

"$cargo_command" build --locked --release --target "$target"

mkdir -p package/bin
cp "$binary" package/bin/kpm-ui
chmod 755 package/bin/kpm-ui package/install.sh package/launch.sh package/uninstall.sh package/scriptlets/kpm_ui.sh

echo "Staged package/bin/kpm-ui"
