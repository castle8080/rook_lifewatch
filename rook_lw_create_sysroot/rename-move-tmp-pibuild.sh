#!/bin/bash

set -euo pipefail

sysroots_dir="../var/sysroots"
sysroot_tmp_dir="../var/tmp/pibuild"

sanitize_token() {
	# Keep only characters safe for path names.
	# shellcheck disable=SC2001
	echo "$1" | tr '[:upper:]' '[:lower:]' | sed -E 's/[^a-z0-9._-]+/-/g; s/^-+//; s/-+$//'
}

# Get the sysroot name from the downloaded sysroot files
id="unknown"
version_id="unknown"
codename="unknown"
debian_version=""

if [ -f "$sysroot_tmp_dir/etc/os-release" ]; then
    # shellcheck disable=SC1090
    . "$sysroot_tmp_dir/etc/os-release" || true
    id="${ID:-unknown}"
    version_id="${VERSION_ID:-unknown}"
    codename="${VERSION_CODENAME:-${VERSION:-unknown}}"
fi

if [ -f "$sysroot_tmp_dir/etc/debian_version" ]; then
    debian_version="$(tr -d '\n' < "$sysroot_tmp_dir/etc/debian_version" || true)"
fi

arch="unknown"
if [ -d "$sysroot_tmp_dir/usr/lib/aarch64-linux-gnu" ]; then
    arch="arm64"
elif [ -d "$sysroot_tmp_dir/usr/lib/arm-linux-gnueabihf" ]; then
    arch="armhf"
fi

id_s="$(sanitize_token "$id")"
ver_s="$(sanitize_token "$version_id")"
code_s="$(sanitize_token "$codename")"
dt_timestamp="$(date -u +%Y-%m-%d)"
if [ -n "$debian_version" ]; then
    deb_s="$(sanitize_token "$debian_version")"
    sysroot_name="${dt_timestamp}-${id_s}-${code_s}-${deb_s}-${arch}"
else
    sysroot_name="${dt_timestamp}-${id_s}-${code_s}-${ver_s}-${arch}"
fi

echo "Renaming and moving sysroot to $sysroots_dir/$sysroot_name..."
sudo mv "$sysroot_tmp_dir" "$sysroots_dir/$sysroot_name"
sudo touch "$sysroots_dir/$sysroot_name"