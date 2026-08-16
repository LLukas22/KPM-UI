#!/bin/sh

set -eu

absolute_path() {
    case "$1" in
        /*) printf '%s\n' "$1" ;;
        *) printf '%s/%s\n' "$(pwd)" "$1" ;;
    esac
}

state=${KPM_UI_DESKTOP_STATE:-target/desktop-state}
packages=${KPM_UI_PACKAGES_DIR:-$state/packages}
mkdir -p "$state" "$packages"

if [ -z "${KPM_UI_KPM:-}" ]; then
    if [ ! -f KPM/meson.build ]; then
        echo "Initializing the KPM submodule..."
        git submodule update --init KPM
    fi

    build=target/desktop-kpm
    database=$(absolute_path "$state/kpm.db")
    package_root=$(absolute_path "$packages")
    configuration="$database|$package_root|$(git -C KPM rev-parse HEAD)"
    configured=
    if [ -f "$build/kpm-ui-configuration" ]; then
        configured=$(sed -n '1p' "$build/kpm-ui-configuration")
    fi
    if [ "$configured" != "$configuration" ]; then
        configure_kpm() {
            meson setup "$build" KPM \
                --buildtype=release \
                -Ddb_path="$database" \
                -Dpkg_path="$package_root" \
                -Dkindle_platform=kindlehf \
                "$@"
        }
        if [ -f "$build/build.ninja" ]; then
            configure_kpm --reconfigure
        else
            configure_kpm
        fi
        printf '%s\n' "$configuration" > "$build/kpm-ui-configuration"
    fi
    meson compile -C "$build" cli/kpm:executable

    state_root=$(absolute_path "$state")
    sandbox=$state_root/root
    shims=$sandbox/shims
    mkdir -p \
        "$sandbox/home" \
        "$sandbox/mnt/us/documents" \
        "$sandbox/mnt/us/extensions" \
        "$sandbox/opt" \
        "$sandbox/var/local" \
        "$sandbox/var/tmp" \
        "$shims"
    cp scripts/desktop-kindle-noop.sh "$shims/kindle-noop"
    chmod 755 "$shims/kindle-noop"
    for command in eips eips_print_bottom_centered initctl lipc-get-prop lipc-set-prop mntroot restart start status stop; do
        ln -sf kindle-noop "$shims/$command"
    done
    sqlite3 "$sandbox/var/local/appreg.db" < scripts/desktop-appreg.sql

    KPM_UI_REAL_KPM=$(absolute_path "$build/cli/kpm")
    KPM_UI_DB=$database
    KPM_UI_SANDBOX_ROOT=$sandbox
    KPM_UI_DESKTOP_STATE_ROOT=$state_root
    KPM_UI_KPM=$(absolute_path scripts/desktop-kpm.sh)
    export KPM_UI_REAL_KPM KPM_UI_SANDBOX_ROOT KPM_UI_DESKTOP_STATE_ROOT KPM_UI_KPM KPM_UI_DB

    if [ ! -f "$state/index-initialized" ]; then
        echo "Initializing the desktop KPM package index..."
        "$KPM_UI_KPM" update
        touch "$state/index-initialized"
    fi
fi

export KPM_UI_KPM
export KPM_UI_PACKAGES_DIR="$packages"
exec cargo run --locked
