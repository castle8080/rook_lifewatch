#!/bin/sh

set -e

projct_dirs="
    rook_lw_libcamera_capture
    rook_lw_daemon
"

for dir in $projct_dirs; do
    echo "Building project in directory: $dir"
    (
        cd "$dir"
        ./scripts/build.sh
        ./scripts/install.sh
    )
done

./scripts/download-model-files.sh
