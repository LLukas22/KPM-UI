#!/bin/sh

set -eu

action=${1:-install}
case "$action" in
    install) description="installation" ;;
    update) description="update" ;;
    uninstall) description="uninstallation" ;;
    *)
        echo "Usage: $0 [install|update|uninstall]" >&2
        exit 2
        ;;
esac

if [ "$action" != uninstall ]; then
    version=$(python -c 'import json; print(".".join(map(str, json.load(open("package/manifest.json"))["version"])))')
    artifact="dist/kpm_ui_${version}_kindlehf.kpkg"
    if [ ! -f "$artifact" ]; then
        echo "Package not found: $artifact" >&2
        exit 1
    fi
fi

if command -v gio >/dev/null 2>&1; then
    run_gio() {
        gio "$@"
    }
elif command -v nix >/dev/null 2>&1; then
    run_gio() {
        nix develop path:.#kindle --command gio "$@"
    }
else
    echo "MTP deployment requires GIO or Nix." >&2
    exit 1
fi

if command -v lsusb >/dev/null 2>&1; then
    run_lsusb() {
        lsusb "$@"
    }
elif command -v nix >/dev/null 2>&1; then
    run_lsusb() {
        nix develop path:.#kindle --command lsusb "$@"
    }
else
    echo "MTP deployment requires lsusb or Nix." >&2
    exit 1
fi

usb_device=$(run_lsusb -d 1949: | sed -n '1{s/^Bus \([0-9][0-9]*\) Device \([0-9][0-9]*\):.*/\1,\2/p;}')
if [ -z "$usb_device" ]; then
    echo "No Kindle was found. Connect it over USB and unlock the screen." >&2
    exit 1
fi

mtp_root="mtp://[usb:${usb_device}]/"
run_gio mount "$mtp_root" >/dev/null 2>&1 || true
storage=$(run_gio list "$mtp_root" | sed -n '1p')
if [ -z "$storage" ]; then
    echo "The Kindle storage could not be opened. Reconnect it and unlock the screen." >&2
    exit 1
fi

storage_path=$(python -c 'import sys, urllib.parse; print(urllib.parse.quote(sys.argv[1].rstrip("/")))' "$storage")
documents="${mtp_root}${storage_path}/documents"

upload() {
    source=$1
    destination=$2
    run_gio remove "$destination" >/dev/null 2>&1 || true
    run_gio copy -T "$source" "$destination"

    local_size=$(wc -c < "$source" | tr -d ' ')
    remote_size=$(run_gio info -a standard::size "$destination" | sed -n 's/^  standard::size: //p')
    if [ "$remote_size" != "$local_size" ]; then
        echo "Upload verification failed for $source (local $local_size, remote ${remote_size:-unknown})." >&2
        exit 1
    fi
}

echo "Uploading KPM UI $description Scriptlet..."
if [ "$action" = uninstall ]; then
    upload tools/uninstall-kpm-ui.sh "$documents/Uninstall%20KPM%20UI.sh"
    scriptlet="Uninstall KPM UI"
else
    upload "$artifact" "$documents/kpm-ui.kpkg"
    upload tools/install-kpm-ui.sh "$documents/Install%20KPM%20UI.sh"
    scriptlet="Install or Update KPM UI"
fi

echo
echo "Upload complete."
echo "Disconnect USB, open the Kindle library, and tap '$scriptlet'."
