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

#ifdef __cplusplus
}
#endif
