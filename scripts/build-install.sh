#!/bin/sh

set -e

# Running build and install for each sub-project
projct_dirs="
    rook_lw_libcamera_capture
    rook_lw_daemon
    rook_lw_admin
"

for dir in $projct_dirs; do
    echo "Building project in directory: $dir"
    (
        cd "$dir"
        ./scripts/build.sh
        ./scripts/install.sh
    )
done

# Copy scripts needed for running
cp -vf scripts/run_daemon.py dist/bin
cp -vf scripts/start*.sh dist/bin/

# Download model and 3rd party libs.
./scripts/download-model-files.sh
./scripts/install-onnxruntime.sh
