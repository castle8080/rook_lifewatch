#include "rook_lw_libcamera_capture/rook_lw_libcamera_capture.h"

#include "rook_lw_libcamera_capture/rook_lw_libcamera_capture.hpp"

#include <cerrno>
#include <cstdlib>
#include <cstring>
#include <iostream>
#include <string>
#include <vector>

using rook::lw_libcamera_capture::CameraCapturer;

// === FFI boundary rules (C / Rust callers) ===
//
// This translation unit implements the C ABI wrapper around the C++ core.
// It exists so higher-level languages (notably Rust) can call into the library.
//
// IMPORTANT: C++ exceptions MUST NOT cross an FFI boundary.
//
// Letting a C++ exception propagate through an `extern "C"` function is undefined
// behavior and can terminate the process in surprising ways (especially once Rust
// unwinding/abort settings and different standard libraries are involved).
//
// Therefore every exported `extern "C"` function in this file must:
// - catch `rook::lw_libcamera_capture::CameraException` and translate it into a
//   C-friendly return value (null pointer, 0, or a negative error code)
// - catch `...` as a last resort and translate similarly
// - never throw, even on allocation failure
//
// If you add new FFI exports here, follow the same pattern.
//
// Memory/lifetime notes:
// - Any `const char*` returned to C/Rust must point to storage owned by the
//   capturer (or some other stable storage) and must not reference a temporary.
//
// Logging policy:
// - We currently log exceptions to stderr for diagnostics. Rust should treat any
//   non-success return as the error signal; stderr is best-effort.

struct rook_lw_camera_capturer {
	CameraCapturer impl;
    std::vector<std::string> camera_names;
};

static bool rook_lw_refresh_camera_names(rook_lw_camera_capturer &capturer)
{
    capturer.camera_names.clear();

    const unsigned count = capturer.impl.cameraCount();
    capturer.camera_names.reserve(count);
    for (unsigned i = 0; i < count; ++i) {
        capturer.camera_names.push_back(capturer.impl.cameraName(i));
    }

    return true;
}

extern "C" rook_lw_camera_capturer_t *rook_lw_camera_capturer_create(void)
{
	try {
        auto *capturer = new rook_lw_camera_capturer();
        rook_lw_refresh_camera_names(*capturer);
        return capturer;
	}
    catch (const rook::lw_libcamera_capture::CameraException &) {
        std::cerr << "CameraException caught in rook_lw_camera_capturer_create" << std::endl;
        return nullptr;
    }
    catch (...) {
        std::cerr << "Unknown exception caught in rook_lw_camera_capturer_create" << std::endl;
		return nullptr;
	}
}

extern "C" void rook_lw_camera_capturer_destroy(rook_lw_camera_capturer_t *capturer)
{
	if (capturer == nullptr) {
		return;
	}
	delete capturer;
}

extern "C" unsigned rook_lw_camera_capturer_get_camera_count(const rook_lw_camera_capturer_t *capturer)
{
	if (capturer == nullptr) {
		return 0;
    }
    try {
        // Expose the cached list size so returned name pointers remain stable.
        return static_cast<unsigned>(capturer->camera_names.size());
    }
    catch (const rook::lw_libcamera_capture::CameraException &) {
        std::cerr << "CameraException caught in rook_lw_camera_capturer_get_camera_count" << std::endl;
        return 0;
    }
    catch (...) {
        std::cerr << "Unknown exception caught in rook_lw_camera_capturer_get_camera_count" << std::endl;
        return 0;
    }
}

extern "C" const char *rook_lw_camera_capturer_get_camera_name(const rook_lw_camera_capturer_t *capturer,
                                                                unsigned index)
{
	if (!capturer) {
		return nullptr;
    }

    try {
        if (index >= capturer->camera_names.size()) {
            return nullptr;
        }
        return capturer->camera_names[index].c_str();
    }
    catch (const rook::lw_libcamera_capture::CameraException &) {
        std::cerr << "CameraException caught in rook_lw_camera_capturer_get_camera_name" << std::endl;
        return nullptr;
    }
    catch (...) {
        std::cerr << "Unknown exception caught in rook_lw_camera_capturer_get_camera_name" << std::endl;
        return nullptr;
    }
}

extern "C" int rook_lw_capture_10_frames(const char *output_dir)
{
    try {
        return rook::lw_libcamera_capture::capture10Frames(output_dir);
    }
    catch (const rook::lw_libcamera_capture::CameraException &e) {
        std::cerr << "CameraException caught in rook_lw_capture_10_frames: " << e.what() << std::endl;
        return (e.code() < 0) ? e.code() : -EIO;
    }
    catch (...) {
        std::cerr << "Unknown exception caught in rook_lw_capture_10_frames" << std::endl;
        return -EIO;
    }
}
