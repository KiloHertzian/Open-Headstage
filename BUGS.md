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
    4.  **Compiler Errors Guide Architecture:** The series of ownership, lifetime, and trait implementation errors were not just syntax problems; they were pointing to a fundamental architectural issue in how state was being shared. Resolving them required changing the data structures themselves, not just the function calls.
