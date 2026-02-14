=# Rook LifeWatch Copilot Instructions

## Project Overview
Trail-cam-like software for Raspberry Pi that captures frames, detects motion, performs object detection, and stores interesting frames. Multi-language monorepo with Rust (backend/frontend), C++ (camera), and Python (build scripts).

## Architecture

### Component Pipeline (Producer-Consumer Pattern)
The daemon uses crossbeam channels for concurrent image processing:
1. **MotionWatcher** (Producer) → captures frames → sends `ImageProcessingEvent` to channel
2. **ImageDetector** (Consumer/Producer) → receives events → runs object detection → sends to next channel  
3. **ImageStorer** (Consumer) → receives events → persists to SQLite and disk

See [rook_lw_daemon/src/app/app.rs](rook_lw_daemon/src/app/app.rs) for pipeline setup with bounded channels (backpressure).

### Key Components
- **rook_lw_daemon** - Main service: motion detection, object detection (YOLOv4-tiny via OpenCV), image storage
- **rook_lw_admin** - Actix-web REST API server for device management
- **rook_lw_admin_fe** - Leptos (Rust→WASM) SPA frontend served by admin
- **rook_lw_models** - Shared Rust data models (ImageInfo, Detection, SearchOptions)
- **rook_lw_image_repo** - Repository pattern for SQLite/filesystem access
- **rook_lw_libcamera_capture** - C++ library wrapping libcamera (linked via FFI to daemon)
- **rook_lw_mule** - Tauri desktop app (experimental)

### Cross-Component Communication
- Frontend → Backend: REST APIs via gloo_net
- Components share types from `rook_lw_models` (serde-serialized)
- Daemon → Storage: Repository pattern abstracts SQLite + filesystem

## Build System

### Python-Based Build Orchestration
Each project has `make.py` with targets: `clean`, `build`, `test`, `install`, `run`

```bash
# Build/install all from root
./scripts/build-install.py

# Individual project
cd rook_lw_daemon && ./make.py build install

# Frontend development server (with proxy to backend)
cd rook_lw_admin_fe && ROOK_LW_PROXY=http://localhost:8080/api ./make.py run
```

**Install Target:** Copies artifacts to `../dist/` (binaries → `dist/bin/`, web assets → `dist/www/`)

### Feature Flags
- `rook_lw_daemon` has `libcamera` feature (default on Linux, disabled on Windows)
- Build system auto-detects platform and adjusts project list (see [scripts/build-install.py](scripts/build-install.py))

### C++ Build (libcamera)
```bash
cd rook_lw_libcamera_capture
cmake --preset default
cmake --build --preset default
cmake --install build/default --prefix "$PWD/../dist"
```

Cross-compile for Pi:
```bash
. ./setup-cross-compiler-env.sh  # Sets ROOK_PI_SYSROOT
cmake --preset rpi-arm64
```

### Frontend (Leptos/WASM)
Uses **trunk** for building/serving:
```bash
cd rook_lw_admin_fe
./make.py init_dev  # Installs trunk + wasm32 target
./make.py build     # Outputs to dist/, copied to ../dist/www/admin/
```

## Development Workflows

### Run System Locally
```bash
# Terminal 1: Backend
dist/bin/start_rook_lw_daemon.sh

# Terminal 2: Admin API
dist/bin/start_rook_lw_admin.sh  # Serves on http://localhost:8080

# Terminal 3: Frontend dev (hot reload)
cd rook_lw_admin_fe && ./make.py run  # Serves on http://localhost:8081
```

### Configuration
Daemon config: [rook_lw_daemon/config/rook_lw_daemon.toml](rook_lw_daemon/config/rook_lw_daemon.toml)
- Motion detector types: `yplane_motion_percentile`, `yplane_boxed_average`
- Object detector: OpenCV DNN with configurable model paths/thresholds
- Paths: `image_directory`, `database_path`

### Logging
All services use **tracing** crate. Initialize with `app::init_tracing()` in main.

## Code Conventions

### Error Handling
- Custom result types: `RookLWResult<T>` (see [rook_lw_daemon/src/error.rs](rook_lw_daemon/src/error.rs))
- Implements `From` for common errors (io, channels, serde, etc.)

### Producer-Consumer Pattern
Modules in `rook_lw_daemon/src/prodcon/`:
- `ProducerTask`: Spawns thread, publishes to channel
- `ConsumerTask`: Spawns thread, reads from channel
- Tasks implement both traits to chain processing stages

### Data Models
Structs in `rook_lw_models` use `#[derive(Serialize, Deserialize, Clone)]` for cross-boundary use.

### FFI Integration
Daemon links C++ libcamera library via `build.rs`:
- Checks for `CARGO_FEATURE_LIBCAMERA` env var
- Uses `pkg-config` to find system libcamera
- Links static archive from `../dist/lib/librook_lw_libcamera_capture.a`

## Common Tasks

### Add New API Endpoint (Admin)
1. Add handler in `rook_lw_admin/src/controllers/`
2. Register route in app setup (likely in `main.rs` or `app_state.rs`)
3. Add model to `rook_lw_models/src/` if needed
4. Update frontend service in `rook_lw_admin_fe/src/services/`

### Add Processing Stage to Pipeline
1. Create task in `rook_lw_daemon/src/tasks/` implementing `ProducerTask`/`ConsumerTask`
2. Wire channels in [rook_lw_daemon/src/app/app.rs](rook_lw_daemon/src/app/app.rs) `App::run()`
3. Add configuration to [config/rook_lw_daemon.toml](rook_lw_daemon/config/rook_lw_daemon.toml)

### Update ML Model
```bash
./scripts/download-model-files.py  # Fetches YOLOv4 weights/config
# Configure paths in rook_lw_daemon.toml
```

## Raspberry Pi Specifics
- Cross-compilation sysroot scripts in `rook_lw_create_sysroot/` (experimental, linking issues)
- libcamera-specific code conditionally compiled behind feature flag
- OpenCV used as fallback on non-Pi platforms

## Key Files to Reference
- [doc/system_summary.md](doc/system_summary.md) - Architecture overview
- [doc/notes/dev_notes.md](doc/notes/dev_notes.md) - Feature roadmap and tech debt
- [scripts/build-install.py](scripts/build-install.py) - Build orchestration entry point
- [rook_lw_daemon/src/prodcon/](rook_lw_daemon/src/prodcon/) - Producer-consumer abstractions
