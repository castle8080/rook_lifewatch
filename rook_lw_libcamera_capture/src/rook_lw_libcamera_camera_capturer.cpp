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

std::shared_ptr<libcamera::Camera> get_camera(const std::string &camera_name, libcamera::CameraManager &camera_manager)
{
	for (auto &cam : camera_manager.cameras()) {
		if (cam->id() == camera_name) {
			return cam;
		}
	}
	return nullptr;
}

CameraCapturer::CameraCapturer()
{
	int ret = _camera_manager.start();
	if (ret != 0) {
		throw CameraException("Failed to start libcamera CameraManager", -EIO);
	}
}

CameraCapturer::~CameraCapturer()
{
    reset_camera();
    _camera_manager.stop();
}

unsigned CameraCapturer::camera_count() const
{
	return static_cast<unsigned>(_camera_manager.cameras().size());
}

const std::string* CameraCapturer::camera_name(unsigned index) const
{
	const auto &cams = _camera_manager.cameras();
	if (index >= cams.size()) {
		return nullptr;
	}
	auto& camera_name = cams[index]->id();
	return &camera_name;
}

void CameraCapturer::reset_camera()
{
	if (_camera) {
		if (_is_camera_started) {
			try {
				_camera->stop();
				_is_camera_started = false;
			} catch (...) {
				// Ignore exceptions during cleanup
			}
		}
		_camera->release();
		_camera.reset();
	}
	_allocator.reset();
	_config.reset();
}

void CameraCapturer::set_camera_source(const std::string &camera_name)
{
	using namespace libcamera;

	if (_camera) {
		throw CameraException("Camera source already set", -EINVAL);
	}

	_camera = get_camera(camera_name, _camera_manager);

	if (!_camera) {
		throw CameraException("Camera with specified name not found", -ENODEV);
	}

	if (int ret = _camera->acquire(); ret != 0) {
		reset_camera();
		throw CameraException("Failed to acquire camera", -EACCES);
	}

	_allocator = std::make_shared<FrameBufferAllocator>(_camera);
	if (!_allocator) {
		reset_camera();
		throw CameraException("Failed to create FrameBufferAllocator", -ENOMEM);
	}

	_config = _camera->generateConfiguration({ StreamRole::StillCapture });
	if (!_config || _config->empty()) {
		reset_camera();
		throw CameraException("Failed to generate camera configuration", -EINVAL);
	}

	if (_config->validate() == CameraConfiguration::Invalid) {
		reset_camera();
		throw CameraException("Invalid camera configuration", -EINVAL);
	}

	if (int ret = _camera->configure(_config.get()); ret != 0) {
		reset_camera();
		throw CameraException("Failed to configure camera", -EIO);
	}

	StreamConfiguration &stream_config = _config->at(0);

	// Allocate frame buffers for the configured stream.
	Stream *stream = stream_config.stream();
	if (int ret = _allocator->allocate(stream); ret < 0) {
		reset_camera();
		throw CameraException("Failed to allocate frame buffers", -ENOMEM);
	}

	// Register callback for request completion.
	_camera->requestCompleted.connect(this, &CameraCapturer::on_request_completed);
}

uint32_t CameraCapturer::get_pixel_format() {
	using namespace libcamera;

	if (!_camera || !_config) {
		throw CameraException("Camera source not set", -EINVAL);
	}

	StreamConfiguration &stream_config = _config->at(0);

	return stream_config.pixelFormat.fourcc();
}

uint32_t CameraCapturer::get_width() {
	using namespace libcamera;

	if (!_camera || !_config) {
		throw CameraException("Camera source not set", -EINVAL);
	}

	StreamConfiguration &stream_config = _config->at(0);

	return stream_config.size.width;
}

uint32_t CameraCapturer::get_height() {
	using namespace libcamera;

	if (!_camera || !_config) {
		throw CameraException("Camera source not set", -EINVAL);
	}

	StreamConfiguration &stream_config = _config->at(0);

	return stream_config.size.height;
}

void CameraCapturer::start()
{
	if (_is_camera_started) {
		return;
	}

	if (!_camera) {
		throw CameraException("Camera source not set", -EINVAL);
	}

	if (int ret = _camera->start(); ret != 0) {
		throw CameraException("Failed to start camera", -EIO);
	}

	_is_camera_started = true;
}

void CameraCapturer::stop()
{
	if (!_is_camera_started) {
		return;
	}

	if (!_camera) {
		throw CameraException("Camera source not set", -EINVAL);
	}

	if (int ret = _camera->stop(); ret != 0) {
		throw CameraException("Failed to stop camera", -EIO);
	}

	_is_camera_started = false;
}

int CameraCapturer::checkout_frame_buffer_index()
{
	using namespace libcamera;

	if (!_camera || !_allocator || !_config) {
		throw CameraException("Camera source not set", -EINVAL);
	}

	Stream *stream = _config->at(0).stream();
	auto& buffers = _allocator->buffers(stream);
	
	// Implementation to checkout a frame buffer index
	{
		std::lock_guard<std::mutex> lock(_mutex);
		for (std::size_t i = 0; i < buffers.size(); ++i) {
			if (_in_use_frame_buffer_indices.find(i) == _in_use_frame_buffer_indices.end()) {
				_in_use_frame_buffer_indices.insert(i);
				return i;
			}
		}
	}

	return -1;
}

void CameraCapturer::return_frame_buffer_index(int index)
{
	// Implementation to return a frame buffer index
	{
		std::lock_guard<std::mutex> lock(_mutex);
		_in_use_frame_buffer_indices.erase(index);
	}
}

void CameraCapturer::release_request_resources(CaptureRequest* request)
{
	if (!request) {
		return;
	}

	int frame_buffer_index = request->get_frame_buffer_index();
	return_frame_buffer_index(frame_buffer_index);
}

std::shared_ptr<CaptureRequest> CameraCapturer::acquire_frame()
{
	using namespace libcamera;

	if (!_camera || !_allocator || !_config) {
		throw CameraException("Camera source not set", -EINVAL);
	}

	if (!_is_camera_started) {
		throw CameraException("Camera not started", -EINVAL);
	}

	Stream *stream = _config->at(0).stream();

	// TODO: might not assume that buffer 0 is free.
	// Need to track buffers in use vs free.
	auto& buffers = _allocator->buffers(stream);

	// Get a frame buffer that can be used for this request.
	int frame_buffer_index = checkout_frame_buffer_index();
	if (frame_buffer_index < 0) {
		throw CameraException("No available frame buffers", -EIO);
	}

	std::shared_ptr<libcamera::Request> request = std::move(_camera->createRequest(_next_request_sequence++));
	if (request->addBuffer(stream, buffers[frame_buffer_index].get()) != 0) {
		return_frame_buffer_index(frame_buffer_index);
		throw CameraException("Failed to add buffer to request", -EIO);
	}

	std::shared_ptr<CaptureRequest> capture_request =
		std::make_shared<CaptureRequest>(this, request, frame_buffer_index);

	// Store the CaptureRequest associated with this request's cookie.
	{
		std::lock_guard<std::mutex> lock(_mutex);
		_requests[request->cookie()] = capture_request;
	}

	if (_camera->queueRequest(request.get()) != 0) {
		return_frame_buffer_index(frame_buffer_index);
		{
			std::lock_guard<std::mutex> lock(_mutex);
			_requests.erase(request->cookie());
		}
		throw CameraException("Failed to queue request", -EIO);
	}

	capture_request->on_request_pending();

	return capture_request;
}

void CameraCapturer::on_request_completed(libcamera::Request *request) {
	if (!request) {
		return;
	}

	// Find the associated CaptureRequest and notify it.
	auto it = _requests.find(request->cookie());
	if (it != _requests.end()) {
		auto value = std::move(it->second);
		_requests.erase(it);

		if (request->status() == libcamera::Request::RequestCancelled) {
			value->on_request_cancelled();
		}
		else {
			value->on_request_completed();
		}
	}
}

} // namespace rook::lw_libcamera_capture
