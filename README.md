# Rook LifeWatch

Hobby project: trail-cam-like software for a Raspberry Pi.

The goal is to capture frames from the Pi camera, detect motion, and (eventually) record/retain the interesting bits in a predictable, debuggable way.

## Repository layout

- `rook_lw_daemon/` — Main Rust daemon (motion detection / pipeline orchestration).
- `rook_lw_libcamera_capture/` — C++ capture helper/library built on `libcamera`.
- `rook_lw_create_sysroot/` — Scripts to build/manage ARM64 sysroots for cross-compiling.
- `var/` — Local artifacts (images, sysroots, prompts). Not intended for releases.
- `doc/` — Notes and design docs.

## Status

Early-stage / experimental.

## Quick start

### Build the daemon (native)

From `rook_lw_daemon/`:

- Debug build: `cargo build`
- Run: `cargo run`

### Features

The daemon uses Cargo features to select implementations:

- `libcamera` — enable the libcamera-based capture implementation
- `opencv` — enable OpenCV-based motion detection / processing helpers

Example:

- `cargo run --features opencv`

(See `rook_lw_daemon/Cargo.toml` and `rook_lw_daemon/README.md` for the current feature matrix.)

### Build the libcamera capture helper

From `rook_lw_libcamera_capture/`:

- Configure + build (default preset): `cmake --preset default && cmake --build --preset default`

For cross-compilation and sysroot setup, see `rook_lw_create_sysroot/README.md`.

## License

Apache License 2.0. See [LICENSE](LICENSE).
