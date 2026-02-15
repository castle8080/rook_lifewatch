#!/bin/bash
set -e

# Get the directory where this script is located
SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"

# Change to the script directory
cd "$SCRIPT_DIR"

echo "Working directory: $SCRIPT_DIR"
echo ""

# Source the setup script to ensure environment is ready
source ./setup-env.sh

echo ""
echo "Starting JupyterLab..."
echo ""

# Start JupyterLab
jupyter lab
