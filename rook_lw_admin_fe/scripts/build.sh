#!/bin/sh
set -e

dirname=$(dirname "$0")

# Run the add-dev-deps script to ensure all dev dependencies are installed
$dirname/add-dev-deps.sh

# Build the project using trunk
trunk build --release --public-url /admin/