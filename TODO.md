<canon>
**Directive:** This document is a living history of the project. Completed tasks and phases must never be removed. They serve as a record of progress and decision-making.
</canon>

**Verification Protocol:** Before any task involving user-facing changes (UI, UX, or core functionality) is marked as complete (`[x]`), the user (Project Lead) must give an explicit "GO" after a verification run. A "NO GO" will result in the task being re-opened to address the feedback.

# TODO List

This file tracks the development tasks for the Open Headstage project.

**NOTE:** Tasks marked as complete (`[x]`) are functionally integrated but may require further verification and hardening as the project evolves.

---
## Current Priority

- [x] **[UI-REFACTOR] Restore Original UI Layout:**
    - **Description:** After attempting to make the PEQ panel a permanent part of the UI, the layout was disrupted. The task was to revert the changes and restore the original, preferred layout with the slide-out panel.
    - **Priority:** Highest.
    - **Status:** DONE.

- [x] **[BUG-FIX] Fix Slider Double-Click Reset:**
    - **Description:** The double-click-to-reset functionality on the main sliders (Master Output, Azimuth, Elevation) is not working as intended. This needs to be fixed and verified.
    - **Priority:** Highest.
    - **Status:** DONE.

- [x] **[DOCS] Update BUGS.md with Slider Reset Lesson:**
    - **Description:** Add a "Lesson Learned" entry to `BUGS.md` detailing the resolution of the slider reset bug.
    - **Priority:** High.
    - **Content to Add:**
      ```markdown
      ### `egui` Slider Reset Logic (Resolved)

      *   **Original Problem:** The double-click-to-reset functionality on the main UI sliders was not working. Multiple attempts to fix this by manually handling the `double_clicked()` and `changed()` events failed, leading to incorrect behavior or compilation errors.
      *   **Root Cause Analysis:** The core issue was fighting the `nih-plug` framework instead of using its provided tools. The manual implementation attempted to re-create logic that was already built into the framework's own widgets. The `egui::Slider` is a generic widget, but `nih-plug` provides a "smart" `nih_plug_egui::widgets::ParamSlider` that is deeply integrated with the parameter system. Furthermore, when attempting to use `ParamSlider`, the incorrect constructor (`::new()`) was used instead of the correct one (`::for_param()`), which requires passing the `ParamSetter`.
      *   **Resolution:** The manual `egui::Slider` and all associated event-handling logic were removed entirely. They were replaced with the idiomatic `nih_plug_egui::widgets::ParamSlider`.
          1.  The `use nih_plug_egui::widgets;` statement was added.
          2.  All sliders were changed to `ui.add(widgets::ParamSlider::for_param(&params.my_param, setter));`.
          3.  This widget correctly handles user interactions, including double-click-to-reset, and communicates with the `nih-plug` parameter system automatically. The `.with_value_to_string()` formatter on the `FloatParam` was also automatically respected, solving the dB display issue without any extra code.
      *   **Lesson Learned:**
          1.  **Use the Framework's Widgets:** When a framework provides its own UI widgets (like `ParamSlider`), always prefer them over generic ones. They contain critical integration logic that is difficult and error-prone to replicate manually. The framework's widgets are the framework's intended API for UI interaction.
          2.  **Simplify to the Intended Path:** The repeated failures and increasing complexity of the manual solution were a strong signal that the approach was wrong. The solution was to simplify by removing all the manual code and reverting to the single, framework-provided component. If a solution feels like you're fighting the tool, you probably are.
          3.  **Check the Constructor Signature:** A quick look at the `ParamSlider` source or documentation would have revealed the correct `for_param(&param, &setter)` constructor, preventing the final build failure. The compiler's error message was the ultimate guide here.
      ```

---
## Backlog & Research Tasks

- [ ] **[RESEARCH] Custom Window Decorations:**
  - **Description:** Research and potentially implement custom window decorations (minimize, maximize, close) to replace the OS title bar.
  - **Priority:** Medium.
  - **Considerations:** Must ensure cross-platform compatibility (Linux/Plasma, Windows, Mac) and aim for a simple implementation if possible.
- [ ] **[FEATURE] Master Bypass and Reset:**
    - **Description:** Implement global controls for bypassing and resetting the plugin.
    - **Priority:** Medium.
    - **Sub-tasks:**
        - [ ] **Master Bypass:** Add a master bypass button under the Master Output section. Requires research into click/pop-free audio muting techniques.
        - [ ] **Reset to Default:** Add a button to reset all plugin parameters to their initial "out-of-the-box" state without deleting user profiles or presets.

<canon>
**Directive:** This document is a living history of the project. Completed tasks and phases must never be removed. They serve as a record of progress and decision-making.
</canon>

**Verification Protocol:** Before any task involving user-facing changes (UI, UX, or core functionality) is marked as complete (`[x]`), the user (Project Lead) must give an explicit "GO" after a verification run. A "NO GO" will result in the task being re-opened to address the feedback.

# TODO List

This file tracks the development tasks for the Open Headstage project.

**NOTE:** Tasks marked as complete (`[x]`) are functionally integrated but may require further verification and hardening as the project evolves.

---
## Current Priority

- [ ] **[BUG-FIX] Fix Slider Double-Click Reset:**
    - **Description:** The double-click-to-reset functionality on the main sliders (Master Output, Azimuth, Elevation) is not working as intended. This needs to be fixed and verified.
    - **Priority:** Highest.

---
## Backlog & Research Tasks

- [ ] **[FEATURE] Implement Advanced EQ Features:**
    - **Description:** Add core features for advanced equalization, including target curve loading and visualization, and audio device selection for the standalone version.
    - **Priority:** Medium.
    - **Sub-tasks:**
        - [ ] **Audio Device Selector:** Research and implement an audio output device selector in the settings panel using `cpal` and `egui`.
        - [ ] **Target Curve Loading:** Add a button to the EQ editor to load target curves from text/CSV files using `egui-file-dialog` and a custom parser.
        - [ ] **Target Curve Visualization:** Extend the EQ graph to draw the loaded target curve.

- [ ] **[RESEARCH] Custom Window Decorations:**
  - **Description:** Research and potentially implement custom window decorations (minimize, maximize, close) to replace the OS title bar.
  - **Priority:** Medium.
  - **Considerations:** Must ensure cross-platform compatibility (Linux/Plasma, Windows, Mac) and aim for a simple implementation if possible.

- [ ] **[FEATURE] Master Bypass and Reset:**
    - **Description:** Implement global controls for bypassing and resetting the plugin.
    - **Priority:** Medium.
    - **Sub-tasks:**
        - [ ] **Master Bypass:** Add a master bypass button under the Master Output section. Requires research into click/pop-free audio muting techniques.
        - [ ] **Reset to Default:** Add a button to reset all plugin parameters to their initial "out-of-the-box" state without deleting user profiles or presets.

- [ ] **[BUILD-4] Diagnose DAW Plugin Detection Failure:**
  - **Description:** With a stable standalone binary now working, resume the diagnostic protocol to determine why the CLAP plugin is not being detected by hosts like Carla and Reaper.
  - **Priority:** Low.
  - **Sub-tasks:**
    - [ ] Systematically follow the steps in `docs/research/Diagnostic Protocol for CLAP on LINUX.md`.
    - [ ] Verify dynamic library dependencies using `ldd`.
    - [ ] Investigate host-specific logging and caching.

- [ ] **[UI-REFINEMENT] Implement Scalable Window Resizing:**
  - **Description:** Allow the window to be resized within a certain range, with the UI elements scaling proportionally.
  - **Priority:** Low.

- [ ] **[TEST-2] Test Linux Release:**
  - **Description:** Perform comprehensive testing on the standalone Linux build on various distributions.

- [ ] **[TEST-3] Test Windows Release:**
  - **Description:** Create and test a standalone Windows build.

- [ ] **[DOCS-1] Review ARCHITECTURE.md:**
  - **Description:** Perform a detailed review of the `ARCHITECTURE.md` file to ensure it accurately reflects the current state of the project and make any necessary updates.

- [ ] **[SOFA-5] Revisit SOFA Loading and Resampling:**
  - **Description:** Verify that the SOFA loading and HRIR resampling engine is robust and performs correctly across a wide range of host sample rates.
  - **Sub-tasks:**
    - [ ] Test with standard sample rates (44.1 kHz, 48 kHz, 96 kHz).
    - [ ] Test with high-end sample rates (192 kHz, 384 kHz).
    - [ ] Investigate and fix any potential audio artifacts or performance issues at the rate extremes.

- [ ] **[DOCS-2] Re-implement and maintain CHANGELOG.md:**
  - **Description:** Re-create the changelog with accurate dates and ensure it aligns with the project's legal framework and licensing considerations. This task should be done in conjunction with the legal framework phase.
  - **Priority:** Lowest.

---
## Completed Tasks (History)

- [x] **[BUG-FIX] Restore File Dialog Functionality:**
    - **Description:** The file dialogs for loading SOFA and AutoEQ files are no longer appearing. This is a critical regression.
    - **Priority:** Highest.

- [x] **[UI-REFINEMENT] Implement High-Priority GUI Polish:**
  - **Description:** Apply a new set of detailed UI refinements based on user feedback to improve usability and aesthetics.
  - **Priority:** Highest.
  - **Sub-tasks:**
    - [ ] **Main Sliders Reset:** Implement double-click-to-reset functionality for the Master Output, Azimuth, and Elevation sliders.
    - [x] **Main Headers & Buttons Font:** Increase the font size for the "Master Output", "Speaker Configuration", and "Headphone Equalization" headers.
    - [x] **Apply/Cancel Buttons:** Increase the size of the "Apply" and "Cancel" buttons in the EQ editor by ~50%.
    - [x] **EQ Curve Visualizer:** Increase the height of the placeholder by 40%.

- [x] **[UI-REFINEMENT] Overhaul EQ Editor Panel:**
  - **Description:** Refactor the slide-out EQ editor panel to improve usability and fix layout bugs.
  - **Priority:** High.
  - **Sub-tasks:**
    - [x] **Q-Factor Control:**
        - [x] Change the Q-factor control from a slider to a `DragValue` widget.
        - [x] Implement double-click reset on the "Q" **label** to a default of 0.7.
        - [x] Set the default Q value for all bands to 0.7 out-of-the-box.
    - [x] **Control Value Ranges:** Ensure only the Gain control can have negative values. Frequency and Q should be restricted to positive values.
    - [x] **Layout Spacing:** Add spacers or fixed widths to the Freq/Q/Gain controls and between the Enable/Disable button and filter type dropdown to prevent the layout from resizing dynamically.

- [x] **[UI-REFINEMENT] Redesign Main Panel Layout:**
  - **Description:** Refactor the main panel UI into logical, well-spaced groups to improve clarity and usability.
  - **Status:** DONE - The main panel is already organized with collapsible headers. The speaker sliders are now correctly aligned using a grid layout.

- [x] **[UI-BUG] Fix Speaker Configuration Layout:**
  - **Description:** The "Left" speaker control is floating incorrectly, and the "Right" control is not visible. The layout needs to be fixed to correctly display both speaker controls.
  - **Status:** DONE - Replaced the horizontal layout with a `egui::Grid` to ensure proper alignment of speaker controls.

- [x] **[UI-REFINEMENT] Increase Control and Font Size:**
  - **Description:** Increase the font size for all UI elements except for the PEQ section. Buttons and sliders should also be made larger to improve usability and readability.
  - **Status:** DONE - Modified the `egui::Style` in the editor setup to increase the base font sizes and item spacing.

- [x] **[UI-REFINEMENT] Enhance EQ Editor Panel:**
  - **Description:** Improve the layout and functionality of the slide-out EQ editor panel.
  - **Status:** DONE - Replaced the horizontal `ScrollArea` with a `egui::Grid` and vertical sliders to create a compact, non-scrolling layout.

- [x] **[UI-REFINEMENT] Refactor `with_layer_id`:**
  - **Description:** Replace the deprecated `with_layer_id` with the modern `ui.scope()` pattern in `src/lib.rs` to remove the compilation warning.
  - **Status:** DONE - The function `with_layer_id` was not found in the codebase, indicating the task was already completed or is no longer relevant.

- [x] **[BUILD-3] Implement Standalone Executable for Debugging:**
  - **Description:** Configure the project to build a standalone executable for Linux. This will allow for easier debugging of the GUI and core logic outside of a DAW.
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
  - **Status:** DONE

- [ ] **[UI-ADV] Design Foundational UI for Advanced EQ:**
  - **Description:** Conduct a dedicated UI/UX research and design phase for the advanced EQ features. The goal is to create a powerful and intuitive interface.
  - **Sub-tasks:**
    - [ ] Design a "Noob Mode" with simplified controls.
    - [ ] Design an "Expert Mode" that exposes all filter controls and advanced options.
    - [ ] Design the UI flow for selecting a headphone and target profile, in preparation for the AutoEQ integration.
- [ ] **[META] Phase Review:** Document key decisions and lessons learned in `BUGS.md`.

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
  - **Description:** Under strict user supervision, review the `GEMINI.md` file to.
  - **Goal:** Solidify the operational framework and ensure long-term stability of the agent's core directives.
- [ ] **[META] Phase Review:** Document key decisions and lessons learned in `BUGS.md`.

## Future Vision (Post-MVP)

- [ ] **Surround Sound:** Extend the engine to support 5.1/7.1 to stereo binaural downmixing.
- [ ] **Dynamic Head Tracking:** Integrate dynamic head tracking to adjust the binaural rendering in real-time.

## Reminders, Ideas, & Lessons Learned

*   **Lesson (`egui-file-dialog`):** The correct usage pattern for `egui-file-dialog` is to chain the `.picked()` method call directly to the `.update()` call (e.g., `if let Some(path) = file_dialog.update(ctx).picked()`). The `update()` method returns a `Dialog` object containing the frame's interaction results, and this return value must be used to get the outcome.
*   **Lesson (`nih-plug` UI):** The `nih-plug-egui` widgets like `ParamSlider` are "smart" and automatically handle the `begin_set_parameter()` and `end_set_parameter()` calls when a user interacts with them. Do not wrap them manually. These calls are only needed when setting parameter values programmatically, for instance when applying a preset.
*   **Lesson (Rust Syntax):** To silence an "unused variable" warning, prefix the variable name with an underscore (e.g., `_async_executor`). This is a direct signal to the compiler that the variable is intentionally unused.
*   **Architectural Pattern (`nih-plug` Threading):** To avoid `clap-validator` warnings about recursive `on_main_thread()` calls, do not trigger background tasks directly from the GUI closure. The correct pattern is to use a channel to send a task message from the GUI thread to the plugin's `process()` method. The `process()` method, which has access to the `ProcessContext`, can then safely execute the background task without causing recursion.
*   **Future Refactoring Idea (Code Quality):** In `src/lib.rs`, the `StereoParametricEQ` is initialized with a hardcoded sample rate of `44100.0`. This should be refactored to use the `current_sample_rate` variable, which is dynamically updated by the host in the `initialize()` method. This will make the EQ more robust and adaptable to different host environments.
*   **Operational Reminder (Plugin Validation):** The `cargo xtask bundle` command can fail silently. The most reliable validation workflow is: 1. `cargo build --release`. 2. Manually create the `.clap` directory. 3. Manually copy the `.so` file into the bundle. 4. Run `clap-validator` directly on the `.so` file inside the bundle.
*   **Lesson (Code Hygiene):** When refactoring, it's crucial to remove all remnants of the old approach. The `ParamChange` enum and its related channel logic were leftovers from a previous implementation that cluttered the code and created misleading compiler warnings. A clean refactor removes the old code entirely.
*   **Operational Reminder (Tooling):** The `clap-validator` is a critical tool but is not in the system's `PATH` by default. For consistent development, its location should be documented, or it should be installed in a standard location.


- [ ] **[BUILD-4] Diagnose DAW Plugin Detection Failure:**
  - **Description:** With a stable standalone binary now working, resume the diagnostic protocol to determine why the CLAP plugin is not being detected by hosts like Carla and Reaper.
  - **Priority:** Low.
  - **Sub-tasks:**
    - [ ] Systematically follow the steps in `docs/research/Diagnostic Protocol for CLAP on LINUX.md`.
    - [ ] Verify dynamic library dependencies using `ldd`.
    - [ ] Investigate host-specific logging and caching.

- [ ] **[UI-REFINEMENT] Implement Scalable Window Resizing:**
  - **Description:** Allow the window to be resized within a certain range, with the UI elements scaling proportionally.
  - **Priority:** Low.

- [ ] **[TEST-2] Test Linux Release:**
  - **Description:** Perform comprehensive testing on the standalone Linux build on various distributions.

- [ ] **[TEST-3] Test Windows Release:**
  - **Description:** Create and test a standalone Windows build.

- [ ] **[DOCS-1] Review ARCHITECTURE.md:**
  - **Description:** Perform a detailed review of the `ARCHITECTURE.md` file to ensure it accurately reflects the current state of the project and make any necessary updates.

- [ ] **[SOFA-5] Revisit SOFA Loading and Resampling:**
  - **Description:** Verify that the SOFA loading and HRIR resampling engine is robust and performs correctly across a wide range of host sample rates.
  - **Sub-tasks:**
    - [ ] Test with standard sample rates (44.1 kHz, 48 kHz, 96 kHz).
    - [ ] Test with high-end sample rates (192 kHz, 384 kHz).
    - [ ] Investigate and fix any potential audio artifacts or performance issues at the rate extremes.

- [ ] **[DOCS-2] Re-implement and maintain CHANGELOG.md:**
  - **Description:** Re-create the changelog with accurate dates and ensure it aligns with the project's legal framework and licensing considerations. This task should be done in conjunction with the legal framework phase.
  - **Priority:** Lowest.

---
## Completed Tasks (History)

- [x] **[BUG-FIX] Restore File Dialog Functionality:**
    - **Description:** The file dialogs for loading SOFA and AutoEQ files are no longer appearing. This is a critical regression.
    - **Priority:** Highest.

- [x] **[UI-REFINEMENT] Implement High-Priority GUI Polish:**
  - **Description:** Apply a new set of detailed UI refinements based on user feedback to improve usability and aesthetics.
  - **Priority:** Highest.
  - **Sub-tasks:**
    - [ ] **Main Sliders Reset:** Implement double-click-to-reset functionality for the Master Output, Azimuth, and Elevation sliders.
    - [x] **Main Headers & Buttons Font:** Increase the font size for the "Master Output", "Speaker Configuration", and "Headphone Equalization" headers.
    - [x] **Apply/Cancel Buttons:** Increase the size of the "Apply" and "Cancel" buttons in the EQ editor by ~50%.
    - [x] **EQ Curve Visualizer:** Increase the height of the placeholder by 40%.

- [x] **[UI-REFINEMENT] Overhaul EQ Editor Panel:**
  - **Description:** Refactor the slide-out EQ editor panel to improve usability and fix layout bugs.
  - **Priority:** High.
  - **Sub-tasks:**
    - [x] **Q-Factor Control:**
        - [x] Change the Q-factor control from a slider to a `DragValue` widget.
        - [x] Implement double-click reset on the "Q" **label** to a default of 0.7.
        - [x] Set the default Q value for all bands to 0.7 out-of-the-box.
    - [x] **Control Value Ranges:** Ensure only the Gain control can have negative values. Frequency and Q should be restricted to positive values.
    - [x] **Layout Spacing:** Add spacers or fixed widths to the Freq/Q/Gain controls and between the Enable/Disable button and filter type dropdown to prevent the layout from resizing dynamically.

- [x] **[UI-REFINEMENT] Redesign Main Panel Layout:**
  - **Description:** Refactor the main panel UI into logical, well-spaced groups to improve clarity and usability.
  - **Status:** DONE - The main panel is already organized with collapsible headers. The speaker sliders are now correctly aligned using a grid layout.

- [x] **[UI-BUG] Fix Speaker Configuration Layout:**
  - **Description:** The "Left" speaker control is floating incorrectly, and the "Right" control is not visible. The layout needs to be fixed to correctly display both speaker controls.
  - **Status:** DONE - Replaced the horizontal layout with a `egui::Grid` to ensure proper alignment of speaker controls.

- [x] **[UI-REFINEMENT] Increase Control and Font Size:**
  - **Description:** Increase the font size for all UI elements except for the PEQ section. Buttons and sliders should also be made larger to improve usability and readability.
  - **Status:** DONE - Modified the `egui::Style` in the editor setup to increase the base font sizes and item spacing.

- [x] **[UI-REFINEMENT] Enhance EQ Editor Panel:**
  - **Description:** Improve the layout and functionality of the slide-out EQ editor panel.
  - **Status:** DONE - Replaced the horizontal `ScrollArea` with a `egui::Grid` and vertical sliders to create a compact, non-scrolling layout.

- [x] **[UI-REFINEMENT] Refactor `with_layer_id`:**
  - **Description:** Replace the deprecated `with_layer_id` with the modern `ui.scope()` pattern in `src/lib.rs` to remove the compilation warning.
  - **Status:** DONE - The function `with_layer_id` was not found in the codebase, indicating the task was already completed or is no longer relevant.

- [x] **[BUILD-3] Implement Standalone Executable for Debugging:**
  - **Description:** Configure the project to build a standalone executable for Linux. This will allow for easier debugging of the GUI and core logic outside of a DAW.
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
  - **Status:** DONE

- [ ] **[UI-ADV] Design Foundational UI for Advanced EQ:**
  - **Description:** Conduct a dedicated UI/UX research and design phase for the advanced EQ features. The goal is to create a powerful and intuitive interface.
  - **Sub-tasks:**
    - [ ] Design a "Noob Mode" with simplified controls.
    - [ ] Design an "Expert Mode" that exposes all filter controls and advanced options.
    - [ ] Design the UI flow for selecting a headphone and target profile, in preparation for the AutoEQ integration.
- [ ] **[META] Phase Review:** Document key decisions and lessons learned in `BUGS.md`.

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
- [ ] **Dynamic Head Tracking:** Integrate dynamic head tracking to adjust the binaural rendering in real-time.

## Reminders, Ideas, & Lessons Learned

*   **Lesson (`egui-file-dialog`):** The correct usage pattern for `egui-file-dialog` is to chain the `.picked()` method call directly to the `.update()` call (e.g., `if let Some(path) = file_dialog.update(ctx).picked()`). The `update()` method returns a `Dialog` object containing the frame's interaction results, and this return value must be used to get the outcome.
*   **Lesson (`nih-plug` UI):** The `nih-plug-egui` widgets like `ParamSlider` are "smart" and automatically handle the `begin_set_parameter()` and `end_set_parameter()` calls when a user interacts with them. Do not wrap them manually. These calls are only needed when setting parameter values programmatically, for instance when applying a preset.
*   **Lesson (Rust Syntax):** To silence an "unused variable" warning, prefix the variable name with an underscore (e.g., `_async_executor`). This is a direct signal to the compiler that the variable is intentionally unused.
*   **Architectural Pattern (`nih-plug` Threading):** To avoid `clap-validator` warnings about recursive `on_main_thread()` calls, do not trigger background tasks directly from the GUI closure. The correct pattern is to use a channel to send a task message from the GUI thread to the plugin's `process()` method. The `process()` method, which has access to the `ProcessContext`, can then safely execute the background task without causing recursion.
*   **Future Refactoring Idea (Code Quality):** In `src/lib.rs`, the `StereoParametricEQ` is initialized with a hardcoded sample rate of `44100.0`. This should be refactored to use the `current_sample_rate` variable, which is dynamically updated by the host in the `initialize()` method. This will make the EQ more robust and adaptable to different host environments.
*   **Operational Reminder (Plugin Validation):** The `cargo xtask bundle` command can fail silently. The most reliable validation workflow is: 1. `cargo build --release`. 2. Manually create the `.clap` directory. 3. Manually copy the `.so` file into the bundle. 4. Run `clap-validator` directly on the `.so` file inside the bundle.
*   **Lesson (Code Hygiene):** When refactoring, it's crucial to remove all remnants of the old approach. The `ParamChange` enum and its related channel logic were leftovers from a previous implementation that cluttered the code and created misleading compiler warnings. A clean refactor removes the old code entirely.
*   **Operational Reminder (Tooling):** The `clap-validator` is a critical tool but is not in the system's `PATH` by default. For consistent development, its location should be documented, or it should be installed in a standard location.
