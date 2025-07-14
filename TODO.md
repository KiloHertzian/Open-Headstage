# Open Headstage - TODO

This file tracks the development tasks for the Open Headstage project.

## Phase 1: Anechoic Core CLAP Plugin (MVP)

### 1.1: Core Infrastructure & Dependencies
- [x] **Core Infra: Basic parameter handling for MVP controls:** Define and implement basic plugin parameters using `nih-plug` for the MVP features.
- [x] **Core Infra: Implement FFI bindings for libmysofa:** Implement FFI bindings for `libmysofa`.
- [x] Build System: Resolve CI/CD compilation issues (Rustc/nih-plug version incompatibility, git fetching errors).
- [x] **Core Infra: Debug and fix initial compilation errors:** Systematically debug and resolve the initial compilation errors related to `nih-plug` integration, FFI, and the borrow checker.
- [ ] **Core Infra: Investigate SOFA file path persistence:** Research and implement the correct method for persisting the SOFA file path in the plugin state.

### 1.2: SOFA HRTF Integration
- [ ] **SOFA: Load SOFA file and parse basic metadata:** Implement functionality to load a .sofa file using the `libmysofa` FFI bindings.
- [ ] **SOFA: Implement logic to select/extract 4 HRIRs based on angle parameters:** Develop logic to select the nearest available HRTF measurement from the loaded SOFA data.
- [ ] **SOFA: Implement HRIR resampling if needed:** Implement HRIR resampling using a library like `rubato`.
- [ ] **SOFA: Unit tests for SOFA loading and IR extraction:** Create unit tests for the SOFA manager.

### 1.3: Core DSP - Convolution Engine
- [ ] **Convolution: Implement core 4-path partitioned FFT convolution:** Develop the 4-path partitioned FFT convolution engine.
- [ ] **Convolution: Unit tests for convolution engine:** Create comprehensive unit tests for the convolution engine.

### 1.4: Headphone Parametric Equalization
- [ ] **HeadphoneEQ: Implement IIR parametric EQ bank:** Implement a minimum 10-band IIR parametric equalizer.
- [ ] **HeadphoneEQ: Implement AutoEq preset parsing (basic):** Implement basic parsing for AutoEq project PEQ settings.
- [ ] **HeadphoneEQ: Unit tests for PEQ filters:** Create unit tests for the parametric EQ filters.

### 1.5: Integration & UI
- [ ] **Integration: Integrate all modules into the main plugin processing chain:** Combine the Convolution Engine, SOFA Manager, and Headphone EQ modules into the main audio processing chain.
- [ ] **UI: Connect parameters to a generic UI provided by nih-plug:** Ensure all defined plugin parameters are correctly exposed and controllable through `nih-plug`'s generic editor.
- [ ] **UI: Basic UI elements for SOFA loading, angle selection, PEQ:** Create basic UI elements for SOFA file selection, HRTF angle input, and controls for the headphone PEQ.

### 1.6: Testing & Deployment
- [ ] **Integration: DAW testing and debugging:** Perform thorough testing of the plugin in various Linux DAWs.
- [ ] **Build System: Run `cargo clippy` for linting and code quality checks.**
- [ ] **Build System: Run `cargo fmt` for code formatting.**
- [ ] **Build System: Create CLAP bundle:** Use `nih_plug_xtask` to bundle the CLAP plugin for distribution.

## Phase 2: Enhancements (Post-MVP)
- [ ] **Speaker Emulation EQ:** Research and implement common speaker type emulations.
- [ ] **Basic Room Simulation Module:** Integrate a simple algorithmic reverb or a convolution slot for a basic Room Impulse Response (RIR).
- [ ] **Advanced SOFA/HRIR Management:** UI for listing available HRIR measurements within a loaded SOFA file.
- [ ] **UI Enhancements:** Implement a 2D draggable XY Pad for intuitive speaker angle selection.
- [ ] **Performance Optimization:** In-depth profiling of audio processing chain.

## Future Considerations
- [ ] **VST3 Plugin Format:** Explore and implement VST3 support.
- [ ] **Advanced Room Acoustics:** More sophisticated room simulation algorithms.
- [ ] **Dynamic Head Tracking:** Research and integrate head tracking.
- [ ] **Expanded Headphone EQ Database/Integration:** Easier import or built-in support for a wider range of headphone EQ profiles.
- [ ] **Cross-platform Compatibility:** Investigate potential for compiling on other platforms.
