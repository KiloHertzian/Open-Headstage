//! # Build Script
//!
//! This script runs before the main compilation and is responsible for two main tasks:
//!
//! 1.  **Generating `libmysofa` Bindings**: It uses `bindgen` to create safe Rust bindings
//!     for the `libmysofa` C library, which is used for reading SOFA files. This allows
//!     the Rust code to call C functions for HRTF processing. The script specifies which
//!     functions and types to include and manually defines the signature for
//!     `mysofa_getfilter_float` to ensure correctness.
//!
//! 2.  **Indexing Headphone Profiles**: It walks the `PRESERVE/AutoEq/results` directory
//!     to find all available headphone equalization profiles. It then generates a JSON file
//!     (`headphone_index.json`) in the `OUT_DIR`, which is embedded into the plugin binary.
//!     This index allows the plugin to quickly search for and load headphone profiles at runtime.
//!
//! The script also tells Cargo to link against the `mysofa` system library and to re-run
//! the script if the `wrapper.h` header or the AutoEq directory changes.

extern crate bindgen;

use serde::Serialize;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

/// Represents a single headphone entry in the generated index.
#[derive(Serialize, Debug)]
struct Headphone {
    /// The name of the headphone model.
    name: String,
    /// The source of the measurement (e.g., "oratory1990").
    source: String,
    /// The full path to the Parametric EQ file.
    path: PathBuf,
}

/// Scans the AutoEq directory and generates a JSON index of all found headphone profiles.
///
/// This function is responsible for creating the `headphone_index.json` file that gets
/// compiled into the plugin. It ensures that the build process is re-run whenever the
/// contents of the AutoEq results directory change.
fn generate_headphone_index() {
    let out_dir = env::var("OUT_DIR").unwrap();
    let dest_path = Path::new(&out_dir).join("headphone_index.json");
    let autoeq_results_path = Path::new("../PRESERVE/AutoEq/results");

    // Tell Cargo to re-run this script if the AutoEq directory changes.
    println!(
        "cargo:rerun-if-changed={}",
        autoeq_results_path.to_str().unwrap()
    );

    let mut headphone_index: Vec<Headphone> = Vec::new();

    // Walk the directory to find all "ParametricEQ.txt" files.
    for entry in WalkDir::new(autoeq_results_path)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        let path = entry.path();
        if path.is_file() && path.to_str().unwrap_or("").ends_with("ParametricEQ.txt") {
            let components: Vec<_> = path.components().collect();
            if components.len() >= 4 {
                // The components we care about are relative to the `autoeq_results_path`
                let relative_components: Vec<_> = path
                    .strip_prefix(autoeq_results_path)
                    .unwrap()
                    .components()
                    .collect();

                // Extract the source and name from the path components.
                if relative_components.len() >= 3 {
                    let source = relative_components[0]
                        .as_os_str()
                        .to_string_lossy()
                        .into_owned();
                    let name_part = relative_components[relative_components.len() - 2]
                        .as_os_str()
                        .to_string_lossy()
                        .into_owned();

                    headphone_index.push(Headphone {
                        name: name_part,
                        source,
                        path: path.to_path_buf(),
                    });
                }
            }
        }
    }

    // Write the index to a JSON file in the OUT_DIR.
    let json_string = serde_json::to_string_pretty(&headphone_index)
        .expect("Failed to serialize headphone index");
    fs::write(&dest_path, json_string).expect("Failed to write headphone index");
    println!(
        "cargo:warning=open-headstage@0.1.0: Generated headphone index with {} entries.",
        headphone_index.len()
    );
}

/// The main function of the build script.
fn main() {
    // First, run the headphone index generator.
    generate_headphone_index();

    // Link against the system's `libmysofa` library.
    println!("cargo:rustc-link-lib=mysofa");

    // Tell Cargo to re-run this script if the C wrapper header changes.
    println!("cargo:rerun-if-changed=wrapper.h");

    // Configure `bindgen` to generate Rust bindings for `libmysofa`.
    let bindings = bindgen::Builder::default()
        .header("wrapper.h")
        // Invalidate the build crate whenever any of the included header files change.
        .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
        // Whitelist the necessary types and functions to ensure they are generated.
        .allowlist_type("MYSOFA_EASY")
        .allowlist_type("MYSOFA_HRTF")
        .allowlist_type("MYSOFA_ATTRIBUTE")
        .allowlist_type("MYSOFA_LOOKUP")
        .allowlist_type("MYSOFA_NEIGHBORHOOD")
        .allowlist_function("mysofa_open")
        .allowlist_function("mysofa_close")
        .allowlist_function("mysofa_getfilter_float")
        .allowlist_function("mysofa_s2c")
        .allowlist_function("mysofa_c2s")
        .allowlist_function("mysofa_get_sampling_rate")
        // Manually define the `mysofa_getfilter_float` function to correct the return type,
        // as `bindgen` sometimes struggles with complex C function signatures.
        .blocklist_function("mysofa_getfilter_float")
        .raw_line("unsafe extern \"C\" { pub fn mysofa_getfilter_float(easy: *mut MYSOFA_EASY, x: f32, y: f32, z: f32, IRleft: *mut f32, IRright: *mut f32, delayLeft: *mut f32, delayRight: *mut f32,) -> ::std::os::raw::c_int; }")
        .generate()
        .expect("Unable to generate bindings");

    // Write the generated bindings to a file in the `OUT_DIR`.
    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("mysofa_bindings.rs"))
        .expect("Couldn't write bindings!");
}
