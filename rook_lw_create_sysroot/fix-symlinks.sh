#!/bin/bash

# This script fixes absolute symlinks in the mounted Raspberry Pi OS root filesystem
# by converting them to relative symlinks.

sysroots_dir="../var/sysroots"
sysroot="$(ls -1td ../var/sysroots/*/ | head -1)"
if [ -z "$sysroot" ]; then
    echo "Error: No sysroot found in ../var/sysroots" >&2
    exit 1
fi

# Fix absolute symlinks in the mounted filesystem
echo "Fixing absolute symlinks in $sysroot..."
pushd "$sysroot" > /dev/null || exit 1

sudo find . -type l | while read l; do
  target=$(readlink "$l")
  if [[ "$target" == /* ]]; then
    sudo ln -snf ".$target" "$l"
  fi
done

popd > /dev/null