# Open Headstage - Dependency Licenses

This document provides a summary of the licenses for the key third-party libraries and components used in the Open Headstage project. The project aims to use permissively licensed software (MIT, Apache-2.0, ISC, etc.).

## Core Dependencies

| Crate | Version | License | Summary |
|---|---|---|---|
| `nih_plug` | git | `ISC` | A permissive license, functionally identical to MIT. |
| `rustfft` | 6.3.0 | `Apache-2.0 OR MIT` | Permissive, standard Rust ecosystem licenses. |
| `realfft` | 3.4.0 | `MIT` | A permissive license. |
| `rubato` | 0.16.2 | `MIT` | A permissive license. |
| `serde` | 1.0.219 | `Apache-2.0 OR MIT` | Permissive, standard Rust ecosystem licenses. |
| `bindgen` | 0.70.1 | `BSD-3-Clause` | A permissive license. |

## Key Transitive Dependencies

| Crate | License | Summary |
|---|---|---|
| `clap-sys` | `Apache-2.0 OR MIT` | Bindings for the CLAP audio plugin API. |
| `vst3-sys` | `GPLv3` | Bindings for the VST3 API. **Note:** This is a copyleft license. |
| `log` | `Apache-2.0 OR MIT` | Standard logging facade for Rust. |
| `crossbeam` | `Apache-2.0 OR MIT` | Tools for concurrent programming. |

## Important Note on `vst3-sys`

The `vst3-sys` crate, which provides the raw bindings to the VST3 SDK, is licensed under **GPLv3**. The VST3 SDK itself has a dual license (proprietary or GPLv3). By using the `vst3-sys` crate, this project must comply with the terms of the GPLv3.

This means that if you distribute a binary of Open Headstage that includes the VST3 plugin, you must also make the source code available under the terms of the GPLv3. The CLAP version of the plugin is not affected by this.

---
*This list was generated based on the output of `cargo license` and is intended as a summary. For full license text, please refer to the source repositories of the respective crates.*
