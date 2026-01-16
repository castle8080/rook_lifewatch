#!/bin/sh
set -e

dirname=$(dirname "$0")

# Run the add-dev-deps script to ensure all dev dependencies are installed
$dirname/add-dev-deps.sh

# Run the development server using trunk
trunk serve