#!/bin/bash

# Installs development packages inside the mounted Raspberry Pi OS root filesystem.

sysroots_dir="../var/sysroots"
sysroot="$(ls -1td ../var/sysroots/*/ | head -1)"
if [ -z "$sysroot" ]; then
    echo "Error: No sysroot found in ../var/sysroots" >&2
    exit 1
fi

cleanup() {
    echo "Cleaning up mounted filesystems..."
    sudo umount -l "$sysroot/proc" 2>/dev/null || true
    sudo umount -l "$sysroot/sys" 2>/dev/null || true
    sudo umount -l "$sysroot/dev" 2>/dev/null || true
}

exit_err() {
    echo "An error occurred. Exiting."
    cleanup
    exit 1
}

echo "Preparing to install development packages in sysroot: $sysroot"

if [ ! -f "/usr/bin/qemu-aarch64-static" ]; then
    echo "Error: qemu-aarch64-static is not installed on the host." >&2
    exit 1
fi

echo "Copying qemu-aarch64-static into $sysroot"
sudo cp /usr/bin/qemu-aarch64-static "$sysroot/usr/bin/" || exit_err

echo "Mounting /dev, /sys, /proc into $sysroot"
sudo mkdir -p "$sysroot/dev" "$sysroot/sys" "$sysroot/proc" "$sysroot/tmp" || exit_err
sudo mount --bind /dev "$sysroot/dev" || exit_err
sudo mount --bind /sys "$sysroot/sys" || exit_err
sudo mount --bind /proc "$sysroot/proc" || exit_err

echo "Entering chroot to install packages"
sudo chroot "$sysroot" /bin/bash -c '
set -euo pipefail
export DEBIAN_FRONTEND=noninteractive
apt update
apt install -y libcamera-dev libopencv-dev pkg-config cmake build-essential
apt clean
' || exit_err

echo "Chroot package installation finished"

cleanup

echo "Done."