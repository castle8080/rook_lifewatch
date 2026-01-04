# Source this script to set up the cross-compilation environment for Rook Pi

sysroots_dir="$(realpath -m -- "$(pwd)/../var/sysroots")"

ROOK_PI_SYSROOT="$(ls -1td "$sysroots_dir"/*/ | head -n 1)"
if [ -z "$ROOK_PI_SYSROOT" ]; then
    echo "No sysroot found in $sysroots_dir"
    return 1
fi

echo "ROOK_PI_SYSROOT: $ROOK_PI_SYSROOT"

export PKG_CONFIG_SYSROOT_DIR="$ROOK_PI_SYSROOT"
export PKG_CONFIG_DIR=
export PKG_CONFIG_LIBDIR="$ROOK_PI_SYSROOT/usr/lib/aarch64-linux-gnu/pkgconfig:$ROOK_PI_SYSROOT/usr/lib/pkgconfig:$ROOK_PI_SYSROOT/usr/share/pkgconfig"
