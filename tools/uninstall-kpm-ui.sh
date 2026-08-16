#!/bin/sh
# Name: Uninstall KPM UI
# Author: Lukas Kreussel

installed=${KPM_UI_INSTALLED_DIR:-/mnt/us/kmc/kpm/packages/kpm_ui}
kpm=${KPM_UI_KPM:-/var/local/kmc/bin/kpm}
archive=${KPM_UI_ARCHIVE:-/mnt/us/documents/kpm-ui.kpkg}
installer=${KPM_UI_INSTALLER:-/mnt/us/documents/Install KPM UI.sh}
install_error=${KPM_UI_INSTALL_ERROR_FILE:-/mnt/us/documents/kpm-ui-install-error.txt}
error_file=${KPM_UI_ERROR_FILE:-/mnt/us/documents/kpm-ui-uninstall-error.txt}
uninstaller=${KPM_UI_UNINSTALLER:-$0}
status=0

if [ -d "$installed" ]; then
    echo "Uninstalling KPM UI..."
    "$kpm" -y uninstall kpm_ui || status=$?
else
    echo "KPM UI is not installed."
fi

if [ "$status" -eq 0 ]; then
    echo "KPM UI was uninstalled successfully."
    rm -f "$archive" "$installer" "$install_error" "$error_file" "$uninstaller"
else
    echo "KPM UI uninstallation failed with status $status."
    echo "The uninstaller was kept so the operation can be retried."
    printf 'KPM UI uninstallation failed with status %s. Run Uninstall KPM UI again to retry.\n' \
        "$status" > "$error_file"
fi

if command -v lipc-set-prop >/dev/null 2>&1; then
    lipc-set-prop com.lab126.scanner doFullScan 1
fi
exit "$status"
