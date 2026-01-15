#!/bin/sh
set -e

projct_dirs="
    rook_lw_libcamera_capture
    rook_lw_daemon
    rook_lw_admin
"

for dir in $projct_dirs; do
    echo "Cleaning project in directory: $dir"
    (
        cd "$dir"
        ./scripts/clean.sh
    )
done