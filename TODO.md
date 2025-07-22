# TODO List

This file tracks the development tasks for the Open Headstage project.

**NOTE:** Tasks marked as complete (`[x]`) are functionally integrated but may require further verification and hardening as the project evolves.

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

## Phase 2: User Interface & User Experience

- [ ] **UI:** Implement basic UI elements (knobs, buttons) for all parameters using `egui`.
- [ ] **UI:** Connect UI elements to the plugin's parameters.
- [ ] **UI:** Implement a file dialog to allow users to select a SOFA file from within the plugin.
- [ ] **UI:** Research and implement a stable, host-compatible file dialog solution.
- [ ] **Verification:** Debug and fix plugin loading failure in hosts like Carla.
- [ ] **Logging:** Enhance plugin logging to capture more details during initialization in a host environment.
- [ ] **UI:** Design and implement a visual representation of the speaker setup (azimuth/elevation).
- [ ] **UX:** Ensure parameter changes from the UI are smooth and don't cause audio artifacts.

## Phase 3: Advanced Features & Polish

- [ ] **DSP:** Implement HRIR resampling for different sample rates.
- [ ] **Headphone EQ:** Implement parsing of AutoEQ profile files.
- [ ] **Presets:** Implement a system for saving and loading plugin presets.
- [ ] **Performance:** Profile and optimize the DSP code.

## Future Vision (Post-MVP)

- [ ] **Surround Sound:** Extend the engine to support 5.1/7.1 to stereo binaural downmixing.


## Completed Tasks

- **[CORE-INFRA-2] Fix Compilation Errors from EQ Refactor**
  - **Description:** Address the 9 compilation errors that arose after refactoring the EQ parameters to use a `Vec<EqBandParams>`.
  - **Status:** DONE
  - **Sub-tasks:**
    - [x] Re-add the `use crate::dsp::convolution::ConvolutionEngine;` statement.
    - [x] Investigate and fix the `#[param_map]` attribute usage. This is the most critical error. I will start by searching for `Vec<Params>` in the `nih-plug` repository examples.
    - [x] Correct the UI code for the frequency `ParamSlider` to not use the non-existent `logarithmic()` method.
    - [x] Fix the audio buffer access in the `process` function, reverting to a safe pattern to get left and right channels.
    - [x] Correct the final gain application loop to properly iterate over samples.
  - **Next Step:** N/A

- **[CORE-INFRA-1] Basic Parameter Handling**
  - **Description:** Implement the core parameter handling for the plugin using `nih-plug`. This includes defining parameters for output gain, speaker angles, and the SOFA file path.
  - **Status:** DONE
  - **Sub-tasks:**
    - [x] Define `OpenHeadstageParams` struct.
    - [x] Add `output_gain` `FloatParam`.
    - [x] Add speaker angle `FloatParam`s.
    - [x] Add `sofa_file_path` as a persistent string.
    - [x] Refactor EQ parameters to be a `Vec<EqBandParams>`.
    - [x] Update UI to dynamically display all EQ bands.
    - [x] Update processing loop to use the new EQ parameter structure.
  - **Next Step:** N/A
