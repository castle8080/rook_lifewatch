#!/bin/bash

set -euo pipefail

# Directory where this script resides
script_dir="$(cd "$(dirname "${BASH_SOURCE[0]:-$0}")" >/dev/null 2>&1 && pwd)"

# Make sure we have sudo access
$script_dir/check-sudo.sh

# Download the sysroot from the Raspberry Pi
$script_dir/download-sysroot-from-pi.sh

# Rename and move the downloaded sysroot to the final location
$script_dir/rename-move-tmp-pibuild.sh

# Fix up links
$script_dir/fix-symlinks.sh

# Check that the setup is correct
$script_dir/check-setup.sh

# Create sysroot archive
$script_dir/create-sysroot-archive.sh

echo "Setup completed successfully."