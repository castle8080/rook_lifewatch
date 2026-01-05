#include "rook_lw_libcamera_capture/rook_lw_libcamera_capture.h"
#include "rook_lw_libcamera_capture/rook_lw_libcamera_capture.hpp"

#include <cstdio>

int main(int argc, char **argv)
{
	if (argc != 2) {
		std::fprintf(stderr, "Usage: %s <output_dir>\n", argv[0]);
		return 2;
	}

	rook::lw_libcamera_capture::CameraCapturer capturer;

	for (unsigned i = 0; i < capturer.cameraCount(); ++i) {
		std::printf("Camera %u: %s\n", i, capturer.cameraName(i).c_str());
	}

	if (capturer.cameraCount() == 0) {
		std::fprintf(stderr, "No cameras found\n");
		return 1;
	}

	capturer.setCameraSource(capturer.cameraName(0));

	/*
	int rc = rook_lw_capture_10_frames(argv[1]);
	if (rc != 0) {
		std::fprintf(stderr, "rook_lw_capture_10_frames failed: %d\n", rc);
		return 1;
	}

	std::printf("Wrote 10 frames to %s\n", argv[1]);
	*/
	return 0;
}
