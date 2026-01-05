use std::env;
use std::path::PathBuf;

fn main() {
    // Only link the native libcamera capture library when the feature is enabled.
    if env::var_os("CARGO_FEATURE_LIBCAMERA").is_none() {
        return;
    }

    // Link against system libcamera via pkg-config.
    // This supplies the correct -L and -l flags for the distro.
    let libcamera_probe = pkg_config::Config::new()
        .cargo_metadata(true)
        .probe("libcamera");

    if let Err(err) = libcamera_probe {
        panic!(
            "Failed to find system libcamera via pkg-config: {err}\n\
Install the system development package (often `libcamera-dev`) and `pkg-config`, then re-run cargo."
        );
    }

    // Some distros split symbols into libcamera-base; probe if available.
    // If it's not present, ignore (libcamera.pc may already include it).
    let _ = pkg_config::Config::new()
        .cargo_metadata(true)
        .probe("libcamera-base");

    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR not set"));
    let lib_dir = manifest_dir.join("../dist/lib");
    let archive = lib_dir.join("librook_lw_libcamera_capture.a");

    println!("cargo:rerun-if-env-changed=CARGO_FEATURE_LIBCAMERA");
    println!("cargo:rerun-if-changed={}", archive.display());

    if !archive.exists() {
        panic!(
            "libcamera feature enabled but native archive not found: {} (expected ../dist/lib/librook_lw_libcamera_capture.a relative to the crate)",
            archive.display()
        );
    }

    // Link search path and library name. Rust/Cargo will add `lib` prefix and `.a` suffix.
    println!("cargo:rustc-link-search=native={}", lib_dir.display());
    println!("cargo:rustc-link-lib=static=rook_lw_libcamera_capture");

    // Many libcamera capture implementations are C++.
    // If the archive is pure C this is harmless; if it's C++ it avoids missing stdc++.
    if cfg!(target_os = "linux") {
        println!("cargo:rustc-link-lib=dylib=stdc++");
    }
}
