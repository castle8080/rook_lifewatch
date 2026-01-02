# Coding Guidelines

Purpose: concise, practical rules for writing and reviewing code in this repository.

- **Avoid `unwrap` / `expect`:** never use `unwrap` or `expect` in library or production code. Return `Result` types and propagate errors using `?`. Reserve `unwrap`/`expect` only for quick prototypes or tests where failure is impossible.

- **Error types:** use `thiserror` for typed errors in libraries and `anyhow` for quick propagation in binaries. Convert lower-level errors into domain-specific error variants at crate boundaries.

- **No async runtimes:** do not add or rely on `tokio` (or other async runtimes) in core or platform code. The project uses OS threads + channels for hardware interrupt handling, capture, and processing. If an async-only network component is needed later, add it behind a clear adapter boundary so the core remains synchronous.

- **Channels for communication:** prefer `crossbeam_channel` for internal event/worker communication (or `std::sync::mpsc` if simplicity is required). Standard patterns:
  - One dedicated producer thread per hardware source (GPIO, capture). Producer sends messages to channels.
  - One or more worker threads consume frames/events from channels for processing.
  - Keep channel types explicit (e.g., `Frame`, `MotionEvent`) and avoid `Box<dyn Any>` messages.

- **Platform separation:** keep Raspberry Pi–specific code in `platform::rpi` (or a separate crate) and desktop/dev code in `platform::desktop`. Core traits and pipeline code must not depend on platform crates.

- **FFI and unsafe:** when writing FFI (C/C++) wrappers (e.g., libcamera capture), minimize `unsafe` to small wrapper modules. Clearly document ownership, lifetimes, and who is responsible for freeing memory. Prefer copying into safe Rust containers until zero-copy is proven necessary.

- **Logging & observability:** use `tracing` for structured logs. Include enough context (source IDs, timestamps) in events to make debugging easier.

- **Formatting & linting:** run `rustfmt` and `clippy` before commits. Fix Clippy lints unless there is a documented, justified exception.

- **Testing:** provide unit tests for core logic using deterministic, simulated `FrameSource` and `MotionEventSource`. For Pi-specific code, provide small integration harnesses that can be run on hardware.

- **Small, focused modules:** keep modules small and focused. Public APIs should be documented with doc comments and examples when helpful.

- **Naming & style:** avoid one-letter variable names (except in short loops), prefer descriptive names, and wrap crate-level types in clear names (e.g., `FrameSource`, `MotionEvent`). Use `snake_case` for functions and `CamelCase` for types.

- **Performance & profiling:** measure before optimizing. For capture paths, prefer a dedicated capture thread that hands frames via channels to processors. Consider zero-copy or shared memory only after profiling demonstrates a bottleneck.

- **Concurrency safety:** avoid global mutable state. When shared state is necessary, prefer `Arc<Mutex<T>>` or `Arc<RwLock<T>>` with clear ownership and short lock spans.

- **Documentation:** document assumptions about hardware (sampling rates, debounce intervals) and the expected semantics of events (e.g., PIR triggers vs. motion-stop events).
