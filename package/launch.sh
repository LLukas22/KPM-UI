#!/bin/sh

set -eu

if [ "${KPM_PLATFORM:-}" != "kindlehf" ]; then
    echo "KPM UI requires a Kindle running firmware 5.16.3 or newer." >&2
    exit 1
fi

export DISPLAY="${DISPLAY:-:0.0}"
exec ./bin/kpm-ui
