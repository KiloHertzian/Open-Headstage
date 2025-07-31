# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Changed
- **Documentation:** Replaced the single architecture diagram in `README.md` with two new, more detailed Mermaid diagrams for "High-Level Architecture" and "Real-time Audio Signal Flow". This provides a clearer and more aesthetically pleasing overview of the project.

### Fixed
- **File Dialog:** Corrected the usage of the `egui-file-dialog` library to ensure that file dialogs for loading SOFA and AutoEQ files now appear correctly. This was a critical regression.
- **Slider Reset:** Refactored all main panel sliders to use the idiomatic `nih_plug_egui::widgets::ParamSlider`. This fixes the double-click-to-reset functionality, which was previously broken.

### Changed
- **UI Polish:** Implemented a series of high-priority UI refinements to improve usability and aesthetics:
    - **Main Sliders:** All main sliders (Master Output, Speaker Azimuth, Speaker Elevation) can now be double-clicked to reset them to their default values.
    - **Headers:** Increased the font size of all section headers in the main panel and EQ editor for better readability.
    - **EQ Buttons:** The "Apply" and "Cancel" buttons in the EQ editor are now 50% larger, making them easier to click.
    - **EQ Visualizer:** The placeholder for the EQ curve visualization is now 40% taller in preparation for its implementation.
- **EQ Editor:** Overhauled the slide-out parametric EQ editor panel:
    - **Q-Factor Control:** The Q-factor `DragValue`'s label can now also be double-clicked to reset the value to a default of 0.7.
    - **Layout:** Added fixed spacing between controls to prevent the layout from shifting during value changes, creating a more stable and visually consistent experience.
