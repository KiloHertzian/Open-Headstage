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
