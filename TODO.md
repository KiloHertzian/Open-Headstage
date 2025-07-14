# TODO: Open Headstage Project Tasks

This file tracks the development tasks for the Open Headstage project.

## 1. Current Sprint

### 1.1: Core Infrastructure & Dependencies
- [x] **Core Infra: Basic parameter handling for MVP controls:** Define and implement basic plugin parameters using `nih-plug` for the MVP features.
- [x] **Core Infra: Implement FFI bindings for libmysofa:** Implement FFI bindings for `libmysofa`.
- [x] Build System: Resolve CI/CD compilation issues (Rustc/nih-plug version incompatibility, git fetching errors).
- [x] **Core Infra: Debug and fix initial compilation errors:** Systematically debug and resolve the initial compilation errors related to `nih-plug` integration, FFI, and the borrow checker.
- [x] **Core Infra: Investigate SOFA file path persistence:** Confirmed that the `sofa_file_path` parameter is not persisting; it's an empty string on initialization. This indicates the DAW is not saving/loading the string parameter as expected.
- [ ] **Core Infra: Research String Parameter Persistence:** Investigate `nih-plug`'s documentation and examples for proper `String` parameter persistence, and research how DAWs typically handle saving/loading string parameters for plugins.

### 1.2: Convolution Engine
- [ ] **Convolution: Implement core FFT convolution:** Develop the core convolution algorithm using `rustfft` and `realfft`.
- [ ] **Convolution: Unit tests for convolution engine:** Write comprehensive unit tests for the convolution engine.

### 1.3: SOFA Loader
- [ ] **SOFA: Load SOFA and parse metadata:** Implement functionality to load SOFA files and extract relevant metadata (e.g., HRIRs, sample rate).
- [ ] **SOFA: Implement logic to select and extract HRIRs:** Develop logic to select and extract appropriate HRIRs based on speaker angles.
- [ ] **SOFA: Implement HRIR resampling:** Implement resampling capabilities for HRIRs to match the plugin's sample rate.
- [ ] **SOFA: Unit tests for loading and extraction:** Write unit tests for SOFA file loading and HRIR extraction.

### 1.4: Headphone EQ
- [ ] **Headphone EQ: Implement IIR parametric EQ:** Develop a parametric EQ using IIR filters.
- [ ] **Headphone EQ: Implement AutoEQ parsing:** Add functionality to parse AutoEQ data for headphone correction.
- [ ] **Headphone EQ: Unit tests for PEQ filters:** Write unit tests for the parametric EQ filters.

### 1.5: User Interface
- [ ] **UI: Basic UI elements:** Create basic UI elements using `egui` for parameter control.
- [ ] **UI: Connect parameters to generic UI:** Connect the plugin parameters to the generic UI elements.

### 1.6: Integration & Testing
- [ ] **Integration: Integrate modules into processing chain:** Combine the convolution engine, SOFA loader, and headphone EQ into a cohesive audio processing chain.
- [ ] **Integration: DAW testing and debugging:** Perform thorough testing and debugging within a Digital Audio Workstation (DAW).

## 2. Backlog

### 2.1: Enhancements
- [ ] **Enhancement: Add speaker distance parameter:** Allow users to adjust the perceived speaker distance.
- [ ] **Enhancement: Implement crossfeed:** Add a crossfeed feature for a more natural listening experience.

### 2.2: Documentation
- [ ] **Documentation: User manual:** Create a comprehensive user manual.
- [ ] **Documentation: Developer guide:** Write a guide for developers contributing to the project.

### 2.3: Performance Optimization
- [ ] **Performance: Profile and optimize DSP code:** Identify and optimize performance bottlenecks in the DSP code.

## 3. Critical Issues
- [x] **Git Push Failure:** The `git push` command failed due to authentication issues. Resolved by manual push.