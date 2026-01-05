#!/bin/sh

set -e

mkdir -p ../dist/bin
cp -v target/release/rook_lw_daemon ../dist/bin/rook_lw_daemon
