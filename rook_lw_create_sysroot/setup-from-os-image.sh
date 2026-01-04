#!/bin/bash

set -euo pipefail

# Directory where this script resides
# Works when run or sourced; falls back to $0 when BASH_SOURCE is unavailable
script_dir="$(cd "$(dirname "${BASH_SOURCE[0]:-$0}")" >/dev/null 2>&1 && pwd)"

# Check for sudo access.
$script_dir/check-sudo.sh

# Download the image
$script_dir/download-image.sh

# Mount the image
$script_dir/mount-image.sh

# Make a sysroot copy
$script_dir/create-sysroot-copy.sh

# Cleanup mounts
$script_dir/cleanup-img-mounts.sh

# Fix up links
$script_dir/fix-symlinks.sh

# Install development packages
$script_dir/install-dev-packages.sh

# Fix up links for anything installed
$script_dir/fix-symlinks.sh

# Check that the setup is correct
$script_dir/check-setup.sh

# Create sysroot archive
$script_dir/create-sysroot-archive.sh

echo "Setup completed successfully."