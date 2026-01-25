# System Summary: rook_lifewatch

## Overview
The rook_lifewatch project is a modular, multi-language system for edge-based image capture, analysis, and management. It integrates modern web technologies, embedded systems, and machine learning.

---

## Core Concepts & Technologies

### Languages & Core Technologies
- **Rust**: Main language for backend services, frontend (via WASM), and shared models.
- **Python**: Used for build automation and orchestration scripts.
- **C++**: Handles low-level camera capture and device integration.
- **Shell Scripting**: For system setup, cross-compilation, and device management.

### Frontend
- **Leptos**: Rust-based reactive web UI framework (SPA).
- **WASM (WebAssembly)**: Rust code compiled to run in the browser.
- **gloo_net**: HTTP requests from frontend to backend.
- **serde/serde_qs**: Serialization for data exchange.

### Backend
- **Actix Web Framework**: Used for REST APIs and backend services.
- **rusqlite & r2d2_sqlite**: SQLite database access and connection pooling.
- **tracing**: Structured logging for observability.
- **serde/serde_json**: Data serialization for APIs and DB.

### Data Models & Patterns
- **Strongly Typed Structs**: For images, detections, search options, etc.
- **Repository Pattern**: Abstracts database access.
- **Crossbeam Channels & Pipeline Architecture**: The backend (especially rook_lw_daemon) uses crossbeam channels to implement a concurrent image processing pipeline. This enables efficient, thread-safe communication between producer and consumer tasks for motion detection, object detection, and other processing stages.

### Build & Project Structure
- **Cargo**: Rust’s package manager and build tool.
- **CMake**: For C++ components.
- **Python Scripts**: For orchestration and cross-language build steps.
- **Monorepo**: Multiple related projects (admin, daemon, models, image repo, camera capture) in one workspace.

### Machine Learning Integration
- **On-device ML Model**: Runs inference on edge devices (e.g., Raspberry Pi) using the YOLOv4-tiny object detection model through OpenCV's DNN module.
- **Model Management**: Scripts for downloading, installing, and updating model files.
- **Real-time Image Analysis**: Efficient resource usage for ML inference and coordination with camera/backend services.

### System Integration
- **Device Management**: Shell scripts for sysroot setup, image mounting, and hardware control.
- **Configuration**: TOML files for service configuration.

### Key Libraries/Crates
- **chrono**: Date/time handling.
- **leptos, leptos_router**: SPA routing and UI.
- **gloo_net**: Web requests in WASM.
- **serde, serde_json, serde_qs**: Serialization.
- **rusqlite, r2d2_sqlite**: Database.
- **tracing**: Logging.

---

## General Architecture
- **Frontend (rook_lw_admin_fe)**: SPA in Rust/Leptos, communicates with backend via REST APIs.
- **Backend (rook_lw_admin, rook_lw_image_repo, rook_lw_daemon)**: Rust services for admin, image storage, and device control.
- **Models (rook_lw_models)**: Shared Rust crate for data types.
- **Camera Integration (rook_lw_libcamera_capture)**: C++ for direct hardware access, exposed to Rust via FFI.
- **ML Inference**: On-device model for real-time detection/classification.

---
