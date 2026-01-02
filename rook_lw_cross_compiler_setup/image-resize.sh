#!/bin/bash

set -euo pipefail

# Mount point for the root filesystem
mount_point="/mnt/rpi-rootfs"

# Where the image files will be stored.
image_file_dir="../var/rpi_images"

# Find the image file in the image directory
image_file="$(ls -1 "$image_file_dir" | grep '\.img$' | grep 'raspios' | head -n 1)"

if [ -z "$image_file" ]; then
    echo "Error: No Raspberry Pi OS image file found in $image_file_dir" >&2
    exit 1
fi

# Have a temporary image file for resizing
tmp_image_file="$image_file.tmp"

grow_by="+1G"
part_num=2

img_size_bytes=$(stat -c%s "$image_file_dir/$image_file")
img_size_bytes_original="$(cat "$image_file_dir/$image_file.size")"

echo "Resizing image: $image_file_dir/$image_file"
echo "Current size: $img_size_bytes bytes"
echo "Original size recorded: $img_size_bytes_original bytes"

if [ "$img_size_bytes" -le "$img_size_bytes_original" ]; then
    echo "Image size matches original size; proceeding with resize."
else
    echo "Image size is larger than original size; resize may have already been done."
    exit 1
fi

echo "Image: $image_file_dir/$image_file"
echo "Grow by: $grow_by"
echo "Partition to resize: $part_num"

cp "$image_file_dir/$image_file" "$image_file_dir/$tmp_image_file"

# Helper to find loop device for this image
find_loop() {
    /usr/sbin/losetup -j "$image_file_dir/$tmp_image_file" | cut -d: -f1 || true
}

# Unmount any mountpoint inside the image if present
mounts=$(mount | awk -v img="$image_file_dir/$tmp_image_file" '$0 ~ img {print $3}') || true
if [ -n "$mounts" ]; then
    echo "Unmounting mounts inside image:"
    echo "$mounts"
    while read -r m; do
        sudo umount -l "$m" || true
    done <<< "$mounts"
fi

echo "Growing image file by $grow_by"
sudo truncate -s "$grow_by" "$image_file_dir/$tmp_image_file"

echo "Attaching image with partitions"
sudo losetup -Pf "$image_file_dir/$tmp_image_file"
loop_dev="$(find_loop)"
if [ -z "$loop_dev" ]; then
    echo "Error: failed to attach loop device" >&2
    exit 1
fi
echo "Loop device: $loop_dev"

part_dev="${loop_dev}p${part_num}"
if [ ! -b "$part_dev" ]; then
    echo "Error: expected partition device $part_dev does not exist." >&2
    echo "Run 'lsblk' to inspect: sudo lsblk -o NAME,SIZE,MOUNTPOINT" >&2
    sudo losetup -d "$loop_dev" || true
    exit 1
fi

echo "Expanding partition $part_num on $loop_dev"

# read start sector of the partition
start_sector=$(sudo parted -s "$loop_dev" unit s print | awk "\$1==${part_num} {print \$2}" | tr -d 's')
if [ -z "$start_sector" ]; then
    # fallback: parse by matching a line that begins with the partition number
    start_sector=$(sudo parted -s "$loop_dev" unit s print | awk '/^\s*'"$part_num"'/ {print $2}' | tr -d 's')
fi
if [ -z "$start_sector" ]; then
    echo "Failed to determine start sector of partition $part_num" >&2
    sudo losetup -d "$loop_dev" || true
    exit 1
fi
echo "Partition $part_num start sector: $start_sector"

# Recreate partition with same start and end at 100%
sudo parted -s "$loop_dev" rm "$part_num"
sudo parted -s --align optimal "$loop_dev" mkpart primary "${start_sector}s" 100%
sudo partprobe "$loop_dev" || true
# re-attach to refresh partition nodes
sudo losetup -d "$loop_dev" || true
sudo losetup -Pf "$image_file_dir/$tmp_image_file"
loop_dev="$(find_loop)"
part_dev="${loop_dev}p${part_num}"


echo "Running filesystem check and resize on $part_dev"
sudo e2fsck -f -y "$part_dev" || true
sudo resize2fs "$part_dev"

echo "Done. New device: $part_dev"

echo "Cleaning up: detaching loop device"
sudo losetup -d "$loop_dev" || true

mv "$image_file_dir/$tmp_image_file" "$image_file_dir/$image_file"

echo "Success"
