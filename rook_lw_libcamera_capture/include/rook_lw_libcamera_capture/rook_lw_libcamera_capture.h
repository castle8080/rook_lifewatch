#pragma once

#include <stdint.h>
#include <stddef.h>

#ifdef __cplusplus
extern "C" {
#endif

// Opaque camera capturer handle (implemented in C++).
typedef struct rook_lw_camera_capturer rook_lw_camera_capturer_t;

// Opaque capture request handle (implemented in C++).
typedef struct rook_lw_capture_request rook_lw_capture_request_t;

// Creates a capturer instance and starts an internal CameraManager.
//
// Returns:
//   non-null on success
//   null on failure
rook_lw_camera_capturer_t *rook_lw_camera_capturer_create(void);

// Destroys a capturer instance created by rook_lw_camera_capturer_create.
void rook_lw_camera_capturer_destroy(
	rook_lw_camera_capturer_t *capturer);

// Returns number of cameras currently visible to the capturer.
int32_t rook_lw_camera_capturer_get_camera_count(
	const rook_lw_camera_capturer_t *capturer,
	uint32_t* out_camera_count);

// Returns the camera "name" (libcamera camera id) for a given index.
//
// The returned pointer is owned by the capturer and remains valid until:
// - the capturer is destroyed, or
// - a future version refreshes the camera list.
//
// Returns null if capturer is null or index is out of range.
int32_t rook_lw_camera_capturer_get_camera_name(
	const rook_lw_camera_capturer_t *capturer,
	uint32_t index,
	const char **out_camera_name);

// Sets the camera source by libcamera camera id (as returned by
// rook_lw_camera_capturer_get_camera_name).
//
// Returns 0 on success or a negative errno-style code on error.
int32_t rook_lw_camera_capturer_set_camera_source(
	rook_lw_camera_capturer_t *capturer,
	const char *camera_name);

int32_t rook_lw_camera_capturer_start(
	rook_lw_camera_capturer_t *capturer);

int32_t rook_lw_camera_capturer_stop(
	rook_lw_camera_capturer_t *capturer);

int32_t rook_lw_camera_capturer_get_pixel_format(
	rook_lw_camera_capturer_t *capturer,
	uint32_t *out_pixel_format);

int32_t rook_lw_camera_capturer_get_width(
	rook_lw_camera_capturer_t *capturer,
	uint32_t *out_width);

int32_t rook_lw_camera_capturer_get_height(
	rook_lw_camera_capturer_t *capturer,
	uint32_t *out_height);

rook_lw_capture_request_t *rook_lw_camera_capturer_acquire_frame(
	rook_lw_camera_capturer_t *capturer);

void rook_lw_capture_request_destroy(
	rook_lw_capture_request_t *capture_request);

int32_t rook_lw_capture_request_wait_for_completion(
	rook_lw_capture_request_t *capture_request);

int32_t rook_lw_capture_request_get_status(
	rook_lw_capture_request_t *capture_request,
	int32_t *out_status);

int32_t rook_lw_capture_request_get_plane_count(
	rook_lw_capture_request_t *capture_request,
	int *out_plane_count);

int32_t rook_lw_capture_request_get_plane_data(
	rook_lw_capture_request_t *capture_request,
	int plane_index,
	void **plane_data,
	size_t *plane_length);

#ifdef __cplusplus
}
#endif
