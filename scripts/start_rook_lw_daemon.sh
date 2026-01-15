#!/bin/sh
set -e

script_dir="$(dirname "$0")"
exec "$script_dir/run_daemon.py" "rook_lw_daemon" "$@"