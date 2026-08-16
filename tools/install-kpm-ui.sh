#!/bin/sh
# Name: Install or Update KPM UI
# Author: Lukas Kreussel

documents=${KPM_UI_DOCUMENTS_DIR:-/mnt/us/documents}
package=${KPM_UI_ARCHIVE:-$documents/kpm-ui.kpkg}
installed=${KPM_UI_INSTALLED_DIR:-/mnt/us/kmc/kpm/packages/kpm_ui}
kpm=${KPM_UI_KPM:-/var/local/kmc/bin/kpm}
error_file=${KPM_UI_ERROR_FILE:-$documents/kpm-ui-install-error.txt}
status=0

if [ -z "${KPM_UI_ARCHIVE:-}" ] && [ ! -f "$package" ]; then
    set -- "$documents"/kpm_ui_*_kindlehf.kpkg
    if [ "$#" -eq 1 ] && [ -f "$1" ]; then
        package=$1
    fi
fi

if [ ! -f "$package" ]; then
    echo "KPM UI package not found in the documents folder."
    echo "Keep exactly one downloaded KPM UI .kpkg beside this Scriptlet."
    status=1
elif [ -d "$installed" ]; then
    echo "Removing the installed KPM UI development build..."
    "$kpm" -y uninstall kpm_ui || status=$?
fi

if [ "$status" -eq 0 ]; then
    echo "Installing the KPM UI package..."
    "$kpm" -y install "file://$package" || status=$?
fi

if [ "$status" -eq 0 ]; then
    echo "KPM UI was installed successfully."
    rm -f "$package" "$error_file" "$0"
else
    echo "KPM UI installation failed with status $status."
    echo "The package and installer were kept so the operation can be retried."
    printf 'KPM UI installation failed with status %s. Run Install or Update KPM UI again to retry.\n' \
        "$status" > "$error_file"
fi

if command -v lipc-set-prop >/dev/null 2>&1; then
    lipc-set-prop com.lab126.scanner doFullScan 1
fi
exit "$status"
