#/bin/bash

# This mounts the Raspberry Pi OS image's root filesystem.

# Mount point for the root filesystem
mount_point="/mnt/rpi-rootfs"

# Where the image files will be stored.
image_file_dir="../var/rpi_images"

# Find the image file in the image directory
image_file="$(ls -1 "$image_file_dir" | grep '\.img$' | grep 'raspios' | head -n 1)"

# Path to losetup command
losetup_path="$(which losetup || echo "/usr/sbin/losetup")"

if [ -z "$image_file" ]; then
    echo "Error: No Raspberry Pi OS image file found in $image_file_dir" >&2
    exit 1
fi

# Set up loop device
loop_device="$($losetup_path -j "$image_file_dir/$image_file" | cut -d: -f1)"
if [ -z "$loop_device" ]; then
    sudo $losetup_path -fP "$image_file_dir/$image_file"
fi
loop_device="$($losetup_path -j "$image_file_dir/$image_file" | cut -d: -f1)"
if [ -z "$loop_device" ]; then
    echo "Error: failed to set up loop device for $image_file_dir/$image_file" >&2
    exit 1
fi
echo "Loop device for $image_file: $loop_device"

# Create mount point if it isn't already a mountpoint
if ! mountpoint -q "$mount_point"; then
    sudo mkdir -p "$mount_point"

    # The root filesystem is usually on the second partition
    loop_mount_device="${loop_device}p2"

    if [ ! -b "$loop_mount_device" ]; then
        echo "Error: expected partition device $loop_mount_device does not exist." >&2
        exit 1
    fi

    echo "Mounting $loop_mount_device to $mount_point..."
    sudo mount "$loop_mount_device" "$mount_point"
fi