#!/bin/bash

set -euo pipefail

exit 1

sysroots_dir="../var/sysroots"
sysroot_tmp_dir="../var/tmp/pibuild"

ssh_base="${1:-}"

if [ -z "$ssh_base" ]; then
	read -r -p "Enter ssh_base (user@host): " ssh_base
	ssh_base="${ssh_base//[[:space:]]/}"
fi

if [ -z "$ssh_base" ]; then
	echo "Error: ssh_base is required (format: user@host)" >&2
    exit 1
fi

# Setup a fresh temp dir
sudo rm -rf "$sysroot_tmp_dir"
mkdir -p "$sysroot_tmp_dir"

echo "Copying sysroot from Raspberry Pi at $ssh_base..."

# Copy the files over.
if ! sudo rsync -aH -x -z \
	--numeric-ids \
	--delete \
	--info=progress2 \
	--partial \
	--rsync-path="sudo rsync" \
	-e "ssh" \
	--exclude=/dev \
	--exclude=/dev/* \
	--exclude=/proc \
	--exclude=/proc/* \
	--exclude=/sys \
	--exclude=/sys/* \
	--exclude=/run \
	--exclude=/run/* \
	--exclude=/tmp \
	--exclude=/tmp/* \
	--exclude=/var/run \
	--exclude=/var/run/* \
	--exclude=/var/tmp \
	--exclude=/var/tmp/* \
	--exclude=/mnt \
	--exclude=/mnt/* \
	--exclude=/media \
	--exclude=/media/* \
    --exclude=/home \
    --exclude=/home/* \
	"$ssh_base:/" \
	"$sysroot_tmp_dir/"; then
	echo "Error: rsync from Raspberry Pi failed" >&2
    exit 1
fi