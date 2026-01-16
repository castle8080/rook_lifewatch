#!/bin/sh
set -e

mkdir -p ../dist/bin
cp -vf target/release/rook_lw_admin ../dist/bin/rook_lw_admin

if [ -d www ]; then
    mkdir -p ../dist/www
    cp -rvf www/ ../dist/
fi
