#!/bin/bash

# Installs development packages inside the mounted Raspberry Pi OS root filesystem.


# Mount point for the root filesystem
mount_point="/mnt/rpi-rootfs"

cleanup() {
    echo "Cleaning up mounted filesystems..."
    sudo umount -l "$mount_point/proc" 2>/dev/null || true
    sudo umount -l "$mount_point/sys" 2>/dev/null || true
    sudo umount -l "$mount_point/dev" 2>/dev/null || true
}

exit_err() {
    echo "An error occurred. Exiting."
    cleanup
    exit 1
}

# Ensure qemu-aarch64-static is available on the host
if [ ! -f "/usr/bin/qemu-aarch64-static" ]; then
    echo "Installing qemu-user-static and binfmt-support on host..."
    sudo apt install -y qemu-user-static binfmt-support
fi

if [ ! -f "/usr/bin/qemu-aarch64-static" ]; then
    echo "Error: qemu-aarch64-static is not installed on the host." >&2
    exit 1
fi

echo "Copying qemu-aarch64-static into $mount_point"
sudo cp /usr/bin/qemu-aarch64-static "$mount_point/usr/bin/" || exit_err

echo "Mounting /dev, /sys, /proc into $mount_point"
sudo mount --bind /dev "$mount_point/dev" || exit_err
sudo mount --bind /sys "$mount_point/sys" || exit_err
sudo mount --bind /proc "$mount_point/proc" || exit_err

echo "Entering chroot to install packages"
sudo chroot "$mount_point" /bin/bash -c '
set -euo pipefail
export DEBIAN_FRONTEND=noninteractive
apt update
apt install -y libcamera-dev libopencv-dev pkg-config cmake build-essential
apt clean
' || exit_err

echo "Chroot package installation finished"

cleanup

echo "Done."