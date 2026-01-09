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

	std::string get_pixel_format();

private:
	friend class CaptureRequest;

	void release_request_resources(CaptureRequest* request);

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


enum CaptureRequestStatus {
	CaptureRequestInitial,
	CaptureRequestPending,
	CaptureRequestComplete,
	CaptureRequestCancelled,
};

class CaptureRequestMappedPlane {
public:
	CaptureRequestMappedPlane(const libcamera::FrameBuffer::Plane &plane);
	~CaptureRequestMappedPlane();

	size_t get_length();

	void* get_data();

private:
	void* _data = nullptr;
	size_t _length = 0;
};

class CaptureRequest {
public:
	CaptureRequest(
		CameraCapturer* capturer,
		std::shared_ptr<libcamera::Request> request,
		int frame_buffer_index);

	~CaptureRequest();

	CaptureRequest(const CaptureRequest &) = delete;
	CaptureRequest &operator=(const CaptureRequest &) = delete;

	CaptureRequest(CaptureRequest &&) = delete;
	CaptureRequest &operator=(CaptureRequest &&) = delete;

	CaptureRequestStatus get_status();

	void wait_for_completion();

	int get_plane_count();

	CaptureRequestMappedPlane* get_mapped_plane(int plane_index);

private:
	friend class CameraCapturer;

	void on_request_completed();

	void on_request_cancelled();

	void on_request_pending();

	int get_frame_buffer_index();

	void clear_mapped_planes();

	CameraCapturer* _capturer;

	CaptureRequestStatus _status = CaptureRequestInitial;

	std::shared_ptr<libcamera::Request> _request;

	std::mutex _mutex;
	
	std::condition_variable _cv;

	int _frame_buffer_index = -1;

	libcamera::FrameBuffer *_frame_buffer = nullptr;

	std::vector<CaptureRequestMappedPlane*> _mapped_planes;
};

} // namespace rook::lw_libcamera_capture
