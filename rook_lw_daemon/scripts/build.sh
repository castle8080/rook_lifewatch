#!/bin/sh

set -e

cargo build --release --features "opencv libcamera"

cp -v scripts/start_rook_lw_daemon.sh target/release/start_rook_lw_daemon.sh
chmod +x target/release/start_rook_lw_daemon.sh
