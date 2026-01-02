# Initial High-Level Design: rook_life_watch

## 1. Goals and Constraints

- Detect animals / interesting motion events around the house using cameras and motion sensors.
- Target platforms:
  - Raspberry Pi 5 (primary deployment, with PIR/motion sensors and Pi camera or USB camera).
  - General desktop / laptop Linux (development, simulation, and possibly secondary deployment).
- Allow development of core logic on a laptop without requiring Pi hardware.
- Keep Raspberry Pi–specific hardware code clearly separated from generic / desktop code.
- Provide an image and event processing pipeline that can evolve over time (simple motion detection first, richer analysis later).

## 2. Architectural Overview

### 2.1 Layers

1. **Core Domain & Abstractions (platform-agnostic)**
   - Defines traits and data types for:
     - `FrameSource` – produces video/image frames.
     - `MotionEventSource` – produces motion events (PIR, algorithmic motion detection, etc.).
     - `Frame` and `ImageBuffer` types (wrapping `image` crate types).
     - `MotionEvent` – timestamp, location (camera ID / sensor ID), intensity, metadata.
   - Defines the **processing pipeline** interfaces:
     - `FrameProcessor` – transforms frames or emits derived events.
     - `EventSink` – stores, logs, or forwards events and images.
   - Contains orchestration logic for wiring sources, processors, and sinks.

2. **Platform Adapters (Pi vs Desktop)**
   - Implement the above traits using platform-specific libraries.
   - Clearly separated modules/crates for Raspberry Pi vs generic desktop.

3. **Applications / Binaries**
   - Command-line or daemon binaries that:
     - Select the appropriate platform adapter (Pi or desktop).
     - Configure the pipeline via config files / CLI flags.
     - Run the event loop and manage shutdown.

### 2.2 Cargo Workspace Structure (Planned)

Convert the project into a Cargo workspace with multiple crates (this can be done incrementally):

- `rook_core` (library)
  - Platform-agnostic traits, data structures, and pipeline orchestration.

- `rook_platform_rpi` (library)
  - Raspberry Pi 5 implementations.
  - Depends on Linux/Pi hardware crates (e.g. `gpio-cdev`, possibly `rppal` if you are willing to fork/maintain).

- `rook_platform_desktop` (library)
  - Desktop/Linux implementations.
  - Simulated motion sources and camera sources, no direct GPIO.

- `rook_pipeline` (library; optional split)
  - Reusable image processing and motion detection components.
  - May depend on `image`, `imageproc`, optionally `opencv` for heavier CV.

- `rook_cli` (binary)
  - User-facing CLI / daemon entrypoint.
  - Chooses platform at compile time via Cargo features (`pi5`, `desktop`) or at runtime via configuration when both are available.

In the short term, you can keep a single crate and put modules under `src/core`, `src/platform/pi`, and `src/platform/desktop`, then evolve to the workspace layout later.

## 3. Platform Abstractions

### 3.1 Core Traits (in rook_core)

- `trait FrameSource`
  - `async fn next_frame(&mut self) -> Result<Frame, FrameError>`
  - May also expose frame streams or callbacks.

- `trait MotionEventSource`
  - `fn events(&self) -> impl Stream<Item = MotionEvent>` (async stream over motion events).

- `struct Frame`
  - Wraps an `image::ImageBuffer` plus metadata:
    - Timestamp.
    - Camera ID.
    - Optional camera pose / configuration.

- `struct MotionEvent`
  - Timestamp, source ID (PIR sensor or camera ID), classification ("pir_trigger", "frame_diff", etc.), confidence, optional region of interest.

- `trait FrameProcessor`
  - `async fn process_frame(&mut self, frame: Frame) -> Result<Vec<MotionEvent>, ProcessingError>`
  - Examples: motion detection, noise filtering, region masking.

- `trait EventSink`
  - `async fn handle_event(&mut self, event: MotionEvent)`
  - Implementations: log to stdout, append to files, send over the network, push to a database, etc.

These traits allow you to:
- Swap in Pi vs desktop implementations of `FrameSource` and `MotionEventSource` without touching the pipeline.
- Expand the pipeline with more `FrameProcessor` and `EventSink` implementations over time.

## 4. Raspberry Pi 5 Implementation (rook_platform_rpi)

### 4.1 Hardware Access

**GPIO & Motion Sensors**

- Primary crate: `gpio-cdev` (Linux GPIO character device API)
  - Works on Raspberry Pi 5 and other Linux systems.
  - Good choice for PIR sensors and other digital inputs.
  - Use edge-triggered events (rising/falling edges) to turn PIR signals into an async `MotionEventSource`.

- Optional / legacy: `rppal`
  - Pi-specific peripheral access library that supports Pi 5 but is archived.
  - If you prefer its ergonomics, consider vendoring/forking it to avoid relying on an archived upstream.

**Camera Access (Pi)**

- Preferred crate for unified camera access: `nokhwa` (cross-platform webcam library)
  - On Raspberry Pi 5 with Bookworm, use `nokhwa`'s V4L2 backend to access either:
    - Pi camera modules exposed via the libcamera/V4L2 stack (when configured that way), or
    - USB webcams.
  - Implement `FrameSource` by wrapping `nokhwa` camera sessions.

- Alternative for more control: `v4l` (Rust V4L2 bindings)
  - Implement `FrameSource` directly on top of V4L2 for more detailed control (formats, buffers, performance tuning).

- Libcamera integration options (if needed later):
  - Shell out to `rpicam-*` tools and read frames/metadata via pipes or files.
  - Or use GStreamer with `libcamerasrc` and Rust GStreamer bindings.
  - Keep libcamera-specific integration behind a separate adapter so that the rest of the system stays unaware of these details.

### 4.2 Pi-Specific Modules

Suggested structure under `rook_platform_rpi`:

- `gpio::pir_motion_source` – wraps a PIR sensor GPIO line via `gpio-cdev` and produces `MotionEvent`s.
- `camera::pi_v4l_camera` – V4L2-based implementation of `FrameSource`.
- `camera::pi_nokhwa_camera` – Nokhwa-based `FrameSource`, using the V4L2 backend.
- `platform_config` – mapping from logical camera/sensor IDs to GPIO line numbers, device nodes, and parameters.

The Pi binary (or Pi-specific feature in `rook_cli`) wires:
- One or more `FrameSource`s for cameras.
- One or more `MotionEventSource`s for PIR sensors.
- Appropriate processing pipeline and sinks.

## 5. Desktop / Laptop Implementation (rook_platform_desktop)

On a general Linux laptop, you likely have:
- USB webcam(s) but no PIR/GPIO.

### 5.1 Camera Access (Desktop)

- Use `nokhwa` as the primary camera abstraction.
  - Enables the same `FrameSource` implementation to work on Pi and desktop (with different configuration).

### 5.2 Motion Sources (Desktop)

- Implement motion detection **algorithmically** instead of via hardware sensors:
  - `MotionEventSource` backed by diffing consecutive frames and applying thresholds.
  - Reuse the same `FrameProcessor` code that you will later run on Pi frames.

This lets you:
- Develop and test the capture + processing pipeline entirely on a laptop.
- Switch to hardware PIR on Pi by swapping in the `rook_platform_rpi` implementation of `MotionEventSource`.

## 6. Processing Pipeline Design (rook_pipeline or in rook_core)

### 6.1 Initial Minimal Pipeline

1. **Capture stage**
   - Pull frames from `FrameSource` (camera) at a configurable rate.

2. **Motion detection stage** (simple)
   - Use `image` and/or `imageproc`:
     - Convert frames to grayscale.
     - Compute absolute difference between consecutive frames.
     - Compute mean or max difference over regions.
     - Emit `MotionEvent` if difference exceeds threshold.

3. **Sinks**
   - Log events to stdout or a text file.
   - Optionally save JPEG snapshots around motion events to disk.

### 6.2 Future Enhancements

- Integrate `opencv` bindings for more advanced CV:
  - Background subtraction, optical flow.
  - Object detection or tracking.

- Add network sinks:
  - Send motion events over HTTP, gRPC, or MQTT to another service.

- Configuration-driven pipelines:
  - YAML/TOML/JSON configuration describing which processors and sinks to use.

## 7. Separation of Pi vs Desktop Code

To keep platform-specific code isolated:

- Use separate crates (`rook_platform_rpi`, `rook_platform_desktop`) or, initially, separate modules:
  - `core` contains only traits and shared types.
  - `platform::pi` contains any code that touches GPIO, libcamera/V4L2 specifics, or Pi-only configuration.
  - `platform::desktop` contains desktop-only camera/motion implementations.

- Use Cargo features to select platform at build time:
  - Feature `pi5` enables dependency on `rook_platform_rpi` and its hardware crates.
  - Feature `desktop` enables `rook_platform_desktop`.
  - The `rook_cli` binary uses `cfg(feature = "pi5")` or `cfg(feature = "desktop")` blocks to choose which platform module to wire up.

- Avoid `#[cfg(target_arch = "aarch64")]` or similar for logic; keep that only as a guard in platform crates. Core and pipeline crates compile identically on both Pi and desktop.

## 8. Non-Functional Considerations

- **Concurrency model: threads & channels (preferred)**
  - For hardware-driven signals (GPIO/PIR, libcamera interrupts), prefer dedicated OS threads and channels instead of relying on an async runtime for everything.
  - Use blocking, edge-driven reads from `gpio-cdev` on a dedicated thread; publish `MotionEvent`s to other components via `std::sync::mpsc` or `crossbeam_channel`.
  - Use a small set of worker threads for CPU-bound frame processing; coordinate work with channels (`crossbeam_channel` recommended for performance and selection features).
  - **No async runtimes (no `tokio`)**: avoid `tokio` and other async runtimes entirely. The project should use OS threads and channels for hardware, capture, processing, and sinks. If you later decide to add async-only network components, do so behind well-defined adapters so the core hardware path remains synchronous and channel-driven.

- **Libcamera / C++ capture plan (RPi)**
  - Implement the initial, high-performance capture in C++ using libcamera or libcamera-apps, exposing a minimal `extern "C"` API that: initializes the camera, registers a callback, and provides frames (pointers + metadata) to Rust.
  - In Rust, call the `extern "C"` functions and have a dedicated capture thread that converts/owns frames and sends `Frame` objects down a channel to processors.
  - Consider zero-copy options (shared memory or handing over DMA buffers) later if profiling shows need; start with copying into an `image::DynamicImage` for simplicity.

- **Camera access on desktop (dev)**
  - Use `opencv` or `nokhwa` for desktop camera capture. `opencv` (via `opencv` crate) is still a good option for algorithm prototyping; `nokhwa` or `v4l` are simpler if you only need webcam frames.
  - Provide a `FrameSource` implementation that wraps `opencv`/`nokhwa` and runs in its own thread, sending frames over a channel to the same pipeline used on Pi.

- **GPIO & PIR interrupt handling (RPi)**
  - Use `gpio-cdev`/`libgpiod` to listen for edge events in a blocking/edge-driven fashion on a dedicated thread. Convert rising/falling edges to `MotionEvent::PirTrigger` and send them via channel.
  - Implement debouncing and “stopped for a period” detection in the PIR handler or a small debounce/aggregator component before publishing final events.

- **Error handling & logging**
  - Use `thiserror` for typed errors and `anyhow` for quick error propagation in binaries.
  - Use `tracing` for structured logging (with `env_filter` for debug vs production).

- **Testing strategy**
  - Unit-test `rook_core` and `rook_pipeline` on desktop using simulated `FrameSource` / `MotionEventSource` that inject deterministic frames and events.
  - Integration tests for Pi-specific code should run on hardware; provide small harness binaries that exercise the GPIO handler and the FFI capture layer.

---

This design lets you:
- Develop and iterate on core logic and the processing pipeline entirely on a laptop.
- Swap in Raspberry Pi 5 hardware integrations later with minimal changes to the rest of the code.
- Keep Pi-specific and desktop-specific implementations cleanly separated, while sharing a common set of abstractions and pipeline components.