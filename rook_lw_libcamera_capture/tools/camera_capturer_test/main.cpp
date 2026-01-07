#include "rook_lw_libcamera_capture/rook_lw_libcamera_capture.h"
#include "rook_lw_libcamera_capture/rook_lw_libcamera_capture.hpp"

#include <cstdio>

int main(int argc, char **argv)
{
	if (argc != 2) {
		std::fprintf(stderr, "Usage: %s <output_dir>\n", argv[0]);
		return 2;
	}

	try {
		rook::lw_libcamera_capture::CameraCapturer capturer;

		for (unsigned i = 0; i < capturer.camera_count(); ++i) {
			std::printf("Camera %u: %s\n", i, capturer.camera_name(i)->c_str());
		}

		if (capturer.camera_count() == 0) {
			std::fprintf(stderr, "No cameras found\n");
			return 1;
		}

		const std::string* cameraName = capturer.camera_name(0);
		if (cameraName != nullptr) {
			capturer.set_camera_source(*cameraName);
		}

		capturer.start();

		for (int i = 0; i < 5; i++)
		{
			std::cout << "Capturing frame " << (i + 1) << "..." << std::endl;
			std::cout << "Acquiring frame..." << std::endl;
			auto capture_request = capturer.acquire_frame();

			std::cout << "Waiting for frame completion..." << std::endl;
			capture_request->wait_for_completion();

			std::cout << "From completed: status = " << capture_request->get_status() << std::endl;
		}

		capturer.stop();
	}
	catch (const rook::lw_libcamera_capture::CameraException &e) {
		std::fprintf(stderr, "CameraException: %s (code %d)\n", e.what(), e.code());
		return 1;
	}
	catch (const std::exception &e) {
		std::fprintf(stderr, "Exception: %s\n", e.what());
		return 1;
	}
	catch (...) {
		std::fprintf(stderr, "Unknown exception caught\n");
		return 1;
	}

	return 0;
}
