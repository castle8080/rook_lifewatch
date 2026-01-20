
#include "rook_lw_libcamera_capture/rook_lw_libcamera_capture.hpp"

#include <system_error>
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

#include <sys/mman.h>
#include <unistd.h>

namespace rook::lw_libcamera_capture {

CaptureRequestMappedPlane::CaptureRequestMappedPlane(const libcamera::FrameBuffer::Plane &plane)
{
    using namespace libcamera;

    int fd = plane.fd.get();
    if (fd < 0) {
        _data = nullptr;
        _length = 0;
        return;
    }

    // Make sure that the offset is aligned to page size for mmap.
    // This is necessary because mmap requires the offset to be page-aligned.
    // Failure to do so will result in EINVAL error.
    size_t page = sysconf(_SC_PAGESIZE);
    off_t aligned_offset = plane.offset & ~(page - 1);
    size_t delta = plane.offset - aligned_offset;

    _length = plane.length + delta;
    _data = mmap(nullptr, _length, PROT_READ, MAP_SHARED, fd, aligned_offset);

    // Incorrect code here.
    //_length = plane.length;
    //_data = mmap(nullptr, _length, PROT_READ, MAP_SHARED, fd, plane.offset);

    if (_data == MAP_FAILED) {
        throw CameraException("Failed to mmap plane: " + std::string(std::system_error(errno, std::generic_category()).what()), errno);
    }
}

CaptureRequestMappedPlane::~CaptureRequestMappedPlane()
{
    if (_data) {
        munmap(_data, _length);
    }

    _data = nullptr;
    _length = 0;
}

size_t CaptureRequestMappedPlane::get_length() {
    return _length;
}

void* CaptureRequestMappedPlane::get_data() {
    return _data;
}

} // namespace rook::lw_libcamera_capture