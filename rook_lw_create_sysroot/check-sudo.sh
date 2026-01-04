#!/bin/bash

if sudo -n true 2>/dev/null; then
    : # sudo available without password
else
    echo "Sudo access required for some operations; requesting authentication..."
    if ! sudo -v; then
        echo "Error: sudo is required but not available or authentication failed." >&2
        exit 1
    fi
fi
