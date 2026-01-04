#include "rook_lw_libcamera_capture/rook_lw_libcamera_capture.h"

#include <cerrno>
#include <cstdint>
#include <cstdio>
#include <cstdlib>
#include <cstring>
#include <filesystem>
#include <fstream>
#include <mutex>
#include <condition_variable>
#include <chrono>
#include <new>
#include <string>
#include <vector>

#include <libcamera/camera.h>
#include <libcamera/camera_manager.h>
#include <libcamera/control_ids.h>
#include <libcamera/formats.h>
#include <libcamera/framebuffer_allocator.h>
#include <libcamera/request.h>
#include <libcamera/stream.h>

#include <sys/mman.h>

namespace {

class CameraCapturer {
public:
	CameraCapturer()
	{
		int ret = cm_.start();
		if (ret != 0) {
			started_ = false;
			return;
		}
		started_ = true;
	}

	~CameraCapturer()
	{
		shutdown();
	}

	bool ok() const { return started_; }

	unsigned camera_count() const
	{
		return static_cast<unsigned>(cm_.cameras().size());
	}

	const char *camera_name(unsigned index) const
	{
		const auto &cams = cm_.cameras();
		if (index >= cams.size())
			return nullptr;
		// Camera::id() is a std::string owned by libcamera; pointer stays valid
		// as long as the Camera object is alive (which is tied to CameraManager).
		return cams[index]->id().c_str();
	}

private:
	void shutdown()
	{
		if (!started_)
			return;
		cm_.stop();
		started_ = false;
	}

	libcamera::CameraManager cm_;
	bool started_ = false;
};

} // namespace

struct rook_lw_camera_capturer {
	CameraCapturer impl;
};

extern "C" rook_lw_camera_capturer_t *rook_lw_camera_capturer_create(void)
{
	try {
		auto *p = new (std::nothrow) rook_lw_camera_capturer();
		if (!p)
			return nullptr;
		if (!p->impl.ok()) {
			delete p;
			return nullptr;
		}
		return p;
	} catch (...) {
		return nullptr;
	}
}

extern "C" void rook_lw_camera_capturer_destroy(rook_lw_camera_capturer_t *capturer)
{
	delete capturer;
}

extern "C" unsigned rook_lw_camera_capturer_get_camera_count(const rook_lw_camera_capturer_t *capturer)
{
	if (!capturer)
		return 0;
	return capturer->impl.camera_count();
}

extern "C" const char *rook_lw_camera_capturer_get_camera_name(const rook_lw_camera_capturer_t *capturer,
                                                                unsigned index)
{
	if (!capturer)
		return nullptr;
	return capturer->impl.camera_name(index);
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

extern "C" int rook_lw_list_cameras(char ***out_ids, unsigned *out_count)
{
	if (!out_ids || !out_count)
		return -EINVAL;
	*out_ids = nullptr;
	*out_count = 0;

	using namespace libcamera;
	CameraManager cm;
	if (int ret = cm.start(); ret != 0)
		return -EIO;

	std::vector<std::string> ids;
	ids.reserve(cm.cameras().size());
	for (const std::shared_ptr<Camera> &cam : cm.cameras())
		ids.push_back(cam->id());

	cm.stop();

	if (ids.empty()) {
		// No cameras available is not an error for enumeration.
		return 0;
	}

	char **list = static_cast<char **>(std::malloc(sizeof(char *) * ids.size()));
	if (!list)
		return -ENOMEM;
	std::memset(list, 0, sizeof(char *) * ids.size());

	for (size_t i = 0; i < ids.size(); ++i) {
		const std::string &s = ids[i];
		char *p = static_cast<char *>(std::malloc(s.size() + 1));
		if (!p) {
			for (size_t j = 0; j < ids.size(); ++j)
				std::free(list[j]);
			std::free(list);
			return -ENOMEM;
		}
		std::memcpy(p, s.c_str(), s.size() + 1);
		list[i] = p;
	}

	*out_ids = list;
	*out_count = static_cast<unsigned>(ids.size());
	return 0;
}

extern "C" void rook_lw_free_camera_id_list(char **ids, unsigned count)
{
	if (!ids)
		return;
	for (unsigned i = 0; i < count; ++i)
		std::free(ids[i]);
	std::free(ids);
}

int rook_lw_capture_10_frames(const char *output_dir)
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
	// Prefer a simple, widely supported format if available.
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
