#!/bin/bash

# This downloads a Raspberry Pi OS image and decompresses it.

# Mount point for the root filesystem
mount_point="/mnt/rpi-rootfs"

# Where the image files will be stored.
image_file_dir="../var/rpi_images"

image_url="https://downloads.raspberrypi.com/raspios_arm64/images/raspios_arm64-2025-12-04/2025-12-04-raspios-trixie-arm64.img.xz"

# Make sure the image directory exists.
mkdir -p "$image_file_dir"

# Derive filename from URL (strip query/fragment, take basename)
# Also get file without compression extension.
image_path="${image_url%%[?#]*}"
image_file_compressed="${image_path##*/}"
image_file="${image_file_compressed%.*}"

# Download and decompress the image if not already present
if [ ! -f "$image_file_dir/$image_file" ]; then
    if [ ! -f "$image_file_dir/$image_file_compressed" ]; then
        tmp="$image_file_dir/${image_file_compressed}.part"
        echo "Downloading $image_file_compressed..."
        if ! curl -fL --retry 3 -C - -o "$tmp" "$image_url"; then
            echo "Error: download failed for $image_url" >&2
            rm -f "$tmp"
            exit 1
        fi
        mv "$tmp" "$image_file_dir/$image_file_compressed"
    fi
    echo "Decompressing $image_file_compressed..."
    if ! xz -d "$image_file_dir/$image_file_compressed"; then
        echo "Error: decompression of $image_file_compressed failed." >&2
        exit 1
    fi
    # Record the size (bytes) of the decompressed image into a .size file
    image_fullpath="$image_file_dir/$image_file"
    if [ -f "$image_fullpath" ]; then
        size_bytes=$(stat -c%s "$image_fullpath")
        echo "$size_bytes" > "$image_fullpath.size"
        echo "Recorded image size: $size_bytes bytes -> $image_fullpath.size"
    fi
fi