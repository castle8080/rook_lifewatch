# Rook LifeWatch

Hobby project: trail-cam-like software for a Raspberry Pi.

The goal is to capture frames from the Pi camera, detect motion, and (eventually) record/retain the interesting bits in a predictable, debuggable way.

## Repository layout

- `rook_lw_daemon/` — Main Rust daemon (motion detection / objection detection / local storage / pipeline orchestration).
- `rook_lw_libcamera_capture/` — C++ capture helper/library built on `libcamera`.
- `rook_lw_models` - Shared rust data types models used between system components.
- `rook_lw_image_repo` - Shared rust repository accessors for access data from `rook_lw_daemon` and `rook_lw_admin`.
- `rook_lw_admin` - Admin app using Actix to server APIs and content over http from the Pi.
- `rook_lw_admin_fe` - Leptos (wasm) front end web appliation served from `rook_lw_admin`.
- `scripts` - Scripts to build and some utility scripts that get installed for running.
- `rook_lw_create_sysroot` - This had scripts to help build a sysroot for cross compilation - it isn't working though. (The build of the sysroot works, but compiling on a Linux desktop was having issues linking correct to run on Arm.)

## Status

Early-stage / experimental.

## Quick start

### Build the daemon (native)

From project root:
  - Build and install to dist: `./scripts/build-install.py`

Run the camera motion watcher (`rook_lw_daemon`):
    - dist/bin/start_rook_lw_daemon.sh

Run the admin app (`rook_lw_admin`):
    - dist/bin/start_rook_lw_admin.sh

## License

Apache License 2.0. See [LICENSE](LICENSE).
