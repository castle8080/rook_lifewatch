#pragma once

#include <libcamera/camera.h>
#include <libcamera/camera_manager.h>

#include <stdexcept>
#include <string>
#include <vector>
#include <mutex>
#include <condition_variable>

namespace rook::lw_libcamera_capture {

class CameraCapturer;
class CaptureRequest;
class CameraException;

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

enum CaptureRequestStatus {
	CaptureRequestInitial,
	CaptureRequestPending,
	CaptureRequestComplete,
	CaptureRequestCancelled,
};

class CaptureRequest {
public:
	CaptureRequest(
		CameraCapturer* capturer,
		std::shared_ptr<libcamera::Request> request,
		int frame_buffer_index)
		: _request(request), _capturer(capturer), _frame_buffer_index(frame_buffer_index)
	{
	};

	~CaptureRequest();

	CaptureRequest(const CaptureRequest &) = delete;
	CaptureRequest &operator=(const CaptureRequest &) = delete;

	CaptureRequest(CaptureRequest &&) = delete;
	CaptureRequest &operator=(CaptureRequest &&) = delete;

	void set_status(CaptureRequestStatus status);

	CaptureRequestStatus get_status();

	int get_frame_buffer_index();

	void on_request_completed();

	void on_request_cancelled();

	void wait_for_completion();

private:
	CaptureRequestStatus _status = CaptureRequestInitial;

	std::shared_ptr<libcamera::Request> _request;

	CameraCapturer* _capturer = nullptr;

	int _frame_buffer_index = -1;

	std::mutex _mutex;
	
	std::condition_variable _cv;
};

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
	
	std::shared_ptr<CaptureRequest> acquire_frame();

	void start();
	void stop();

	void release_request_resources(CaptureRequest* request);

private:

	void on_request_completed(libcamera::Request *request);

	int checkout_frame_buffer_index();

	void return_frame_buffer_index(int index);

	std::mutex _mutex;

	libcamera::CameraManager _camera_manager;

	std::shared_ptr<libcamera::FrameBufferAllocator> _allocator = nullptr;

    std::shared_ptr<libcamera::Camera> _camera = nullptr;

	std::shared_ptr<libcamera::CameraConfiguration> _config = nullptr;
	bool _is_camera_started = false;

	uint64_t _next_request_sequence = 0;

	std::map<uint32_t, std::shared_ptr<CaptureRequest>> _requests;
	std::set<int> _in_use_frame_buffer_indices;
};

// Opens the first available libcamera camera and writes 10 frames to output_dir.
// Returns 0 on success, negative errno-like values on failure.
int capture10Frames(const char *output_dir);

// Populates out_ids with libcamera camera IDs.
// Returns 0 on success, negative errno-like values on failure.
int listCameras(std::vector<std::string> &out_ids);

} // namespace rook::lw_libcamera_capture
