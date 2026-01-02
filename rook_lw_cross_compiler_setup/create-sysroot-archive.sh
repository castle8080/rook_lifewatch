#!/bin/bash

sysroots_dir="../var/sysroots"
sysroot="$(ls -1td ../var/sysroots/*/ | head -1)"
base_name="$(basename "$sysroot")"

if [ -z "$sysroot" ]; then
    echo "Error: No sysroot found in ../var/sysroots" >&2
    exit 1
fi

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