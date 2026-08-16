#!/bin/sh

set -eu

: "${KPM_UI_REAL_KPM:?KPM_UI_REAL_KPM is not set}"
: "${KPM_UI_SANDBOX_ROOT:?KPM_UI_SANDBOX_ROOT is not set}"
: "${KPM_UI_DESKTOP_STATE_ROOT:?KPM_UI_DESKTOP_STATE_ROOT is not set}"

root=$KPM_UI_SANDBOX_ROOT
exec bwrap \
    --ro-bind / / \
    --bind "$KPM_UI_DESKTOP_STATE_ROOT" "$KPM_UI_DESKTOP_STATE_ROOT" \
    --dev /dev \
    --proc /proc \
    --tmpfs /tmp \
    --bind "$root/mnt" /mnt \
    --bind "$root/opt" /opt \
    --bind "$root/var" /var \
    --setenv HOME "$root/home" \
    --setenv PATH "$root/shims:$PATH" \
    --cap-drop ALL \
    --unshare-pid \
    --new-session \
    --die-with-parent \
    --chdir "$(pwd)" \
    "$KPM_UI_REAL_KPM" "$@"
