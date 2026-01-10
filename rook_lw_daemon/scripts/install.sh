#!/bin/sh

set -e

mkdir -p ../dist/bin
cp -v target/release/rook_lw_daemon ../dist/bin/rook_lw_daemon
cp -v target/release/start_rook_lw_daemon.sh ../dist/bin/start_rook_lw_daemon.sh
chmod +x ../dist/bin/start_rook_lw_daemon.sh
