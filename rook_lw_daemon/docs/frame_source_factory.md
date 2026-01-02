# Frame Source Factory

The frame source factory provides a flexible way to create frame sources with both compile-time and runtime selection capabilities.

## Features

### Compile-Time Selection

Feature flags control which frame sources are compiled into the binary:

- `libcamera` - Enable libcamera support
- `opencv` - Enable OpenCV support
- `default` - Both libcamera and opencv are enabled by default

### Runtime Selection

Even when multiple sources are compiled in, the factory will:
1. Try each available source in preference order
2. Skip sources that fail runtime initialization
3. Return the first successfully initialized source

## Usage Examples

### Basic Usage (Default Preference)

```rust
use rook_life_watch::implementation::factory::FrameSourceFactory;

let frame_source = FrameSourceFactory::create()
    .expect("Failed to create frame source");
```

Default preference order: libcamera → opencv

### Custom Preference Order

```rust
use rook_life_watch::implementation::factory::{
    FrameSourceFactory,
    FrameSourcePreference
};

// Prefer OpenCV over libcamera
let frame_source = FrameSourceFactory::create_with_preference(
    FrameSourcePreference::PreferOpenCV
).expect("Failed to create frame source");
```

### Query Available Sources

```rust
use rook_life_watch::implementation::factory::FrameSourceFactory;

let sources = FrameSourceFactory::available_sources();
println!("Compiled-in sources: {:?}", sources);
```

## Building with Different Features

### Default (All Features)

```bash
cargo build
```

### Only libcamera

```bash
cargo build --no-default-features --features libcamera
```

### Only OpenCV

```bash
cargo build --no-default-features --features opencv
```

### Specific Combination

```bash
cargo build --no-default-features --features "libcamera,opencv"
```

## Architecture

### Factory Pattern

The `FrameSourceFactory` uses:
- **Conditional compilation** (`#[cfg(feature = "...")]`) to only include enabled sources
- **Runtime initialization** via `try_new()` methods that can fail gracefully
- **Preference ordering** to try sources in user-specified order

### Implementation Structure

```
implementation/
├── factory.rs           # Factory with feature-gated source creation
├── libcamera/
│   └── mod.rs          # LibCameraFrameSource with try_new()
└── opencv/
    └── mod.rs          # OpencvFrameSource with try_new()
```

### Error Handling

The factory provides detailed error types:

- `FactoryError::NoImplementationAvailable` - No compiled sources could be initialized
- `FactoryError::InitializationFailed(String)` - Specific initialization error from a source

## Extending the Factory

To add a new frame source implementation:

1. Add a feature flag in `Cargo.toml`:
   ```toml
   [features]
   mynewsource = []
   ```

2. Create implementation module:
   ```rust
   // implementation/mynewsource/mod.rs
   pub struct MyNewFrameSource;
   
   impl MyNewFrameSource {
       pub fn try_new() -> Result<Self, FrameError> {
           // Runtime check for availability
           Ok(Self)
       }
   }
   
   impl FrameSource for MyNewFrameSource {
       fn next_frame(&mut self) -> Result<Frame, FrameError> {
           // Implementation
       }
   }
   ```

3. Add to `implementation/mod.rs`:
   ```rust
   #[cfg(feature = "mynewsource")]
   pub mod mynewsource;
   ```

4. Add to factory's `try_create` method and preference enum.

## Design Rationale

### Why Feature Flags?

- **Reduced binary size**: Only include needed implementations
- **Platform-specific builds**: Some sources may not be available on all platforms
- **Dependency management**: Avoid pulling in unnecessary dependencies

### Why Runtime Selection?

- **Hardware availability**: A camera might not be present or accessible
- **Fallback behavior**: Try multiple sources until one works
- **User preference**: Allow users to prefer one implementation over another
- **Testing**: Different sources can be tried without recompilation

### Why Both?

The combination provides maximum flexibility:
- **Build time** eliminates code for unsupported platforms
- **Runtime** handles actual hardware/software availability

This two-tier approach ensures robust operation across different environments while maintaining minimal binary size.
