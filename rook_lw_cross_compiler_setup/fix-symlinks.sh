#!/bin/bash

# This script fixes absolute symlinks in the mounted Raspberry Pi OS root filesystem
# by converting them to relative symlinks.

# Mount point for the root filesystem
mount_point="/mnt/rpi-rootfs"

# Fix absolute symlinks in the mounted filesystem
echo "Fixing absolute symlinks in $mount_point..."
pushd "$mount_point" > /dev/null || exit 1

sudo find . -type l | while read l; do
  target=$(readlink "$l")
  if [[ "$target" == /* ]]; then
    sudo ln -snf ".$target" "$l"
  fi
done

popd > /dev/null