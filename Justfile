set shell := ["sh", "-eu", "-c"]

default:
    @just --list

build:
    cargo build --locked

# Build and run the app locally with GTK 2 and sample package data
desktop:
    @sh scripts/with-desktop-env.sh sh scripts/run-desktop.sh

run: desktop

test:
    cargo test --lib --locked

lint:
    cargo fmt -- --check
    cargo clippy --lib -- -D warnings
    cargo clippy --bin kpm-ui -- -D warnings
    for script in package/*.sh package/scriptlets/*.sh scripts/*.sh tools/*.sh; do sh -n "$script"; done

check: lint test

# Build and stage the Kindle binary, entering the included dev shell if needed
kindle:
    @sh scripts/with-kindle-env.sh just _kindle

# Build the distributable KPM package
package:
    @sh scripts/with-kindle-env.sh just _package

# Build and upload everything needed for a first-time MTP installation
install:
    @sh scripts/with-kindle-env.sh just _install

# Build and upload a replacement package over MTP
update:
    @sh scripts/with-kindle-env.sh just _update

# Upload a Scriptlet that removes KPM UI from the Kindle
uninstall:
    @sh scripts/deploy-mtp.sh uninstall

[private]
_kindle:
    sh scripts/build-kindle.sh

[private]
_package: _kindle
    python tools/package.py package dist

[private]
_install: _package
    sh scripts/deploy-mtp.sh install

[private]
_update: _package
    sh scripts/deploy-mtp.sh update

clean:
    cargo clean
    rm -rf package/bin dist
