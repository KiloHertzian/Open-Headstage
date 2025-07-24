# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.2.0] - 2025-07-24

### Added
- Implemented multiple filter types for the parametric EQ: Low Pass, High Pass, Band Pass, Notch, and All Pass.
- Added a `ComboBox` to the UI for selecting the filter type for each EQ band.

### Changed
- Refactored the biquad filter implementation to use the `biquad` crate for improved robustness and accuracy.
- Improved the layout of the 10-band parametric EQ editor for a more compact and user-friendly experience.

## [0.1.0] - 2025-07-23

### Added
- Initial project setup.
- Core DSP structure for convolution and parametric EQ.
- SOFA file loading capabilities via `libmysofa` and FFI.
- Basic `nih-plug` integration for CLAP plugin format.
- `architecture.md` for developer and AI guidance.
- This `CHANGELOG.md` file.
- Parsing of AutoEQ CSV files to apply headphone correction profiles.
- A preset system for saving and loading plugin states.
- A visualizer for the speaker setup.

---

**Note to AI Agents:** When contributing changes, please add a concise entry to the `[Unreleased]` section under the appropriate heading (Added, Changed, Fixed, etc.). Describe the user-facing impact of your change.
For example:
- Added feature X.
- Fixed bug Y that caused Z.
- Changed behavior of A to B for reason C.