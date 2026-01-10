#include "rook_lw_libcamera_capture/rook_lw_libcamera_capture.hpp"

#include <cerrno>
#include <condition_variable>
#include <chrono>
#include <cstdio>
#include <cstring>
#include <mutex>
#include <string>
#include <vector>

#include <libcamera/camera.h>
#include <libcamera/camera_manager.h>
#include <libcamera/formats.h>
#include <libcamera/framebuffer_allocator.h>
#include <libcamera/request.h>
#include <libcamera/stream.h>

namespace rook::lw_libcamera_capture {

CaptureRequest::CaptureRequest(
	CameraCapturer* capturer,
	std::shared_ptr<libcamera::Request> request,
	int frame_buffer_index)
	: _capturer(capturer), _request(request), _frame_buffer_index(frame_buffer_index)
{
}

CaptureRequest::~CaptureRequest()
{
	std::lock_guard<std::mutex> lock(_mutex);
	_frame_buffer = nullptr;

	if (_capturer) {
		_capturer->release_request_resources(this);
	}
	clear_mapped_planes();
}

CaptureRequestStatus CaptureRequest::get_status() {
	std::lock_guard<std::mutex> lock(_mutex);
	return _status;
}

int CaptureRequest::get_frame_buffer_index() {
	return _frame_buffer_index;
}

void CaptureRequest::on_request_pending()
{
	std::lock_guard<std::mutex> lock(_mutex);
	_status = CaptureRequestPending;
}

void CaptureRequest::on_request_completed()
{
	using namespace libcamera;

	std::lock_guard<std::mutex> lock(_mutex);
	_status = CaptureRequestComplete;

	const auto &buffers = _request->buffers();
	auto it = buffers.find(_capturer->_config->at(0).stream());
	if (it == buffers.end()) {
		_cv.notify_all();
		return;
	}
	_frame_buffer = it->second;

	// Initialize space for mapped planes.
	int plane_count = static_cast<int>(_frame_buffer->planes().size());
	clear_mapped_planes();
	_mapped_planes.resize(plane_count, nullptr);
	_cv.notify_all();
}

void CaptureRequest::on_request_cancelled()
{
	std::lock_guard<std::mutex> lock(_mutex);
	_status = CaptureRequestCancelled;
	clear_mapped_planes();
	_cv.notify_all();
}

void CaptureRequest::wait_for_completion()
{
	std::unique_lock<std::mutex> lock(_mutex);
	_cv.wait(lock, [this]() {
		return _status == CaptureRequestComplete || _status == CaptureRequestCancelled;
	});
}

int CaptureRequest::get_plane_count()
{
    std::lock_guard<std::mutex> lock(_mutex);
    if (!_frame_buffer) {
        return -1;
    }

    auto &planes = _frame_buffer->planes();

    return static_cast<int>(planes.size());
}

CaptureRequestMappedPlane* CaptureRequest::get_mapped_plane(int plane_index)
{
	std::lock_guard<std::mutex> lock(_mutex);
	if (!_frame_buffer) {
		return nullptr;
	}

	if (plane_index >= static_cast<int>(_mapped_planes.size())) {
		return nullptr;
	}

	// Check if the plane is already mapped.
	if (_mapped_planes[plane_index] != nullptr) {
		return _mapped_planes[plane_index];
	}

	auto &planes = _frame_buffer->planes();
	auto &plane = planes[plane_index];

	_mapped_planes[plane_index] = new CaptureRequestMappedPlane(plane);


	auto* mapped_plane = _mapped_planes[plane_index];
	return mapped_plane;
}

void CaptureRequest::clear_mapped_planes()
{
	for (auto* mapped_plane : _mapped_planes) {
		if (mapped_plane) {
			delete mapped_plane;
		}
	}
	_mapped_planes.clear();
}

} // namespace rook::lw_libcamera_capture