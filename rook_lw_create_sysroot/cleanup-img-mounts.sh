#!/bin/bash

# Make sure the mount point is unmounted
if grep -q '/mnt/rpi-rootfs' /proc/mounts; then
    echo "Unmounting /mnt/rpi-rootfs..."
    sudo umount /mnt/rpi-rootfs
else
    echo "/mnt/rpi-rootfs is not mounted."
fi

# Detach any loop devices associated with the Raspberry Pi images
/usr/sbin/losetup -a | grep rpi | cut -d: -f1 | while read -r loopdev; do
    echo "Detaching loop device $loopdev..."
    sudo /usr/sbin/losetup -d "$loopdev"
done