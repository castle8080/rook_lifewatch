# Dev Notes

Notes for tracking ideas and steps.

## Tasks

### Features

*rook_lw_admin / rook_lw_admin_fe*

1. Image display page with search filters.
2. Display image (not through static file handler)
3. Draw detection boxes on screen.
4. Summary view including analysis of detected classes.
5. Ability to set location of the device.

*hardware device and testing*

1. Setup and run test with new camera (still using USB based web camera.)
2. Try running with battery banks and see how long it will run.
3. Configure ability to either connect to phone hotspot or put device into hotspot mod itself, when no connection. (Should be able to connect to it in the field with no wifi it connects to.) It starting hotspot is probably good as either laptop or phone could then connect to it.

*rook_lw_daemon*

1. Add 2nd stage object detector using better ML model. (yolov4-tiny current being used.)
2. Use image similarity search to calculcate uniqueness of images. See if embeddings of images can be created to make similary search fast enough of pi, and use for filtering or at least for searching for more interesting pictures.
3. Refine motion detection more.
4. Record device identifier / name with records.
5. Record device location with records.

*rook_lw_client*

**doesn't exist yet**

Create a client app (desktop/android) that installs directly on device to sync/cache data. Considering using tauri and using webview to run same app on device. Fun feature would be to detect devices on network (port scan?) and connect to them.

*rook_lw_server*

**doesn't exist yet**

Server that can be used to collect multiple devices and have long term storage and more advanced analysis. Devices can then be cleared as well. Should allow for device to directly communicate upload, or allow client to collect and then upload.

### Tech Debt / Clean Up

*rook_lw_admin*

1. Add https support.
2. Add login and support for JWTs.


### Low(ish) Priority Tech Debt / Clean Up
Things to do to generally improve the code more. Not really needed yet.

*rook_lw_daemon*

1. Examine thread boundaries and add logging or error reporting.
2. Add panic handlers for threads.
3. Have configuration look at environment variables to get data directories.
4. Have configuration read from json/jsonc.
5. Add timing logs around core image processing operations.
6. Log stats about memory and CPU usage.
7. Configure tracing to output rolling logs and use a format where I can extracted structured info.
8. Add command line option parsing (should be able to choose camera and frame capture source)

*rook_lw_admin*

1. Add timing logs around core image processing operations.
2. Log stats about memory and CPU usage.
3. Configure tracing to output rolling logs and use a format where I can extracted structured info.

*scripts / startup*

1. Pid checking just uses ps to see if apps are running. Use pid lock files.
2. Add shutdown scripts.


*dev env*

1. Get compilation and running on windows with opencv as frame source.
2. For better testing create video file frame source (simulation).

### Current Task Notes

I have the new camera in IR mode working and had to adjust for image formats.
I ran into some issues:

1. mmap failed on some planes and learned I need to do some page offset changes.
2. The camera had a default frame buffer of 1 and I needed 2 for the app. I need to expose a way to set frame buffer on the ffi layer.
3. The camera is doesn't change day/night mode, but there are some settings that might make day look better.

I want to look into changing controls:

```

#include <libcamera/libcamera.h>

std::unique_ptr<libcamera::Camera> camera = ...; // initialized camera

libcamera::ControlList controls(camera->controls());
controls.set(libcamera::controls::AwbMode, libcamera::controls::AwbModeValues::Daylight);
controls.set(libcamera::controls::ColourSaturation, 1.0f);
controls.set(libcamera::controls::AnalogueGain, 256);
controls.set(libcamera::controls::ExposureTime, 8000);

camera->setControls(controls);

```