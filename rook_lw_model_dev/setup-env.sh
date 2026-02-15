#!/bin/bash
set -e

echo "=== Rook LifeWatch Model Development Environment Setup ==="

# Check if uv is installed
if ! command -v uv &> /dev/null; then
    echo "uv not found. Installing uv..."
    curl -LsSf https://astral.sh/uv/install.sh | sh
    
    # Source the shell configuration to get uv in PATH
    if [ -f "$HOME/.cargo/env" ]; then
        source "$HOME/.cargo/env"
    fi
    
    # Verify installation
    if ! command -v uv &> /dev/null; then
        echo "ERROR: uv installation failed or not in PATH"
        echo "Please restart your shell or run: source ~/.cargo/env"
        exit 1
    fi
    
    echo "✓ uv installed successfully"
else
    echo "✓ uv is already installed"
fi

# Deactivate current virtual environment if active
if [ -n "$VIRTUAL_ENV" ]; then
    echo "Deactivating current virtual environment: $VIRTUAL_ENV"
    deactivate 2>/dev/null || true
fi

# Create virtual environment if it doesn't exist
if [ -d ".venv" ]; then
    echo "✓ Virtual environment already exists at .venv"
else
    echo "Creating virtual environment..."
    uv venv
    echo "✓ Virtual environment created at .venv"
fi

# Activate virtual environment
echo "Activating virtual environment..."
source .venv/bin/activate

# Install dependencies
echo "Installing dependencies from requirements.txt..."
uv pip install -r requirements.txt

echo ""
echo "=== Setup Complete! ==="
echo ""
echo "Virtual environment is now active."
echo "To activate it in future sessions, run:"
echo "  source .venv/bin/activate"
echo ""
echo "To start JupyterLab, run:"
echo "  jupyter lab"
