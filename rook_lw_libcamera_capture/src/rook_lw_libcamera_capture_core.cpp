#include "rook_lw_libcamera_capture/rook_lw_libcamera_capture.hpp"

#include <cerrno>
#include <condition_variable>
#include <chrono>
#include <cstdio>
#include <cstring>
#include <filesystem>
#include <fstream>
#include <mutex>
#include <string>
#include <vector>

#include <libcamera/camera.h>
#include <libcamera/camera_manager.h>
#include <libcamera/formats.h>
#include <libcamera/framebuffer_allocator.h>
#include <libcamera/request.h>
#include <libcamera/stream.h>

#include <sys/mman.h>

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
				std::cout << "Stopping camera before reset" << std::endl;
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

	std::cout << "Setting camera source to: " << camera_name << std::endl;

	_camera = get_camera(camera_name, _camera_manager);

	if (!_camera) {
		throw CameraException("Camera with specified name not found", -ENODEV);
	}

	std::cout << "Acquiring camera: " << _camera->id() << std::endl;

	if (int ret = _camera->acquire(); ret != 0) {
		reset_camera();
		throw CameraException("Failed to acquire camera", -EACCES);
	}

	std::cout << "Creating FrameBufferAllocator" << std::endl;

	_allocator = std::make_shared<FrameBufferAllocator>(_camera);
	if (!_allocator) {
		reset_camera();
		throw CameraException("Failed to create FrameBufferAllocator", -ENOMEM);
	}

	std::cout << "Configuring camera" << std::endl;

	_config = _camera->generateConfiguration({ StreamRole::Viewfinder });
	if (!_config || _config->empty()) {
		reset_camera();
		throw CameraException("Failed to generate camera configuration", -EINVAL);
	}

	StreamConfiguration &stream_config = _config->at(0);
	stream_config.pixelFormat = formats::BGR888;
	stream_config.size.width = 640;
	stream_config.size.height = 480;

	if (_config->validate() == CameraConfiguration::Invalid) {
		reset_camera();
		throw CameraException("Invalid camera configuration", -EINVAL);
	}

	std::cout << "Pixel format: " << stream_config.pixelFormat << std::endl;

	if (int ret = _camera->configure(_config.get()); ret != 0) {
		reset_camera();
		throw CameraException("Failed to configure camera", -EIO);
	}

	// Allocate frame buffers for the configured stream.
	std::cout << "Allocating frame buffers" << std::endl;
	Stream *stream = stream_config.stream();
	if (int ret = _allocator->allocate(stream); ret < 0) {
		reset_camera();
		throw CameraException("Failed to allocate frame buffers", -ENOMEM);
	}

	// Register callback for request completion.
	_camera->requestCompleted.connect(this, &CameraCapturer::on_request_completed);
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

	std::cout << "Acquiring frame:" << __FILE__ << ":" << __LINE__ << std::endl;

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

	if (_camera->queueRequest(request.get()) != 0) {
		return_frame_buffer_index(frame_buffer_index);
		throw CameraException("Failed to queue request", -EIO);
	}

	std::shared_ptr<CaptureRequest> capture_request =
		std::make_shared<CaptureRequest>(this, request, frame_buffer_index);

	capture_request->set_status(CaptureRequestPending);

	// Store the CaptureRequest associated with this request's cookie.
	{
		std::lock_guard<std::mutex> lock(_mutex);
		_requests[request->cookie()] = capture_request;
	}

	std::cout << "Acquiring frame:" << __FILE__ << ":" << __LINE__ << std::endl;

	return capture_request;
}

void CameraCapturer::on_request_completed(libcamera::Request *request) {
	std::cout << "on_request_completed:" << __FILE__ << ":" << __LINE__ << std::endl;
	if (!request) {
		return;
	}

	std::cout
		<< "Request completed with status: "
		<< request->status()
		<< " sequence->" << request->sequence()
		<< " cookie->" << request->cookie()
		<< std::endl;

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

CaptureRequest::~CaptureRequest()
{
	std::cout << "CaptureRequest::~CaptureRequest()" << std::endl;

	if (_capturer) {
		_capturer->release_request_resources(this);
	}
}

void CaptureRequest::set_status(CaptureRequestStatus status) {
	std::lock_guard<std::mutex> lock(_mutex);
	_status = status;
	_cv.notify_all();
}

CaptureRequestStatus CaptureRequest::get_status() {
	std::lock_guard<std::mutex> lock(_mutex);
	return _status;
}

int CaptureRequest::get_frame_buffer_index() {
	return _frame_buffer_index;
}

void CaptureRequest::on_request_completed()
{
	std::cout << "CaptureRequest::on_request_completed()" << std::endl;
	set_status(CaptureRequestComplete);
}

void CaptureRequest::on_request_cancelled()
{
	std::cout << "CaptureRequest::on_request_cancelled()" << std::endl;
	set_status(CaptureRequestComplete);
}

void CaptureRequest::wait_for_completion()
{
	std::unique_lock<std::mutex> lock(_mutex);
	_cv.wait(lock, [this]() {
		return _status == CaptureRequestComplete || _status == CaptureRequestCancelled;
	});
}

namespace {

int ensure_dir(const char *path)
{
	try {
		std::filesystem::create_directories(std::filesystem::path(path));
		return 0;
	} catch (...) {
		return -1;
	}
}

struct MappedPlane {
	void *addr = nullptr;
	size_t length = 0;
};

MappedPlane map_plane(int fd, size_t length)
{
	MappedPlane mapped;
	mapped.length = length;

	void *addr = mmap(nullptr, length, PROT_READ, MAP_SHARED, fd, 0);
	if (addr == MAP_FAILED)
		return {};

	mapped.addr = addr;
	return mapped;
}

void unmap_plane(const MappedPlane &mapped)
{
	if (mapped.addr && mapped.length)
		munmap(mapped.addr, mapped.length);
}

int write_frame_raw(const std::string &file_path, const libcamera::FrameBuffer &buffer)
{
	std::ofstream out(file_path, std::ios::binary);
	if (!out)
		return -1;

	const auto &planes = buffer.planes();
	for (unsigned i = 0; i < planes.size(); ++i) {
		const auto &plane = planes[i];
		const int fd = plane.fd.get();
		const size_t length = plane.length;

		MappedPlane mapped = map_plane(fd, length);
		if (!mapped.addr)
			return -1;

		out.write(static_cast<const char *>(mapped.addr), static_cast<std::streamsize>(mapped.length));
		unmap_plane(mapped);

		if (!out)
			return -1;
	}

	return 0;
}

struct CaptureContext {
	std::mutex mtx;
	std::condition_variable cv;
	bool done = false;
	int frames_written = 0;
	int error = 0;

	std::string output_dir;
	libcamera::Stream *stream = nullptr;
	libcamera::Camera *camera = nullptr;

	void on_request_completed(libcamera::Request *request)
	{
		using namespace libcamera;
		if (!request)
			return;

		if (request->status() == Request::RequestCancelled)
			return;

		const auto &buffers = request->buffers();
		auto it = buffers.find(stream);
		if (it == buffers.end()) {
			std::lock_guard<std::mutex> lock(mtx);
			error = -EIO;
			done = true;
			cv.notify_one();
			return;
		}

		const FrameBuffer *buffer = it->second;

		int local_index = 0;
		{
			std::lock_guard<std::mutex> lock(mtx);
			local_index = frames_written;
		}

		char name[64];
		std::snprintf(name, sizeof(name), "frame_%03d.raw", local_index);
		std::filesystem::path out_path = std::filesystem::path(output_dir) / name;

		if (write_frame_raw(out_path.string(), *buffer) != 0) {
			std::lock_guard<std::mutex> lock(mtx);
			error = -EIO;
			done = true;
			cv.notify_one();
			return;
		}

		bool should_stop = false;
		{
			std::lock_guard<std::mutex> lock(mtx);
			frames_written++;
			should_stop = (frames_written >= 10);
			if (should_stop)
				done = true;
		}

		if (should_stop) {
			cv.notify_one();
			return;
		}

		request->reuse(Request::ReuseBuffers);
		camera->queueRequest(request);
	}
};

} // namespace

int listCameras(std::vector<std::string> &out_ids)
{
	out_ids.clear();

	using namespace libcamera;
	CameraManager cm;
	if (int ret = cm.start(); ret != 0)
		return -EIO;

	out_ids.reserve(cm.cameras().size());
	for (const std::shared_ptr<Camera> &cam : cm.cameras())
		out_ids.push_back(cam->id());

	cm.stop();
	return 0;
}

int capture10Frames(const char *output_dir)
{
	if (!output_dir || !*output_dir)
		return -EINVAL;

	if (ensure_dir(output_dir) != 0)
		return -EIO;

	using namespace libcamera;

	CameraManager cm;
	if (int ret = cm.start(); ret != 0)
		return -EIO;

	if (cm.cameras().empty()) {
		cm.stop();
		return -ENODEV;
	}

	std::shared_ptr<Camera> camera = cm.cameras().front();
	if (int ret = camera->acquire(); ret != 0) {
		cm.stop();
		return -EACCES;
	}

	std::unique_ptr<CameraConfiguration> config = camera->generateConfiguration({ StreamRole::Viewfinder });
	if (!config || config->empty()) {
		camera->release();
		cm.stop();
		return -EINVAL;
	}

	StreamConfiguration &stream_config = config->at(0);
	stream_config.pixelFormat = formats::YUV420;
	stream_config.size.width = 640;
	stream_config.size.height = 480;

	if (config->validate() == CameraConfiguration::Invalid) {
		camera->release();
		cm.stop();
		return -EINVAL;
	}

	if (int ret = camera->configure(config.get()); ret != 0) {
		camera->release();
		cm.stop();
		return -EIO;
	}

	Stream *stream = stream_config.stream();
	FrameBufferAllocator allocator(camera);
	if (int ret = allocator.allocate(stream); ret < 0) {
		camera->release();
		cm.stop();
		return -ENOMEM;
	}

	std::vector<std::unique_ptr<Request>> requests;
	requests.reserve(allocator.buffers(stream).size());

	for (const std::unique_ptr<FrameBuffer> &buffer : allocator.buffers(stream)) {
		std::unique_ptr<Request> request = camera->createRequest();
		if (!request)
			continue;

		if (request->addBuffer(stream, buffer.get()) != 0)
			continue;

		requests.push_back(std::move(request));
	}

	if (requests.empty()) {
		allocator.free(stream);
		camera->release();
		cm.stop();
		return -ENOMEM;
	}

	CaptureContext ctx;
	ctx.output_dir = output_dir;
	ctx.stream = stream;
	ctx.camera = camera.get();

	camera->requestCompleted.connect(&ctx, &CaptureContext::on_request_completed);

	if (int ret = camera->start(); ret != 0) {
		allocator.free(stream);
		camera->release();
		cm.stop();
		return -EIO;
	}

	for (std::unique_ptr<Request> &req : requests) {
		if (camera->queueRequest(req.get()) != 0) {
			camera->stop();
			allocator.free(stream);
			camera->release();
			cm.stop();
			return -EIO;
		}
	}

	{
		std::unique_lock<std::mutex> lock(ctx.mtx);
		ctx.cv.wait_for(lock, std::chrono::seconds(10), [&] { return ctx.done; });
	}

	camera->stop();
	allocator.free(stream);
	camera->release();
	cm.stop();

	if (ctx.error)
		return ctx.error;
	if (ctx.frames_written < 10)
		return -ETIMEDOUT;
	return 0;
}

} // namespace rook::lw_libcamera_capture
