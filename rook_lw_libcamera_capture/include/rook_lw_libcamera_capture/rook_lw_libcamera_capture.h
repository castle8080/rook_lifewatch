#pragma once

#ifdef __cplusplus
extern "C" {
#endif

// Opens the first available libcamera camera and writes 10 frames to output_dir.
//
// Returns:
//   0 on success
//   negative values on failure
int rook_lw_capture_10_frames(const char *output_dir);

// Returns a newly allocated array of NUL-terminated UTF-8 camera IDs.
//
// Ownership:
// - On success, *out_ids points to an array of *out_count pointers.
// - Each entry (*out_ids)[i] is separately allocated.
// - Call rook_lw_free_camera_id_list() to free the list.
//
// Returns:
//   0 on success
//   negative values on failure
int rook_lw_list_cameras(char ***out_ids, unsigned *out_count);

// Frees a list returned by rook_lw_list_cameras.
void rook_lw_free_camera_id_list(char **ids, unsigned count);

// Opaque camera capturer handle (implemented in C++).
typedef struct rook_lw_camera_capturer rook_lw_camera_capturer_t;

// Creates a capturer instance and starts an internal CameraManager.
//
// Returns:
//   non-null on success
//   null on failure
rook_lw_camera_capturer_t *rook_lw_camera_capturer_create(void);

// Destroys a capturer instance created by rook_lw_camera_capturer_create.
void rook_lw_camera_capturer_destroy(rook_lw_camera_capturer_t *capturer);

// Returns number of cameras currently visible to the capturer.
unsigned rook_lw_camera_capturer_get_camera_count(const rook_lw_camera_capturer_t *capturer);

// Returns the camera "name" (libcamera camera id) for a given index.
//
// The returned pointer is owned by the capturer and remains valid until:
// - the capturer is destroyed, or
// - a future version refreshes the camera list.
//
// Returns null if capturer is null or index is out of range.
const char *rook_lw_camera_capturer_get_camera_name(const rook_lw_camera_capturer_t *capturer,
													unsigned index);

#ifdef __cplusplus
}
#endif
