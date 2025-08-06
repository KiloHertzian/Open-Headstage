# BUGS - 

#  THIS DOCCUMENT IS NEVER PRUNED AND SHOULD ALWAYS GROW WITH PAST BUGS THAT GOT FIXED. THE GOAL IS TO KNOW HOW TO FIX REGRESSIONS, LEARN AND TRACK CURRENT BUGS.

## 1. Build Failure During Stereo Preset Implementation

*   **Status:** Fixed
*   **Priority:** High

### Root Cause Analysis

A series of build failures occurred while attempting to implement the stereo angle preset feature. The failures stemmed from two primary sources:

1.  **Incorrect `strum` Crate Configuration and Usage:**
    *   **Problem:** There was significant confusion regarding the correct setup for the `strum` and `strum_macros` crates. This led to incorrect `Cargo.toml` entries (combining them, missing features) and consequently, incorrect `use` statements in `app/src/lib.rs` and `app/src/dsp/parametric_eq.rs`.
    *   **Symptoms:** The compiler reported unresolved imports for traits like `Enum`, `EnumIter`, and `Display`, and errors indicating that methods like `.iter()` were not found for the enums. The `#[display(...)]` attribute was also not recognized.
    *   **Correct Approach:** The `strum` crate requires its `derive` feature to be enabled in `Cargo.toml`. The correct enums should use `#[derive(..., strum::EnumIter, strum::Display)]`.

2.  **Incorrect Handling of `egui::InnerResponse`:**
    *   **Problem:** The `egui::ComboBox` widget returns an `InnerResponse<Option<()>>`. The code attempted to directly call methods like `.map_or()` or use `if let Some(...)` on the `InnerResponse` struct itself, rather than on its `.inner` field which holds the `Option`.
    *   **Symptoms:** The compiler repeatedly threw `mismatched types` and `method not found` errors, correctly stating that `InnerResponse` is not an `Option`.
    *   **Correct Approach:** The `.inner` field must be accessed first. The robust way to handle this is with the pattern: `if let Some(response) = response.inner { if response.changed() { ... } }`.

### Lesson Learned

When encountering persistent build failures after modifying dependencies, the first step should be to meticulously verify `Cargo.toml` and the official documentation for the crate in question. For UI-related compiler errors, it is crucial to inspect the exact return types of the widgets being used and access nested fields (`.inner`) as required by the API, rather than making assumptions. Repeated, rapid-fire `replace` calls without re-reading the file context led to a frustrating debugging loop. A more methodical approach of read -> analyze -> write is necessary.

## 2. Runtime Errors and Warnings

*   **Status:** Active
*   **Priority:** High

### Root Cause Analysis

1.  **AutoEQ Path Resolution:**
    *   **Problem:** The application fails to find AutoEQ profile files because it's using a hardcoded relative path (`../PRESERVE/AutoEq/...`). This path is incorrect when the application is run from a different directory than the project root.
    *   **Symptoms:** `Failed to parse AutoEQ file ... No such file or directory` error in the console.
    *   **Proposed Solution:** The path to the `PRESERVE` directory should be determined at runtime relative to the location of the application executable.

2.  **Incorrect Parameter Setting:**
    *   **Problem:** Several `nih_plug` warnings (`Debug assertion failed: self.active_params.contains(param_id)`) indicate that `setter.set_parameter()` is being called without being properly enclosed in a `setter.begin_set_parameter()` and `setter.end_set_parameter()` block. This can lead to incorrect parameter updates and potential race conditions.
    *   **Symptoms:** Console is flooded with warnings when interacting with GUI controls like sliders and buttons.
    *   **Proposed Solution:** Audit all `setter` calls in `app/src/lib.rs` and ensure they follow the correct `begin -> set -> end` pattern.

3.  **JACK Audio Server Errors:**
    *   **Problem:** The application attempts to connect to the JACK audio server by default, which may not be running.
    *   **Symptoms:** A series of `jack server is not running or cannot be started` errors appear on startup.
    *   **Current State:** The application correctly falls back to the ALSA backend, so this is not a critical failure, but it does produce noise in the logs.
    *   **Proposed Solution:** Investigate if the default audio backend can be configured more gracefully or if the JACK connection attempt can be suppressed if not explicitly requested.