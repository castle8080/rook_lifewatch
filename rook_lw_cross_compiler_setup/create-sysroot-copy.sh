#!/bin/bash

# Directory where sysroots will be stored
sysroots_dir="../var/sysroots"

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

# Remove the .img extension to get the base name
base_name="${image_file%.*}"

echo "Creating sysroot copy from $mount_point to $sysroots_dir/$base_name"
mkdir -p "$sysroots_dir/$base_name"

if ! sudo rsync -a \
  --delete \
  "$mount_point/" \
  "$sysroots_dir/$base_name/"; then
    echo "Error: failed to copy sysroot from $mount_point to $sysroots_dir/$base_name" >&2
    exit 1
fi

echo "Sysroot copied to $sysroots_dir/$base_name"

pushd "$sysroots_dir" > /dev/null || exit 1

echo "Creating sysroot archive: ${base_name}-sysroot.tar.zst"
sudo tar --numeric-owner --zstd -cpf \
  "${base_name}-sysroot.tar.zst" \
  "$base_name"

if [ $? -ne 0 ]; then
    echo "Error: failed to create sysroot archive ${base_name}-sysroot.tar.zst" >&2
    popd > /dev/null
    exit 1
fi
popd > /dev/null

echo "Sysroot archive created at $sysroots_dir/${base_name}-sysroot.tar.zst"