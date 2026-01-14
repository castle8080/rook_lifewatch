#!/bin/sh

set -e

ORT_USE_PREBUILT=OFF RUSTFLAGS="-C link-args=-lstdc++" cargo build --release --features "libcamera"

cp -v scripts/start_rook_lw_daemon.sh target/release/start_rook_lw_daemon.sh
chmod +x target/release/start_rook_lw_daemon.sh
