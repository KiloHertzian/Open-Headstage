# TODO List

This file tracks the development tasks for the Open Headstage project.

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
- [ ] **UI:** Design and implement a visual representation of the speaker setup (azimuth/elevation).
- [ ] **UX:** Ensure parameter changes from the UI are smooth and don't cause audio artifacts.

## Phase 3: Advanced Features & Polish

- [ ] **DSP:** Implement HRIR resampling for different sample rates.
- [ ] **Headphone EQ:** Implement parsing of AutoEQ profile files.
- [ ] **Presets:** Implement a system for saving and loading plugin presets.
- [ ] **Performance:** Profile and optimize the DSP code.

## Future Vision (Post-MVP)

- [ ] **Surround Sound:** Extend the engine to support 5.1/7.1 to stereo binaural downmixing.
