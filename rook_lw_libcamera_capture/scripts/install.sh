#!/bin/sh
set -e

mkdir -p ../dist
cmake --install build/default --prefix "$PWD/../dist"