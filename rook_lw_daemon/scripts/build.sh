#!/bin/sh

set -e

cargo build --release --features "opencv libcamera"
