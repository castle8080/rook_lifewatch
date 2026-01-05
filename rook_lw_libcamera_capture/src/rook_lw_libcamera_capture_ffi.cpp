#include "rook_lw_libcamera_capture/rook_lw_libcamera_capture.h"

#include "rook_lw_libcamera_capture/rook_lw_libcamera_capture.hpp"

#include <cerrno>
#include <cstdlib>
#include <cstring>
#include <new>
#include <string>
#include <vector>

using rook::lw_libcamera_capture::CameraCapturer;

struct rook_lw_camera_capturer {
	CameraCapturer impl;
};

extern "C" rook_lw_camera_capturer_t *rook_lw_camera_capturer_create(void)
{
	try {
		auto *p = new (std::nothrow) rook_lw_camera_capturer();
		if (!p)
			return nullptr;
		return p;
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
    if (!capturer)
        return;
	delete capturer;
}

extern "C" unsigned rook_lw_camera_capturer_get_camera_count(const rook_lw_camera_capturer_t *capturer)
{
	if (!capturer)
		return 0;
	return capturer->impl.cameraCount();
}

extern "C" const char *rook_lw_camera_capturer_get_camera_name(const rook_lw_camera_capturer_t *capturer,
                                                                unsigned index)
{
	if (!capturer)
		return nullptr;

	if (index >= capturer->impl.cameraCount())
		return nullptr;

    return capturer->impl.cameraName(index).c_str();
}

extern "C" int rook_lw_capture_10_frames(const char *output_dir)
{
	return rook::lw_libcamera_capture::capture10Frames(output_dir);
}
