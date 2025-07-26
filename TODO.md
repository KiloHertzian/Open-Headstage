<canon>
**Directive:** This document is a living history of the project. Completed tasks and phases must never be removed. They serve as a record of progress and decision-making.
</canon>

# TODO List

This file tracks the development tasks for the Open Headstage project.

**NOTE:** Tasks marked as complete (`[x]`) are functionally integrated but may require further verification and hardening as the project evolves.

---
## Current Priority

- [ ] **[BUILD-4] Diagnose DAW Plugin Detection Failure:**
  - **Description:** With a stable standalone binary now working, resume the diagnostic protocol to determine why the CLAP plugin is not being detected by hosts like Carla and Reaper.
  - **Priority:** Highest.
  - **Sub-tasks:**
    - [ ] Systematically follow the steps in `docs/research/Diagnostic Protocol for CLAP on LINUX.md`.
    - [ ] Verify dynamic library dependencies using `ldd`.
    - [ ] Investigate host-specific logging and caching.

- [x] **[BUILD-3] Implement Standalone Executable for Debugging:**
  - **Description:** Configure the project to build a standalone executable for Linux. This will allow for easier debugging of the GUI and core logic outside of a DAW.
  - **Priority:** Highest. This is a critical step to unblock the DAW detection issue.
  - **Sub-tasks:**
    - [x] Add the `standalone` feature to the `nih_plug` dependency in `Cargo.toml`.
    - [x] Add the necessary `nih_export_standalone!` macro to `src/main.rs`.
    - [x] Build and run the standalone binary to verify it launches.
    - [x] Debug and fix `GuiContext` warnings related to incorrect parameter setting.
  - **Status:** DONE

---
## Phase 1: Core Functionality & Integration (Complete)

- [x] **Core Infrastructure:** Basic parameter handling.
- [x] **Core Infrastructure:** FFI bindings for `libmysofa`.
- [x] **SOFA:** Load SOFA files and parse metadata.
- [x] **SOFA:** Implement logic to select and extract HRIRs.
- [x] **DSP:** Implement core FFT-based convolution engine.
- [x] **DSP:** Implement IIR parametric EQ for headphone correction.
- [x] **Integration:** Integrate all modules into a functional processing chain.
- [x] **Build System:** Set up and validate the build process for CLAP and VST3 plugins.
- [x] **Documentation:** Create initial `ARCHITECTURE.md` and `LICENSES.md`.

## Phase 2: User Interface & User Experience (Complete)

- [x] **UI:** Implement a stable, host-compatible file dialog using `egui-file-dialog`.
- [x] **Verification:** Debug and fix plugin loading failure in hosts like Carla.
- [x] **Logging:** Enhance plugin logging to capture more details during initialization in a host environment.
- [x] **UI:** Design and implement a visual representation of the speaker setup (azimuth/elevation).
- [x] **UX:** Ensure parameter changes from the UI are smooth and don't cause audio artifacts.

## Phase 3: Advanced Features & Polish (Complete)

- [x] **DSP:** Implement HRIR resampling for different sample rates.
- [x] **Headphone EQ:** Implement parsing of AutoEQ profile files.
- [x] **Presets:** Implement a system for saving and loading plugin presets.

---
## Phase 4: Legal & License Management

- [ ] **[LEGAL-1] Establish Licensing and Bill of Materials (BOM):**
  - **Description:** Formalize the project's licensing and maintain a clear record of all dependencies and their licenses.
  - **Sub-tasks:**
    - [ ] Create and maintain an evergreen Software Bill of Materials (SBOM).
    - [ ] Evaluate the licenses of all dependencies for compliance and risk.
    - [ ] Research and decide on the outbound license for the project.
- [ ] **[META] Phase Review:** Document key decisions and lessons learned in `BUGS.md`.

## Phase 5: Advanced EQ Engine & UI Design

- [x] **[BUILD-1] Verify Project Compilation:**
  - **Description:** After significant changes to licensing and dependencies (disabling VST3), ensure the project compiles cleanly on the main branch.
  - **Status:** DONE

- [x] **[BUILD-2] Investigate `clap-validator` recursive call warning:**
  - **Description:** The `clap-validator` tool reports a warning: `The plugin recursively called 'clap_host::on_main_thread()'. Aborted after ten iterations.`. Investigate the cause of this warning and resolve it.
  - **Status:** DONE

- [x] **[DSP-ADV] Implement Multiple Filter Types:**
  - **Description:** Implement a selection of DSP filter types for the parametric EQ, allowing the user to choose based on their needs.
  - **Sub-tasks:**
    - [x] Implement standard IIR (Minimum-Phase) biquad filters using the `biquad` crate.
    - [ ] Implement Linear-Phase FIR filters (via FFT convolution) for high-fidelity "mastering" mode.
    - [ ] Research and potentially implement a hybrid approach (e.g., oversampling).
  - **Reference:** `docs/research/EQ Implementation in Rust Research.md`
  - **Status:** DONE

- [ ] **[UI-ADV] Design Foundational UI for Advanced EQ:**
  - **Description:** Conduct a dedicated UI/UX research and design phase for the advanced EQ features. The goal is to create a powerful and intuitive interface.
  - **Sub-tasks:**
    - [ ] Design a "Noob Mode" with simplified controls.
    - [ ] Design an "Expert Mode" that exposes all filter controls and advanced options.
    - [ ] Design the UI flow for selecting a headphone and target profile, in preparation for the AutoEQ integration.
- [ ] **[META] Phase Review:** Document key decisions and lessons learned in `BUGS.md`.

## Phase 6: UI Refinement & Release Preparation

- [ ] **[UI-REFINEMENT] Improve layout and clarity of the 10-band PEQ editor.**
- [ ] **[RELEASE-PREP] Verify all features are working as expected for a v0.2.0 release.**
- [ ] **[RELEASE-PREP] Update CHANGELOG.md for v0.2.0.**
- [ ] **[RELEASE-PREP] Create a git tag for v0.2.0.**

## Phase 7: AutoEQ Database Integration

- [ ] **[FEAT-AEQ] Integrate AutoEQ Database:**
  - **Description:** Parse the AutoEQ results database and make it available within the plugin's UI as a preset system.
  - **Sub-tasks:**
    - [ ] Create a build script to parse the AutoEQ `ParametricEQ.txt` files into a single, bundled JSON file.
    - [ ] Implement the UI selectors (designed in Phase 5) for choosing a headphone and target.
    - [ ] Implement the logic to apply the selected AutoEQ preset to the plugin's parametric EQ bands.
- [ ] **[META] Phase Review:** Document key decisions and lessons learned in `BUGS.md`.

## Phase 8: Performance, Polish & Education

- [ ] **[PERF-1] Profile and Optimize DSP Code:**
  - **Description:** Profile and optimize the complete DSP code path using the standalone/JACK methodology.
  - **Reference:** `docs/research/Benchmarking-nih-plug.md`

- [ ] **[UI-POLISH] Implement Educational Tooltips & Guides:**
  - **Description:** Enhance the UI with educational components to make it accessible to a wider audience.
  - **Sub-tasks:**
    - [ ] Create an educational tooltips or an info panel that explains audio concepts (e.g., phase, latency).
    - [ ] The panel should include a comparison table for filter types (Latency, CPU Usage, Phase Response, Use Case).
- [ ] **[META] Phase Review:** Document key decisions and lessons learned in `BUGS.md`.

## Phase 9: Process Improvement & Refinement

- [ ] **[META-1] Review and Canonize GEMINI.md:**
  - **Description:** Under strict user supervision, review the `GEMINI.md` file to identify core, immutable principles and enclose them in `<canon>` blocks.
  - **Goal:** Solidify the operational framework and ensure long-term stability of the agent's core directives.
- [ ] **[META] Phase Review:** Document key decisions and lessons learned in `BUGS.md`.

## Future Vision (Post-MVP)

- [ ] **Surround Sound:** Extend the engine to support 5.1/7.1 to stereo binaural downmixing.
- [ ] **Dynamic Head Tracking:**
  - **Description:** Integrate dynamic head tracking to adjust the binaural rendering in real-time based on the user's head orientation. This will involve processing sensor data (e.g., from an IMU) and dynamically selecting or interpolating HRTFs. The system should account for yaw, pitch, and roll (tilt), as well as potential 3D positional tracking (above/below).
  - **Sub-tasks:**
    - [ ] Research open-source code examples and libraries for head tracking integration.
    - [ ] Develop a robust method for receiving and smoothing tracking data.
    - [ ] Extend the convolution engine to apply HRTF changes with minimal audio artifacts.
  - **Reference:** `docs/research/Dynamic Head Tracking Audio Feasibility.md`

## Reminders, Ideas, & Lessons Learned

*   **Lesson (`nih-plug` UI):** The `nih-plug-egui` widgets like `ParamSlider` are "smart" and automatically handle the `begin_set_parameter()` and `end_set_parameter()` calls when a user interacts with them. Do not wrap them manually. These calls are only needed when setting parameter values programmatically, for instance when applying a preset.
*   **Lesson (Rust Syntax):** To silence an "unused variable" warning, prefix the variable name with an underscore (e.g., `_async_executor`). This is a direct signal to the compiler that the variable is intentionally unused.
*   **Architectural Pattern (`nih-plug` Threading):** To avoid `clap-validator` warnings about recursive `on_main_thread()` calls, do not trigger background tasks directly from the GUI closure. The correct pattern is to use a channel to send a task message from the GUI thread to the plugin's `process()` method. The `process()` method, which has access to the `ProcessContext`, can then safely execute the background task without causing recursion.
*   **Future Refactoring Idea (Code Quality):** In `src/lib.rs`, the `StereoParametricEQ` is initialized with a hardcoded sample rate of `44100.0`. This should be refactored to use the `current_sample_rate` variable, which is dynamically updated by the host in the `initialize()` method. This will make the EQ more robust and adaptable to different host environments.
*   **Operational Reminder (Plugin Validation):** The `cargo xtask bundle` command can fail silently. The most reliable validation workflow is: 1. `cargo build --release`. 2. Manually create the `.clap` directory. 3. Manually copy the `.so` file into the bundle. 4. Run `clap-validator` directly on the `.so` file inside the bundle.
*   **Lesson (Code Hygiene):** When refactoring, it's crucial to remove all remnants of the old approach. The `ParamChange` enum and its related channel logic were leftovers from a previous implementation that cluttered the code and created misleading compiler warnings. A clean refactor removes the old code entirely.
*   **Operational Reminder (Tooling):** The `clap-validator` is a critical tool but is not in the system's `PATH` by default. For consistent development, its location should be documented, or it should be installed in a standard location.

## Completed Tasks

- **[CORE-INFRA-2] Fix Compilation Errors from EQ Refactor**
  - **Description:** Address the 9 compilation errors that arose after refactoring the EQ parameters to use a `Vec<EqBandParams>`.
  - **Status:** DONE

- **[CORE-INFRA-1] Basic Parameter Handling**
  - **Description:** Implement the core parameter handling for the plugin using `nih-plug`.
  - **Status:** DONE