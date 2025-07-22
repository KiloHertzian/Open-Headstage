# Open Headstage Architecture

This document provides an overview of the Open Headstage plugin architecture, intended to guide AI agents and human developers in maintaining and extending the project.

## 1. Overview

Open Headstage is an open-source binaural speaker simulation plugin. Its primary goal is to recreate the experience of listening to stereo audio through physical speakers in a defined acoustic space, but over headphones. It aims to be a high-quality, flexible tool, initially targeting Linux-based audio professionals and enthusiasts.

The plugin processes stereo audio input, applies binaural spatialization using Head-Related Transfer Functions (HRTFs), and optionally applies headphone equalization.

## 2. Tech Stack

*   **Core Language:** Rust (2021 Edition)
*   **Plugin Framework:** `nih-plug` (Rust-based framework for audio plugins)
*   **Plugin Formats:**
    *   CLAP (primary target)
    *   VST3 (secondary target)
*   **DSP Libraries:**
    *   `rustfft`: For Fast Fourier Transforms, used in the convolution engine.
    *   `realfft`: Wrapper around `rustfft` for real-valued signals.
    *   `rubato`: For high-quality audio resampling (primarily for HRTFs).
*   **SOFA HRTF Handling:**
    *   `libmysofa`: C library for loading SOFA files.
    *   `bindgen`: Used in `build.rs` to generate Rust FFI bindings to `libmysofa`.
*   **User Interface (Optional Feature):**
    *   `egui` (via `nih_plug_egui`): For creating the graphical user interface.
    *   `rfd` (Rust File Dialogs): For native file dialogs (e.g., loading SOFA files).
*   **Serialization:**
    *   `serde`: For serializing and deserializing plugin state (e.g., parameters).
*   **Build System:** Cargo (Rust's package manager and build tool)
*   **CI/CD:** GitHub Actions (see `.github/workflows/rust_ci.yml`)

## 3. Module Breakdown

The core logic is primarily located in the `src/` directory.

### 3.1. Main Plugin Logic (`src/lib.rs`)

*   **Responsibility:** Defines the main plugin structure (`OpenHeadstagePlugin`), its parameters (`OpenHeadstageParams`), and implements the `nih_plug::prelude::Plugin` trait. It orchestrates the interaction between different modules (DSP, SOFA, UI).
*   **Key Structs:**
    *   `OpenHeadstagePlugin`: Holds the plugin state, including instances of the DSP modules and parameters.
    *   `OpenHeadstageParams`: Defines all user-configurable parameters using `nih_plug`'s parameter system.
*   **Core Methods:**
    *   `initialize()`: Sets up the plugin, loads initial resources (like SOFA files if specified), and configures DSP modules based on the buffer configuration (sample rate, block size).
    *   `process()`: The main audio processing callback. It receives input audio, applies EQ, then convolution, and applies output gain. It also handles parameter smoothing.
    *   `editor()`: (If UI feature is enabled) Provides the GUI editor.
    *   `params()`: Exposes the plugin's parameters to the host.
*   **Plugin Export:** Uses `nih_export_clap!` and `nih_export_vst3!` to make the plugin available in these formats.

### 3.2. Digital Signal Processing (`src/dsp/`)

This directory contains modules for various audio processing tasks.

*   **`src/dsp/convolution.rs` (ConvolutionEngine)**
    *   **Responsibility:** Performs binaural convolution using HRTFs. It should implement a 4-path convolution for anechoic HRTFs (L->L, L->R, R->L, R->R).
    *   **Details:** Likely uses an FFT-based method (Overlap-Add or Overlap-Save) for efficiency. Manages impulse responses and processes audio blocks.
    *   **Key Structs/Methods:** `ConvolutionEngine`, `process_stereo_sample()` (or `process_block()`), methods to set/update impulse responses.

*   **`src/dsp/parametric_eq.rs` (StereoParametricEQ, BiquadFilter)**
    *   **Responsibility:** Implements a multi-band stereo parametric equalizer. Used for headphone correction.
    *   **Details:** Consists of `BiquadFilter` (implementing filter types like Peak, LowShelf, HighShelf based on Audio EQ Cookbook formulas) and `StereoParametricEQ` (managing a bank of these filters for stereo processing).
    *   **Key Structs/Methods:** `StereoParametricEQ`, `BiquadFilter`, `update_band_coeffs()`, `process_stereo_sample()`.

*   **`src/dsp/mod.rs`**
    *   **Responsibility:** Declares the modules within the `dsp` directory, making them accessible.

### 3.3. SOFA HRTF Handling (`src/sofa/`)

*   **`src/sofa/loader.rs` (MySofa)**
    *   **Responsibility:** Provides a safe Rust wrapper around the `libmysofa` C library for loading and interacting with SOFA files.
    *   **Details:** Handles opening SOFA files, extracting HRIR data for specified speaker angles, potentially managing resampling of HRIRs to the plugin's sample rate, and ensuring proper resource management (e.g., closing SOFA files via the `Drop` trait).
    *   **Key Structs/Methods:** `MySofa`, `open()`, methods to get HRIRs (e.g., `getfilter_float` equivalent).
    *   **FFI:** Relies on `bindgen` (configured in `build.rs`) to generate the raw C bindings.

*   **`src/sofa/mod.rs`**
    *   **Responsibility:** Declares the modules within the `sofa` directory.

### 3.4. AutoEQ Parser (`src/autoeq_parser.rs`) (If Present)

*   **Responsibility:** Parses headphone correction data from AutoEQ project text files.
*   **Details:** Reads filter parameters (Fc, Q, Gain, Type) from the specified file format and makes them available, likely to configure the `StereoParametricEQ`.
*   **Key Structs/Methods:** `AutoEqProfile`, `parse_autoeq_file_content()`.

### 3.5. Build Script (`build.rs`)

*   **Responsibility:** Generates FFI bindings to `libmysofa` using `bindgen` before the rest of the Rust code is compiled.
*   **Dependencies:** `bindgen`.
*   **Process:**
    1.  Locates `libmysofa` headers (requires `libmysofa-dev` or equivalent to be installed on the system).
    2.  Runs `bindgen` to generate `bindings.rs` (typically in `OUT_DIR`).
    3.  The generated bindings are then included by `src/sofa/loader.rs`.

## 4. Build Process

*   **Prerequisites:**
    *   Rust toolchain (see `rust-toolchain.toml` or CI setup for version).
    *   `libmysofa` development libraries (e.g., `libmysofa-dev` on Debian/Ubuntu).
    *   For UI: A full GTK3 development environment. The easiest way to install this on Debian/Ubuntu is with the `libgtk-3-dev` package, which includes `glib`, `atk`, `pango`, `gdk`, and other necessary libraries.
*   **Compilation:**
    *   `cargo build`: Compiles the plugin in debug mode.
    *   `cargo build --release`: Compiles the plugin in release mode (optimized).
*   **Output:** The compiled plugin (e.g., `.clap`, `.vst3`) will be located in `target/debug/` or `target/release/`.
*   **Bundling (for distribution):**
    *   `nih-plug` often uses `cargo xtask bundle <plugin_name>` for creating distributable bundles, but this project might have its own specifics or rely on manual copying post-build. Check `README.md` or `nih-plug` documentation.

## 5. Testing

*   **Unit Tests:**
    *   Located alongside the code (e.g., in `src/dsp/convolution.rs` within a `#[cfg(test)] mod tests { ... }` block).
    *   Run with `cargo test`.
*   **Integration Tests:**
    *   Could be in the `tests/` directory at the root of the project.
    *   Run with `cargo test`.
*   **CI:** GitHub Actions workflow in `.github/workflows/rust_ci.yml` runs:
    *   `cargo fmt --check` (formatting)
    *   `cargo clippy` (linting)
    *   `cargo build`
    *   `cargo test`
    *   *(Note: Current CI setup in `BUGS.md` indicates issues with `nih-plug` compatibility, so full testing in CI might be impaired until resolved.)*

## 6. Current Status & Known Issues

*   Refer to `README.md` (Roadmap section) and `BUGS.md` for the most up-to-date information on what parts are working, in progress, or have known limitations.
*   **Key Blocker (as of last update):** `nih-plug` version compatibility with the CI environment's Rust compiler, and git fetching issues for specific `nih-plug` revisions. This primarily affects CI builds and testing; local development with a compatible Rust version is expected.

## 7. Contribution Guidelines for AI Agents

*   **Understand the Goal:** Before making changes, ensure you understand the specific issue or feature request. Refer to GitHub issues if applicable.
*   **Follow Existing Patterns:**
    *   Adhere to the module structure outlined above.
    *   Mimic the style and error handling patterns found in the existing codebase.
    *   Use `nih_log!` for plugin-specific logging.
*   **Dependencies:**
    *   If adding new dependencies, add them to `Cargo.toml`. Prefer widely used and well-maintained crates.
    *   For FFI, ensure `build.rs` is updated accordingly if new C libraries are integrated.
*   **Parameter Handling:**
    *   All user-configurable settings should be exposed as parameters in `OpenHeadstageParams` using `nih_plug`'s parameter types.
    *   Ensure parameters have appropriate ranges, smoothing, units, and string formatting.
    *   Persist parameters that should be saved with the session (e.g., `#[persist = "param_name"]`).
*   **DSP Code:**
    *   Prioritize correctness and clarity.
    *   Optimize for performance only after functionality is verified, especially in the `process()` loop.
    *   Be mindful of real-time audio processing constraints (avoid allocations, heavy computations, or blocking operations in `process()`).
*   **Testing:**
    *   Write unit tests for new DSP logic or complex functions.
    *   Ensure `cargo test` passes after your changes.
*   **Documentation:**
    *   Add Rustdoc comments (`///`) to new public functions, structs, and modules.
    *   Update this `architecture.md` document if you make significant architectural changes (e.g., add a major new module, change how core components interact).
    *   Update `CHANGELOG.md` with a summary of your changes.
*   **Error Handling:**
    *   Use `Result<T, E>` for functions that can fail. Define specific error types where appropriate (e.g., `SofaError` in `src/sofa/loader.rs`).
*   **UI (if working on UI features):**
    *   Ensure UI elements are correctly linked to their corresponding parameters in `OpenHeadstageParams`.
    *   Follow `egui` best practices.
*   **Commit Messages:**
    *   Write clear and concise commit messages, explaining the "what" and "why" of your changes.
*   **Updating this Document:**
    *   If you add a new major module (e.g., a new DSP component, a different type of resource loader).
    *   If you significantly change the interaction between existing core modules.
    *   If the tech stack changes (e.g., a new core library is introduced).
    *   If build or test procedures are fundamentally altered.
    *   Minor changes or bug fixes within existing modules usually don't require updating this document, but ensure code comments and the `CHANGELOG.md` are up to date.

## 8. Documentation

*   **CLAP Plugin Development Guide:** For detailed information on CLAP plugin development using `nih-plug`, SOFA integration, DSP implementation, and UI design, refer to the `CLAP Plugin Development Documentation.md` file in the project root. This document serves as a comprehensive guide for the technical implementation aspects of the plugin.

## 9. License Management

*   **Software Bill of Materials (SBOM):** This project tracks its third-party dependencies and their licenses in the `LICENSES.md` file. This file should be reviewed for compliance and updated whenever new dependencies are added.
*   **Primary Project License:** The license for the Open Headstage project itself is located in the `LICENSE` file in the project root.
*   **GPLv3 Consideration:** As noted in `LICENSES.md`, the `vst3-sys` crate is licensed under GPLv3. This requires that any distributed binary containing the VST3 version of this plugin must also have its corresponding source code made available under the GPLv3.

## 10. Future Vision

This section outlines potential future directions for the project, building upon the current architecture.

*   **Surround Sound to Stereo Mixdown:** A significant future goal is to extend the engine to support 5.1 and 7.1 surround sound input, using the convolution engine to render a high-quality binaural mixdown for headphones. This would involve:
    *   Expanding the plugin's input channel configuration.
    *   Managing a full set of HRTFs for each surround channel (e.g., L, R, C, LFE, Ls, Rs, etc.).
    *   Extending the `ConvolutionEngine` to handle more than two input channels.

---

This document is a living guide. It should be updated as the project evolves.