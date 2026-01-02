#!/bin/bash

# Directory where this script resides
# Works when run or sourced; falls back to $0 when BASH_SOURCE is unavailable
script_dir="$(cd "$(dirname "${BASH_SOURCE[0]:-$0}")" >/dev/null 2>&1 && pwd)"

# Mount point for the root filesystem
mount_point="/mnt/rpi-rootfs"

exit_err() {
    echo "An error occurred. Exiting."
    exit 1
}

# Early check: ensure sudo is available and that the user can authenticate
# Try a non-interactive check first, then prompt for password if needed.
if sudo -n true 2>/dev/null; then
    : # sudo available without password
else
    echo "Sudo access required for some operations; requesting authentication..."
    if ! sudo -v; then
        echo "Error: sudo is required but not available or authentication failed." >&2
        exit 1
    fi
fi

# Download the image
$script_dir/download-image.sh || exit_err

# Mount the image
$script_dir/mount-image.sh || exit_err

# Make a sysroot copy
$script_dir/create-sysroot-copy.sh || exit_err

# Cleanup mounts
$script_dir/cleanup-img-mounts.sh || exit_err

# Fix up links
$script_dir/fix-symlinks.sh || exit_err

# Install development packages
$script_dir/install-dev-packages.sh || exit_err

# Fix up links for anything installed
$script_dir/fix-symlinks.sh || exit_err

# Check that the setup is correct
$script_dir/check-setup.sh || exit_err

# Create sysroot archive
$script_dir/create-sysroot-archive.sh || exit_err

echo "Setup completed successfully."