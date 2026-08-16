#!/bin/sh
# Name: KPM UI
# Author: Lukas Kreussel
# DontUseFBInk

package=/mnt/us/kmc/kpm/packages/kpm_ui
log=/mnt/us/documents/kpm-ui.log
cd "$package"
printf '[kpm-ui] launcher started at %s\n' "$(date)" > "$log"
KPM_PLATFORM=kindlehf DISPLAY=:0.0 ./launch.sh >> "$log" 2>&1
status=$?
printf '[kpm-ui] process exited with status %s\n' "$status" >> "$log"
exit "$status"
