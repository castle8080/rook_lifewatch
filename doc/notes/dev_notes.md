# Dev Notes

Notes for tracking ideas and steps.

## Initial Development Environment Setup

* Currently have some projects setup for the main executable in rust and a lib in C++.
* I need to link the C++ and use FFI from rust to the lib.
* The lib does compile and capture images with a helper.

## Next Steps

1. Work on the capture lib more to list cameras
2. Figure out when the functions would like like to get images.
3. Determine if I can setup different resolutions.
  * Would like low res mode for motion detection
  * Then switch to hi res for capture.
4. Need to design rust FFI.

The current rust source files are probably not what I want. They are generated with AI just to get some basic setup working including using build features. I likely will remove opencv as a method of image capture, but may still use it for algorithms.