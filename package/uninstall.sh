#!/bin/sh

set -eu

scriptlet=/mnt/us/documents/kpm_ui.sh
if [ -f "$scriptlet" ] && cmp -s "$scriptlet" ./scriptlets/kpm_ui.sh; then
    rm "$scriptlet"
fi
