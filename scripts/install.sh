#!/bin/sh

set -e

projct_dirs="
    rook_lw_libcamera_capture
    rook_lw_daemon
"

for dir in $projct_dirs; do
    echo "Installing project in directory: $dir"
    (
        cd "$dir"
        ./scripts/install.sh
    )
done