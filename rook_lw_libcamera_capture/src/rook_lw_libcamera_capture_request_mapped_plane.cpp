
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

    // Align the offset down to the nearest page boundary.
    // This works by removing the lower bits of the offset that are less than the page size.
    // mmap fails if you try to directly map with a non-page-aligned offset.
    size_t page = sysconf(_SC_PAGESIZE);
    off_t aligned_offset = plane.offset & ~(page - 1);
    size_t delta = plane.offset - aligned_offset;
    size_t map_length = plane.length + delta;
    _data = mmap(nullptr, map_length, PROT_READ, MAP_SHARED, fd, aligned_offset);

    if (_data == MAP_FAILED) {
        throw CameraException("Failed to mmap plane: " + std::string(std::system_error(errno, std::generic_category()).what()), errno);
    }

    // Store the logical data length (from the original offset).
    _length = plane.length;

    // Store the delta to adjust the data pointer when returning data.
    _data_delta = delta;
}

CaptureRequestMappedPlane::~CaptureRequestMappedPlane()
{
    if (_data) {
        munmap(_data, _length);
    }

    _data = nullptr;
    _length = 0;
    _data_delta = 0;
}

size_t CaptureRequestMappedPlane::get_length() {
    return _length;
}

void* CaptureRequestMappedPlane::get_data() {
    // Adjust the data pointer by the delta to get the correct starting point.
    // This occurs because we may have mapped from a page-aligned offset.
    return static_cast<char*>(_data) + _data_delta;
}

} // namespace rook::lw_libcamera_capture