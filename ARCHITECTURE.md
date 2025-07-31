# Open Headstage Architecture

This document provides an overview of the Open Headstage plugin architecture, intended to guide AI agents and human developers in maintaining and extending the project.

## 1. Overview

Open Headstage is an open-source, multiplatform binaural speaker simulation application. Its primary goal is to provide a high-quality, flexible tool for experiencing stereo audio over headphones as if listening to physical speakers in a well-defined acoustic space.

The project is developed as a **standalone application first**, targeting Linux, Windows, and macOS. This standalone version serves as the primary platform for development, debugging, and use. The core logic is also planned to be bundled as a **CLAP plugin** for use in digital audio workstations (DAWs), although this is considered an experimental, secondary goal.

The application processes stereo audio input, applies binaural spatialization using Head-Related Transfer Functions (HRTFs), and applies a 10-band parametric headphone equalizer.

## 2. Tech Stack

*   **Core Language:** Rust (2024 Edition)
*   **Plugin Framework:** `nih-plug` (Used for its audio processing abstractions and standalone application wrapper)
*   **Plugin Formats:**
    *   **Standalone:** The primary build target.
    *   **CLAP (Experimental):** Bundling is supported, but full DAW compatibility is under development.
    *   **VST3 (Disabled):** VST3 support has been explicitly disabled to avoid the GPLv3 license of the VST3 SDK.
*   **DSP Libraries:**
    *   `rustfft`: For Fast Fourier Transforms, used in the convolution engine.
    *   `realfft`: Wrapper around `rustfft` for real-valued signals.
    *   `rubato`: For high-quality audio resampling of HRTFs.
*   **SOFA HRTF Handling:**
    *   `libmysofa`: C library for loading SOFA files.
    *   `bindgen`: Used in `build.rs` to generate Rust FFI bindings to `libmysofa`.
*   **User Interface:**
    *   `egui` (via `nih_plug_egui`): For creating the graphical user interface.
    *   `egui-file-dialog`: For a self-contained, host-compatible file dialog rendered within `egui`.
*   **Serialization:**
    *   `serde`: For serializing and deserializing application state (e.g., parameters).
*   **Build System:** Cargo
*   **CI/CD:** GitHub Actions

## 3. Module Breakdown

The core logic is primarily located in the `src/` directory.

### 3.1. Main Plugin Logic (`src/lib.rs`)

*   **Responsibility:** Defines the main application structure (`OpenHeadstagePlugin`), its parameters (`OpenHeadstageParams`), and implements the `nih_plug::prelude::Plugin` trait. It orchestrates the interaction between different modules.
*   **Key Structs:**
    *   `OpenHeadstagePlugin`: Holds the application state, including instances of the DSP modules and parameters.
    *   `OpenHeadstageParams`: Defines all user-configurable parameters.
*   **Core Methods:**
    *   `initialize()`: Sets up the application and configures DSP modules based on the buffer configuration.
    *   `process()`: The main audio processing callback.
    *   `editor()`: Provides the GUI editor.
*   **Plugin Export:** Uses `nih_export_standalone!` to create the main application and `nih_export_clap!` for the experimental plugin.

### 3.2. Digital Signal Processing (`src/dsp/`)

*   **`src/dsp/convolution.rs` (ConvolutionEngine)**
    *   **Responsibility:** Performs binaural convolution using HRTFs via an efficient FFT-based method.
*   **`src/dsp/parametric_eq.rs` (StereoParametricEQ, BiquadFilter)**
    *   **Responsibility:** Implements a 10-band stereo parametric equalizer for headphone correction.
    *   **Reference:** `docs/research/EQ Implementation in Rust Research.md`

### 3.3. SOFA HRTF Handling (`src/sofa/`)

*   **`src/sofa/loader.rs` (MySofa)**
    *   **Responsibility:** Provides a safe Rust wrapper around the `libmysofa` C library for loading and interacting with SOFA files.
    *   **Details:** Handles opening SOFA files, extracting HRIR data for specified speaker angles, and ensuring proper resource management.
    *   **FFI:** Relies on `bindgen` (configured in `build.rs`) to generate the raw C bindings.

### 3.4. AutoEQ Parser (`src/autoeq_parser.rs`)

*   **Responsibility:** Parses headphone correction data from AutoEQ project text files to configure the `StereoParametricEQ`.

### 3.5. Build Script (`build.rs`)

*   **Responsibility:** Generates FFI bindings to `libmysofa` using `bindgen` before the rest of the Rust code is compiled.

## 4. Build Process

*   **Prerequisites:**
    *   Rust toolchain (version 1.87.0 or newer).
    *   `libmysofa` development libraries (e.g., `libmysofa-dev` on Debian/Ubuntu).
    *   For UI: A full GTK3 development environment (e.g., `libgtk-3-dev`).
*   **Compilation:**
    *   `cargo build --release`: Compiles the standalone application in release mode.
*   **Output:** The compiled standalone application is located in `target/release/`.
*   **Bundling (for CLAP):**
    *   The `cargo xtask bundle` command is not used. The current practice involves manual creation of the `.clap` directory and copying the compiled `.so` file into it.

## 5. Testing

*   **Primary Method:** The canonical way to test, debug, and benchmark is to compile and run the **standalone application**. This provides a minimal host that connects to a real audio backend like JACK, allowing for high-fidelity, out-of-process integration testing.
*   **Unit Tests:** Located alongside the code in `#[cfg(test)]` blocks. Run with `cargo test`.
*   **CI:** The GitHub Actions workflow in `.github/workflows/rust_ci.yml` runs `cargo fmt`, `cargo clippy`, `cargo build`, and `cargo test`.

## 6. Cross-cutting Concepts

*   **Threading & State Management:** To prevent GUI-related recursion warnings, the application follows a strict, unidirectional threading pattern for tasks initiated from the UI (e.g., loading a file): **GUI -> Audio Thread -> Background**. Results are communicated back to the GUI via polling a shared `Arc<Mutex<T>>`, which decouples the threads.
*   **Crate Structure (`lib.rs` vs. `main.rs`):** The project uses the standard Rust pattern where `src/lib.rs` defines the core library logic and `src/main.rs` defines a binary crate that consumes the library to create the standalone executable. For a detailed explanation, refer to `docs/research/Rust_lib.rs_main.rs_guide.md`.

## 7. Contribution Guidelines for AI Agents

*   **Understand the Goal:** Refer to `TODO.md` and `BUGS.md` before making changes.
*   **Follow Existing Patterns:** Adhere to the module structure and coding style.
*   **Parameter Handling:** Use `nih_plug`'s `Param` types for host-automatable values. For other persistent state (like file paths), use `#[persist]` on a thread-safe container (e.g., `Arc<RwLock<String>>`).
*   **DSP Code:** Prioritize correctness and clarity. Be mindful of real-time constraints in the `process()` loop.
*   **Testing:** Write unit tests for new logic. Ensure `cargo test` passes.
*   **Documentation:** Add Rustdoc comments. Update this document for significant architectural changes.
*   **Error Handling:** Use `Result<T, E>` for functions that can fail.

## 8. License Management

*   **Software Bill of Materials (SBOM):** `LICENSES.md` tracks all third-party dependencies and their licenses.
*   **Primary Project License:** Apache License, Version 2.0.
*   **GPLv3 Consideration:** VST3 support is disabled. If re-enabled, its GPLv3 license would apply to any distributed VST3 binary.

## 9. Future Vision

This section outlines potential future directions for the project.

*   **Surround Sound to Stereo Mixdown:** Extend the engine to support 5.1 and 7.1 surround sound input, rendering a high-quality binaural mixdown.
*   **Dynamic Head Tracking:** Integrate dynamic head tracking to adjust the binaural rendering in real-time for a more immersive experience.

---
This document is a living guide. It should be updated as the project evolves.
