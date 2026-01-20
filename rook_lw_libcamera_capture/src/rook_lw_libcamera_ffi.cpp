#include "rook_lw_libcamera_capture/rook_lw_libcamera_capture.h"

#include "rook_lw_libcamera_capture/rook_lw_libcamera_capture.hpp"

#include <cerrno>
#include <cstdlib>
#include <cstring>
#include <iostream>
#include <string>
#include <vector>

using rook::lw_libcamera_capture::CameraCapturer;
using rook::lw_libcamera_capture::CaptureRequest;
using rook::lw_libcamera_capture::CameraException;
using rook::lw_libcamera_capture::CaptureRequestMappedPlane;

struct rook_lw_camera_capturer {
    CameraCapturer impl;
};

struct rook_lw_capture_request {
	std::shared_ptr<CaptureRequest> impl;
};

extern "C" rook_lw_camera_capturer_t *rook_lw_camera_capturer_create(void)
{
	try {
        auto *capturer = new rook_lw_camera_capturer();
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
	if (!capturer) {
		return;
	}
	delete capturer;
}

extern "C" int32_t rook_lw_camera_capturer_get_camera_count(
    const rook_lw_camera_capturer_t *capturer,
    uint32_t* out_camera_count)
{
	if (!capturer) {
		return static_cast<int32_t>(-EINVAL);
    }
    try {
        *out_camera_count = static_cast<uint32_t>(capturer->impl.camera_count());
        return 0;
    }
    catch (const rook::lw_libcamera_capture::CameraException &) {
        std::cerr << "CameraException caught in rook_lw_camera_capturer_get_camera_count" << std::endl;
        return static_cast<int32_t>(-EIO);
    }
    catch (...) {
        std::cerr << "Unknown exception caught in rook_lw_camera_capturer_get_camera_count" << std::endl;
        return static_cast<int32_t>(-EIO);
    }
}

extern "C" int32_t rook_lw_camera_capturer_get_camera_name(
    const rook_lw_camera_capturer_t *capturer,
    uint32_t index,
    const char **out_camera_name)
{
	if (!capturer) {
		return static_cast<int32_t>(-EINVAL);
    }

    try {
        if (index >= capturer->impl.camera_count()) {
            return static_cast<int32_t>(-EINVAL);
        }
        
        auto* cameraNamePtr = capturer->impl.camera_name(index);
        if (cameraNamePtr == nullptr) {
            return static_cast<int32_t>(-EIO);
        }
        *out_camera_name = cameraNamePtr->c_str();
        return 0;
    }
    catch (const rook::lw_libcamera_capture::CameraException &) {
        std::cerr << "CameraException caught in rook_lw_camera_capturer_get_camera_name" << std::endl;
        return static_cast<int32_t>(-EIO);
    }
    catch (...) {
        std::cerr << "Unknown exception caught in rook_lw_camera_capturer_get_camera_name" << std::endl;
        return static_cast<int32_t>(-EIO);
    }
}

extern "C" int32_t rook_lw_camera_capturer_set_camera_source(
    rook_lw_camera_capturer_t *capturer,
    const char *camera_name,
	uint32_t required_buffer_size)
{
    if (!capturer || !camera_name) {
        return static_cast<int32_t>(-EINVAL);
    }

    try {
        capturer->impl.set_camera_source(std::string(camera_name), required_buffer_size);
        return 0; // Success
    }
    catch (const rook::lw_libcamera_capture::CameraException &e) {
        std::cerr << "CameraException caught in rook_lw_camera_capturer_set_camera_source: " << e.what() << std::endl;
        return (e.code() < 0) ? static_cast<int32_t>(e.code()) : static_cast<int32_t>(-EIO);
    }
    catch (...) {
        std::cerr << "Unknown exception caught in rook_lw_camera_capturer_set_camera_source" << std::endl;
        return static_cast<int32_t>(-EIO);
    }
}

extern "C" int32_t rook_lw_camera_capturer_get_camera_detail(
    rook_lw_camera_capturer_t *capturer,
    char **out_camera_detail)
{
    if (!capturer || !out_camera_detail) {
        return static_cast<int32_t>(-EINVAL);
    }

    try {
        std::string detail = capturer->impl.get_camera_detail();
        char* detail_cstr = static_cast<char*>(std::malloc(detail.size() + 1));
        if (!detail_cstr) {
            return static_cast<int32_t>(-ENOMEM);
        }
        std::strcpy(detail_cstr, detail.c_str());
        *out_camera_detail = detail_cstr;
        return 0; // Success
    }
    catch (const rook::lw_libcamera_capture::CameraException &e) {
        std::cerr << "CameraException caught in rook_lw_camera_capturer_get_camera_detail: " << e.what() << std::endl;
        return (e.code() < 0) ? static_cast<int32_t>(e.code()) : static_cast<int32_t>(-EIO);
    }
    catch (...) {
        std::cerr << "Unknown exception caught in rook_lw_camera_capturer_get_camera_detail" << std::endl;
        return static_cast<int32_t>(-EIO);
    }
}

extern "C" int32_t rook_lw_camera_capturer_get_pixel_format(
    rook_lw_camera_capturer_t *capturer,
    uint32_t *out_pixel_format)
{
    if (!capturer) {
        return static_cast<int32_t>(-EINVAL);
    }

    try {
        *out_pixel_format = capturer->impl.get_pixel_format();
        return 0;
    }
    catch (const rook::lw_libcamera_capture::CameraException &e) {
        std::cerr << "CameraException caught in rook_lw_camera_capturer_get_pixel_format: " << e.what() << std::endl;
        return (e.code() < 0) ? static_cast<int32_t>(e.code()) : static_cast<int32_t>(-EIO);
    }
    catch (...) {
        std::cerr << "Unknown exception caught in rook_lw_camera_capturer_get_pixel_format" << std::endl;
        return static_cast<int32_t>(-EIO);
    }
}


extern "C" int32_t rook_lw_camera_capturer_get_width(
    rook_lw_camera_capturer_t *capturer,
    uint32_t *out_width)
{
    if (!capturer) {
        return static_cast<int32_t>(-EINVAL);
    }

    try {
        *out_width = capturer->impl.get_width();
        return 0;
    }
    catch (const rook::lw_libcamera_capture::CameraException &e) {
        std::cerr << "CameraException caught in rook_lw_camera_capturer_get_width: " << e.what() << std::endl;
        return (e.code() < 0) ? static_cast<int32_t>(e.code()) : static_cast<int32_t>(-EIO);
    }
    catch (...) {
        std::cerr << "Unknown exception caught in rook_lw_camera_capturer_get_width" << std::endl;
        return static_cast<int32_t>(-EIO);
    }
}

extern "C" int32_t rook_lw_camera_capturer_get_height(
    rook_lw_camera_capturer_t *capturer,
    uint32_t *out_height)
{
    if (!capturer) {
        return static_cast<int32_t>(-EINVAL);
    }

    try {
        *out_height = capturer->impl.get_height();
        return 0;
    }
    catch (const rook::lw_libcamera_capture::CameraException &e) {
        std::cerr << "CameraException caught in rook_lw_camera_capturer_get_height: " << e.what() << std::endl;
        return (e.code() < 0) ? static_cast<int32_t>(e.code()) : static_cast<int32_t>(-EIO);
    }
    catch (...) {
        std::cerr << "Unknown exception caught in rook_lw_camera_capturer_get_height" << std::endl;
        return static_cast<int32_t>(-EIO);
    }
}

extern "C" int32_t rook_lw_camera_capturer_get_stride(
    rook_lw_camera_capturer_t *capturer,
    uint32_t *out_stride)
{
    if (!capturer) {
        return static_cast<int32_t>(-EINVAL);
    }

    try {
        *out_stride = capturer->impl.get_stride();
        return 0;
    }
    catch (const rook::lw_libcamera_capture::CameraException &e) {
        std::cerr << "CameraException caught in rook_lw_camera_capturer_get_stride: " << e.what() << std::endl;
        return (e.code() < 0) ? static_cast<int32_t>(e.code()) : static_cast<int32_t>(-EIO);
    }
    catch (...) {
        std::cerr << "Unknown exception caught in rook_lw_camera_capturer_get_stride" << std::endl;
        return static_cast<int32_t>(-EIO);
    }
}

extern "C" int32_t rook_lw_camera_capturer_start(
    rook_lw_camera_capturer_t *capturer)
{
    if (!capturer) {
        return static_cast<int32_t>(-EINVAL);
    }

    try {
        capturer->impl.start();
        return 0; // Success
    }
    catch (const rook::lw_libcamera_capture::CameraException &e) {
        std::cerr << "CameraException caught in rook_lw_camera_capturer_start: " << e.what() << std::endl;
        return (e.code() < 0) ? static_cast<int32_t>(e.code()) : static_cast<int32_t>(-EIO);
    }
    catch (...) {
        std::cerr << "Unknown exception caught in rook_lw_camera_capturer_start" << std::endl;
        return static_cast<int32_t>(-EIO);
    }
}

extern "C" int32_t rook_lw_camera_capturer_stop(
    rook_lw_camera_capturer_t *capturer)
{
    if (!capturer) {
        return static_cast<int32_t>(-EINVAL);
    }

    try {
        capturer->impl.stop();
        return 0; // Success
    }
    catch (const rook::lw_libcamera_capture::CameraException &e) {
        std::cerr << "CameraException caught in rook_lw_camera_capturer_stop: " << e.what() << std::endl;
        return (e.code() < 0) ? static_cast<int32_t>(e.code()) : static_cast<int32_t>(-EIO);
    }
    catch (...) {
        std::cerr << "Unknown exception caught in rook_lw_camera_capturer_stop" << std::endl;
        return static_cast<int32_t>(-EIO);
    }
}

extern "C" rook_lw_capture_request_t * rook_lw_camera_capturer_acquire_frame(
    rook_lw_camera_capturer_t *capturer)
{
    if (!capturer) {
        return nullptr;
    }

    try {
        std::shared_ptr<CaptureRequest> impl = capturer->impl.acquire_frame();
        if (!impl) {
            return nullptr;
        }
        rook_lw_capture_request_t *request = new rook_lw_capture_request_t();
        if (!request) {
            return nullptr;
        }
        request->impl = impl;
        return request;
    }
    catch (const rook::lw_libcamera_capture::CameraException &e) {
        std::cerr << "CameraException caught in rook_lw_camera_capturer_acquire_frame: " << e.what() << std::endl;
        return nullptr;
    }
    catch (...) {
        std::cerr << "Unknown exception caught in rook_lw_camera_capturer_acquire_frame" << std::endl;
        return nullptr;
    }
}

extern "C" void rook_lw_capture_request_destroy(rook_lw_capture_request_t *request) {
    if (!request) {
        return;
    }
    delete request;
}

extern "C" int32_t rook_lw_capture_request_get_status(
    rook_lw_capture_request_t *capture_request,
    int32_t *out_status)
{
    if (!capture_request || !out_status) {
        return static_cast<int32_t>(-EINVAL);
    }

    try {
        auto status = capture_request->impl->get_status();
        *out_status = static_cast<int32_t>(status);
        return 0; // Success
    }
    catch (const rook::lw_libcamera_capture::CameraException &e) {
        std::cerr << "CameraException caught in rook_lw_capture_request_get_status: " << e.what() << std::endl;
        return (e.code() < 0) ? static_cast<int32_t>(e.code()) : static_cast<int32_t>(-EIO);
    }
    catch (...) {
        std::cerr << "Unknown exception caught in rook_lw_capture_request_get_status" << std::endl;
        return static_cast<int32_t>(-EIO);
    }
}

extern "C" int32_t rook_lw_capture_request_wait_for_completion(
    rook_lw_capture_request_t *capture_request)
{
    if (!capture_request) {
        return static_cast<int32_t>(-EINVAL);
    }

    try {
        capture_request->impl->wait_for_completion();
        return 0; // Success
    }
    catch (const rook::lw_libcamera_capture::CameraException &e) {
        std::cerr << "CameraException caught in rook_lw_capture_request_wait_for_completion: " << e.what() << std::endl;
        return (e.code() < 0) ? static_cast<int32_t>(e.code()) : static_cast<int32_t>(-EIO);
    }
    catch (...) {
        std::cerr << "Unknown exception caught in rook_lw_capture_request_wait_for_completion" << std::endl;
        return static_cast<int32_t>(-EIO);
    }
}

extern "C" int32_t rook_lw_capture_request_get_plane_count(
    rook_lw_capture_request_t *capture_request,
	int32_t *out_plane_count)
{
    if (!capture_request || !out_plane_count) {
        return static_cast<int32_t>(-EINVAL);
    }
    try {
        int plane_count = capture_request->impl->get_plane_count();
        *out_plane_count = plane_count;
        return 0; // Success
    }
    catch (const rook::lw_libcamera_capture::CameraException &e) {
        std::cerr << "CameraException caught in rook_lw_capture_request_get_plane_count: " << e.what() << std::endl;
        return (e.code() < 0) ? static_cast<int32_t>(e.code()) : static_cast<int32_t>(-EIO);
    }
    catch (...) {
        std::cerr << "Unknown exception caught in rook_lw_capture_request_get_plane_count" << std::endl;
        return static_cast<int32_t>(-EIO);
    }
}

extern "C" int32_t rook_lw_capture_request_get_plane_data(
    rook_lw_capture_request_t *capture_request,
    int32_t plane_index,
    void **plane_data,
    size_t *plane_length)
{
    if (!capture_request || !plane_data || !plane_length) {
        return static_cast<int32_t>(-EINVAL);
    }
    try {
        CaptureRequestMappedPlane* plane = capture_request->impl->get_mapped_plane(plane_index);
        if (!plane) {
            return static_cast<int32_t>(-EINVAL);
        }
        *plane_data = plane->get_data();
        *plane_length = plane->get_length();
        return 0; // Success
    }
    catch (const rook::lw_libcamera_capture::CameraException &e) {
        std::cerr << "CameraException caught in rook_lw_capture_request_get_plane_data: " << e.what() << std::endl;
        return (e.code() < 0) ? static_cast<int32_t>(e.code()) : static_cast<int32_t>(-EIO);
    }
    catch (...) {
        std::cerr << "Unknown exception caught in rook_lw_capture_request_get_plane_data" << std::endl;
        return static_cast<int32_t>(-EIO);
    }
}
