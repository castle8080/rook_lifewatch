# rook_lw_libcamera_capture

Small C++ shared library for capturing frames with **libcamera** on Raspberry Pi.

## Requirements

### Host dev
- `cmake` (>= 3.23)
- `ninja` (`sudo apt-get install -y ninja-build`)
- A C++17 compiler

### Raspberry Pi / sysroot
- libcamera development files present in your sysroot (e.g. `libcamera.pc`)
- Cross compiler (typical Debian tooling):
  - `sudo apt-get install -y g++-aarch64-linux-gnu pkg-config`

Your sysroot is expected at:
- `../var/sysroots/2025-12-04-raspios-trixie-arm64`

Alternatively, you can set the sysroot path via an environment variable:

```bash
export ROOK_PI_SYSROOT=/absolute/path/to/your/sysroot
```

## Build (host)

To build the code.

```bash
sudo apt-get update
sudo apt-get install -y \
  build-essential cmake ninja-build pkg-config \
  libcamera-dev
```

```bash
cmake --preset default
cmake --build --preset default
```

## Install (to a custom directory)

This project defines CMake `install()` rules for the library and headers.

To install into `../dist` (relative to the project root):

```bash
cmake --preset default
cmake --build --preset default
cmake --install build/default --prefix "$PWD/../dist"
```

Notes:
- You can use any absolute path for `--prefix`.
- If you prefer to bake the prefix into the build directory’s cache:

```bash
cmake --preset default -DCMAKE_INSTALL_PREFIX="$PWD/../dist"
cmake --build --preset default
cmake --install build/default
```

## Build (cross for Raspberry Pi arm64)

```bash

# Sets ROOK_PI_SYSROOT and PKG_CONFIG_* for the newest sysroot under ../var/sysroots
. ./setup-cross-compiler-env.sh

cmake --preset rpi-arm64
cmake --build --preset rpi-arm64
```

To install the cross build artifacts into `../dist` as well:

```bash
cmake --install build/rpi-arm64 --prefix "$PWD/../dist"
```

## Run on the Pi

Copy the `camera_capturer_test` binary + library to the Pi (or package/install), then:

```bash
./camera_capturer_test /tmp/frames
```

It writes 10 raw frame dumps as `frame_000.raw` ... `frame_009.raw`.

## API

The library exposes a minimal C ABI in:
- `include/rook_lw_libcamera_capture/rook_lw_libcamera_capture.h`

