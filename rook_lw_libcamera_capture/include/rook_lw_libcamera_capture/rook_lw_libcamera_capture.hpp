#pragma once

#include <libcamera/camera.h>
#include <libcamera/camera_manager.h>

#include <stdexcept>
#include <string>
#include <vector>

namespace rook::lw_libcamera_capture {

// C++ core API (non-FFI).
//
// These APIs are intended for C++ callers. The C FFI wrapper lives in
// rook_lw_libcamera_capture.h and forwards into this core.

class CameraException : public std::runtime_error {
public:
	explicit CameraException(const std::string &message, int code = 0)
		: std::runtime_error(message), _code(code)
	{
	}

	int code() const noexcept { return _code; }

private:
	int _code;
};

class CameraCapturer {
public:
	CameraCapturer();
	~CameraCapturer();

	CameraCapturer(const CameraCapturer &) = delete;
	CameraCapturer &operator=(const CameraCapturer &) = delete;

	CameraCapturer(CameraCapturer &&) = delete;
	CameraCapturer &operator=(CameraCapturer &&) = delete;

	unsigned cameraCount() const;

	std::string cameraName(unsigned index) const;

    void setCameraSource(const std::string &camera_name);

private:
	libcamera::CameraManager _cameraManager;
    libcamera::Camera *_camera = nullptr;
};

// Opens the first available libcamera camera and writes 10 frames to output_dir.
// Returns 0 on success, negative errno-like values on failure.
int capture10Frames(const char *output_dir);

// Populates out_ids with libcamera camera IDs.
// Returns 0 on success, negative errno-like values on failure.
int listCameras(std::vector<std::string> &out_ids);

} // namespace rook::lw_libcamera_capture
