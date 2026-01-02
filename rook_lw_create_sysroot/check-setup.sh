#!/bin/bash

# Check if the development packages are installed in the sysroot.
# Get the latest sysroot directory.
export SYSROOT="$(ls -1td ../var/sysroots/*/ | head -1)"

if [ -z "$SYSROOT" ]; then
    echo "Error: No sysroot found in ../var/sysroots" >&2
    exit 1
fi

export PKG_CONFIG_SYSROOT_DIR=$SYSROOT
export PKG_CONFIG_PATH=$SYSROOT/usr/lib/pkgconfig:$SYSROOT/usr/lib/aarch64-linux-gnu/pkgconfig:$SYSROOT/usr/share/pkgconfig

if ! pkg-config --cflags libcamera; then
    echo "Error: libcamera-dev not found in sysroot pkg-config path." >&2
    exit 1
fi

if ! pkg-config --cflags opencv4; then
    echo "Error: libopencv-dev not found in sysroot pkg-config path." >&2
    exit 1
fi