# Known Issues & Development Status

This document tracks known bugs, limitations, and the overall development status of the Open Headstage project.

---

*There are no known critical bugs at this time.*

---

## Resolved Issues & Lessons Learned

### Environment/Toolchain Incompatibility with `nih-plug` (Resolved)

*   **Original Problem:** The project failed to build in the CI environment because the available Rust compiler (1.75.0) was too old for the version of `nih-plug` being used, which required Rust 1.80+. Attempts to pin `nih-plug` to an older version failed due to git fetching errors in the CI.
*   **Resolution:** The CI environment's Rust toolchain was updated to a newer version (1.87.0). This immediately resolved the compilation failures.
*   **Lesson Learned:**
    1.  **Toolchain is Key:** The Rust compiler version is a critical dependency, especially when using cutting-edge libraries that pull from git repositories.
    2.  **CI Configuration:** The `rust-toolchain` file or the CI workflow script (`.github/workflows/rust_ci.yml`) is the canonical source for the required Rust version and must be kept in sync with the project's needs.
    3.  **Clear Error Messages:** The final build success was only possible after getting clear error messages. When builds fail, identifying the root cause (e.g., compiler version vs. code error) is the most important step.

### Cargo Feature Resolution for UI Dependencies (Resolved)

*   **Original Problem:** The project failed to compile with persistent errors related to `nih-plug` features, specifically the `gui` feature. The error message "the package `open-headstage` depends on `nih_plug`, with features: `gui` but `nih_plug` does not have these features" was misleading and caused a series of incorrect fix attempts, including using local path dependencies which introduced workspace conflicts.
*   **Resolution:**
    1.  The local `nih-plug` repository clone was removed to eliminate path and workspace conflicts.
    2.  The `Cargo.toml` dependencies for `nih_plug` and `nih_plug_egui` were changed from local paths to specific versions from `crates.io`.
    3.  The `gui` feature was correctly enabled directly on the `nih_plug` dependency line in `Cargo.toml`: `nih_plug = { version = "0.5.1", features = ["gui"] }`.
    4.  The `ui` feature in the local `[features]` section was corrected to only include the optional UI-specific crates: `ui = ["nih_plug_egui", "rfd"]`.
*   **Lesson Learned:**
    1.  **`crates.io` is the Source of Truth:** When facing intractable dependency or feature issues with a local git checkout, revert to a stable version from `crates.io`. This simplifies the build environment and eliminates potential inconsistencies in the local repository's `Cargo.toml`.
    2.  **Enabling Dependency Features:** To enable a feature for a dependency, it **must** be done on the dependency's line in `Cargo.toml`. The `[features]` section of the current crate is for defining its *own* features, which can then be used to conditionally enable dependencies or code via `cfg` attributes. The two are not interchangeable.
    3.  **Deconstruct the Error:** The error message was confusing. While it said `nih_plug` didn't have the `gui` feature, the real issue was the complex interaction of local paths, workspace conflicts, and an incorrectly defined local feature. Simplifying the situation (by removing the local path) was key to revealing the true, much simpler, solution.

### `nih-plug` Parameter System & Threading (Resolved)

*   **Original Problem:** A series of compilation errors arose from a misunderstanding of the `nih-plug` parameter system and thread-safety requirements.
    *   `StringParam` was used for the SOFA file path, but this parameter type does not exist in the current version of `nih-plug`.
    *   The `task_executor` function signature was incorrect, causing ownership and lifetime errors when trying to share plugin state (`MySofa`, `ConvolutionEngine`) with the background thread.
    *   Attempts to `Clone` `MySofa` failed because it contains a raw pointer and is not designed to be cloned.
*   **Resolution:**
    1.  **Replaced `StringParam`:** The non-existent `StringParam` was replaced with a `#[persist]` field: `pub sofa_file_path: Arc<RwLock<String>>`. This correctly handles persistent, non-parameter state.
    2.  **Corrected Thread Safety:** To safely share state between the audio processor, the UI, and the background task executor, `Arc<RwLock<>>` (for the SOFA path string) and `Arc<Mutex<>>` (for the `MySofa` loader) were used. This ensures safe interior mutability across threads.
    3.  **Fixed `task_executor`:** The `task_executor` was corrected to return a closure that captures the necessary thread-safe state (`Arc`-wrapped handles), resolving the ownership issues.
    4.  **Made `ConvolutionEngine` Clonable:** The `ConvolutionEngine` struct was made clonable (`#[derive(Clone)]`) to allow it to be moved into the background closure, although this was later refactored to avoid unnecessary cloning.
*   **Lesson Learned:**
    1.  **Parameters vs. Persistent State:** Use `nih-plug`'s `Param` types (`FloatParam`, etc.) only for values that are automated and controlled by the host. For other state that needs to be saved, like file paths or editor state, use `#[persist]` on a thread-safe container like an `Arc<RwLock<T>>`.
    2.  **Threading is Explicit:** When sharing state between the UI/background threads and the real-time audio thread, standard Rust thread-safety patterns (`Arc`, `Mutex`, `RwLock`) are required. The plugin's main struct fields must be designed for this from the start if they are to be shared.
    3.  **Consult the Source:** When a type or feature seems to be missing or causes confusing errors, the quickest path to a solution is to inspect the library's source code (`src/params.rs` in this case) to see the available types and their intended usage. `grep` is an invaluable tool for this.
    4.  **Compiler Errors Guide Architecture:** The series of ownership, lifetime, and      trait implementation errors were not just syntax problems; they were pointing to a          fundamental architectural issue in how state was being shared. Resolving them required      changing the data structures themselves, not just the function calls.

### CI Build Failures due to Missing System Dependencies (Resolved)

*   **Original Problem:** After successfully compiling locally, the CI build failed repeatedly with `pkg-config` errors. The build script could not find system libraries required by the GUI toolkit, such as `glib-2.0`, `atk`, and `gdk-3.0`.
*   **Resolution:** The issue was resolved by adding the main GTK3 development meta-package, `libgtk-3-dev`, to the `apt-get install` command in the `.github/workflows/rust_ci.yml` file. This single package automatically pulled in all the necessary dependencies (`glib`, `atk`, `gdk`, `pango`, etc.) that the UI crates rely on.
*   **Lesson Learned:**
    1.  **Install Comprehensive Dev Packages:** When dealing with large C/C++-based libraries or toolkits like GTK, it is more robust to install the primary development package (e.g., `libgtk-3-dev`) rather than trying to install each required sub-library individually. This avoids a "whack-a-mole" scenario where fixing one missing dependency reveals another.
    2.  **Local vs. CI Environments:** A successful local build does not guarantee a successful CI build. The CI environment is clean and will expose any missing system dependencies that might already be installed on a local development machine. The CI configuration must be treated as the canonical list of build requirements.

### GUI File Dialog Failures (Crash on Click)

*   **Original Problem:** The plugin, when loaded in a Qt-based host like Carla, would crash instantly upon clicking the "Select SOFA File" button.
*   **Root Cause Analysis:** The crash was caused by using the `rfd` (Rust File Dialog) crate directly within the plugin's GUI. `rfd` attempts to open a native GTK file dialog, which conflicts with the host's Qt event loop, leading to a segmentation fault. This is a fundamental incompatibility between GUI toolkits.
*   **Failed Solutions & Lessons:**
    1.  **Incorrect API Usage:** An attempt to use `nih-plug`'s host-based file dialogs failed because of incorrect API knowledge. The methods I tried to call (`show_open_dialog`, `show_file_dialog` on `AsyncExecutor`) do not exist. **Lesson:** The `nih-plug` book and official examples are the only reliable source for API usage. Do not guess or assume.
    2.  **Dependency Version Hell:** An attempt to use the `egui-file-dialog` crate failed due to a cascade of dependency version conflicts. The version of `egui-file-dialog` used an older version of `egui` than `nih-plug-egui` did, leading to dozens of compiler errors. **Lesson:** The `nih-plug` ecosystem is very sensitive to dependency versions. Before adding any new `egui`-related crate, use `cargo tree` to verify that it uses the *exact same version* of `egui` as `nih-plug-egui`. If not, it is incompatible.
    3.  **Misunderstanding of State vs. Parameters:** An attempt to change the file path from a persistent `Arc<RwLock<String>>` to a `StringParam` was incorrect. **Lesson:** As documented previously, `Param` types are for host-automatable values. Simple persistent state that is not automatable should use the `#[persist]` attribute on a standard thread-safe type.
*   **Final Lesson:** GUI integrations are a major source of complexity and failure. The path forward requires a slow, careful, example-driven approach. The immediate problem (the crash) is resolved by removing the call to `rfd`, but a robust file dialog solution will require finding a compatible version of `egui-file-dialog` or writing a custom implementation that respects the `nih-plug` architecture.

### `egui` Dependency Version Conflict (Resolved)

*   **Original Problem:** After deciding to use `egui-file-dialog`, the build failed with dozens of errors related to mismatched types and missing methods. The root cause was a dependency conflict: `nih-plug` (from git) used `egui` v0.31.1, while the chosen version of `egui-file-dialog` (v0.5.0) used `egui` v0.27.2.
*   **Resolution:**
    1.  Used `cargo tree` to explicitly identify the two conflicting `egui` versions.
    2.  Used web search to browse the `egui-file-dialog` repository and its `Cargo.toml` files for different tagged versions.
    3.  Identified that `egui-file-dialog` v0.10.0 depends on `egui` v0.31.0, which is compatible with `nih-plug`'s requirement.
    4.  Updated `Cargo.toml` to use this specific compatible version.
*   **Lesson Learned:**
    1.  **`cargo tree` is Essential:** For any dependency-related issue, especially in a complex ecosystem like `egui`, `cargo tree` is the first and most important diagnostic tool. It makes version conflicts immediately obvious.
    2.  **Manual Version Vetting:** When depending on libraries from git, you cannot rely on `crates.io`'s automatic semantic versioning. You must manually check the `Cargo.toml` of the git dependency to find its exact requirements, and then find compatible versions of any related libraries you wish to add.
    3.  **Check the Source:** The `Cargo.toml` file of a crate is the ultimate source of truth for its dependencies. Browsing the file for specific tags in the git repository is a reliable way to find compatible versions.

### VST3 Bundling Failure (Resolved)

*   **Original Problem:** The VST3 plugin was not being generated by `cargo build` and was not loading in Carla, even though the CLAP plugin passed validation. The `cargo xtask bundle` command, which is the standard `nih-plug` way to create bundles, was also failing silently.
*   **Resolution:**
    1.  Created a `bundler.toml` file with the required metadata for the plugin. This did not fix the `xtask` issue.
    2.  Bypassed the `xtask` system entirely and created the VST3 bundle manually by creating the `.vst3` directory structure and copying the compiled `.so` file into it.
*   **Lesson Learned:**
    1.  **`xtask` is not guaranteed:** The `cargo xtask` system is a convention, not a requirement. If it fails, it may be due to subtle configuration or versioning issues.
    2.  **Manual Bundling is a Viable Alternative:** The VST3 and CLAP formats are just specific directory structures. Understanding this allows for manual creation of the bundles, which is a robust fallback when the automated tooling fails.
    3.  **Isolate the Problem:** The `clap-validator` tool was critical in proving that the core plugin code was correct, which allowed me to focus on the VST3 packaging as the source of the problem.
