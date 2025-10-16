//! # Workspace Task Runner
//!
//! This `xtask` crate is a convention for adding automation and build scripts
//! to a Rust workspace. It's a simple binary that can be run with `cargo xtask <subcommand>`.
//!
//! All the build and automation logic for this project is handled by the
//! [`nih_plug_xtask`](https://github.com/robbert-vdh/nih-plug/tree/master/xtask)
//! crate, which provides a standard set of tasks for `nih-plug` based plugins.
//! This `main.rs` file simply acts as an entry point to that functionality.

/// The main entry point for the `xtask` runner.
///
/// This function delegates all command-line arguments and execution to the
/// `nih_plug_xtask::main` function, which provides the actual build and
/// automation tasks for the plugin.
fn main() {
    nih_plug_xtask::main();
}
