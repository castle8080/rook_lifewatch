#include "rook_lw_libcamera_capture/rook_lw_libcamera_capture.h"
#include "rook_lw_libcamera_capture/rook_lw_libcamera_capture.hpp"

#include <cstdio>
#include <filesystem>
#include <fstream>

int main(int argc, char **argv)
{
	if (argc != 2) {
		std::fprintf(stderr, "Usage: %s <output_dir>\n", argv[0]);
		return 2;
	}

	const char* path = argv[1];

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

		std::cout << "Pixel Format: " << capturer.get_pixel_format() << std::endl;

		std::filesystem::create_directories(std::filesystem::path(path));

		capturer.start();

		for (int i = 0; i < 5; i++)
		{
			std::cout << "Capturing frame " << (i + 1) << "..." << std::endl;
			std::cout << "Acquiring frame..." << std::endl;
			auto capture_request = capturer.acquire_frame();

			std::cout << "Waiting for frame completion..." << std::endl;
			capture_request->wait_for_completion();

			std::cout << "From completed: status = " << capture_request->get_status() << std::endl;

			int plane_count = capture_request->get_plane_count();

			std::cout << "Plane count: " << plane_count << std::endl;

			for (int pi = 0; pi < plane_count; pi++) {
				auto* mapped_plane = capture_request->get_mapped_plane(pi);
				if (mapped_plane) {

					size_t data_size = mapped_plane->get_length();
					void* data = mapped_plane->get_data();

					std::cout
						<< "Got mapped plane " << pi
						<< " size: " << mapped_plane->get_length()
						<< " data ptr: " << mapped_plane->get_data()
						<< std::endl;

					// Write plane data to file
					std::filesystem::path file_path = std::filesystem::path(path) / ("frame_" + std::to_string(i) + "_plane_" + std::to_string(pi) + ".raw");
					std::ofstream out(file_path, std::ios::binary);
					if (!out) {
						std::fprintf(stderr, "Failed to open file for writing: %s\n", file_path.string().c_str());
						continue;
					}
					out.write(static_cast<const char*>(data), static_cast<std::streamsize>(data_size));
				}
			}

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
