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

// TODO: Go fix naming conventions.

class CameraCapturer {
public:
	CameraCapturer();
	~CameraCapturer();

	CameraCapturer(const CameraCapturer &) = delete;
	CameraCapturer &operator=(const CameraCapturer &) = delete;

	CameraCapturer(CameraCapturer &&) = delete;
	CameraCapturer &operator=(CameraCapturer &&) = delete;

	unsigned camera_count() const;

	const std::string* camera_name(unsigned index) const;

	void reset_camera();
    void set_camera_source(const std::string &camera_name);
	
	void acquire_frame();

	void start();
	void stop();

private:

	void on_request_completed(libcamera::Request *request);

	libcamera::CameraManager _camera_manager;
	std::shared_ptr<libcamera::FrameBufferAllocator> _allocator = nullptr;
    std::shared_ptr<libcamera::Camera> _camera = nullptr;
	std::shared_ptr<libcamera::CameraConfiguration> _config = nullptr;
	bool _is_camera_started = false;

	std::vector<std::shared_ptr<libcamera::Request>> _requests;
	uint64_t _next_request_sequence = 0;
};

// Opens the first available libcamera camera and writes 10 frames to output_dir.
// Returns 0 on success, negative errno-like values on failure.
int capture10Frames(const char *output_dir);

// Populates out_ids with libcamera camera IDs.
// Returns 0 on success, negative errno-like values on failure.
int listCameras(std::vector<std::string> &out_ids);

} // namespace rook::lw_libcamera_capture
