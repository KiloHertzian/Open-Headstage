extern crate bindgen;

use serde::Serialize;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

#[derive(Serialize, Debug)]
struct Headphone {
    name: String,
    source: String,
    path: PathBuf,
}

fn generate_headphone_index() {
    let out_dir = env::var("OUT_DIR").unwrap();
    let dest_path = Path::new(&out_dir).join("headphone_index.json");
    let autoeq_results_path = Path::new("../PRESERVE/AutoEq/results");

    println!(
        "cargo:rerun-if-changed={}",
        autoeq_results_path.to_str().unwrap()
    );

    let mut headphone_index: Vec<Headphone> = Vec::new();

    for entry in WalkDir::new(autoeq_results_path)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        let path = entry.path();
        if path.is_file() && path.to_str().unwrap_or("").ends_with("ParametricEQ.txt") {
            let components: Vec<_> = path.components().collect();
            if components.len() >= 4 {
                // Expected path structure: ../PRESERVE/AutoEq/results/{source}/{...}/{name}/{name} ParametricEQ.txt
                // The components we care about are relative to the `autoeq_results_path`
                let relative_components: Vec<_> = path
                    .strip_prefix(autoeq_results_path)
                    .unwrap()
                    .components()
                    .collect();

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

    let json_string = serde_json::to_string_pretty(&headphone_index)
        .expect("Failed to serialize headphone index");
    fs::write(&dest_path, json_string).expect("Failed to write headphone index");
    println!(
        "cargo:warning=open-headstage@0.1.0: Generated headphone index with {} entries.",
        headphone_index.len()
    );
}

fn main() {
    // Run the headphone index generator
    generate_headphone_index();

    // Tell cargo to tell rustc to link the system mysofa
    // shared library.
    println!("cargo:rustc-link-lib=mysofa");

    // Tell cargo to invalidate the built crate whenever the wrapper changes
    println!("cargo:rerun-if-changed=wrapper.h");

    // The bindgen::Builder is the main entry point
    // to bindgen, and lets you build up options for
    // the resulting bindings.
    let bindings = bindgen::Builder::default()
        // The input header we would like to generate
        // bindings for.
        .header("wrapper.h")
        // Tell cargo to invalidate the built crate whenever any of the
        // included header files changed.
        .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
        // Whitelist MYSOFA_EASY to ensure its fields are generated,
        // especially if it might otherwise be treated as an opaque type.
        .allowlist_type("MYSOFA_EASY")
        .allowlist_type("MYSOFA_HRTF") // Also whitelist related structs if needed
        .allowlist_type("MYSOFA_ATTRIBUTE")
        .allowlist_type("MYSOFA_LOOKUP")
        .allowlist_type("MYSOFA_NEIGHBORHOOD")
        // Potentially whitelist functions too if their signatures are problematic
        .allowlist_function("mysofa_open")
        .allowlist_function("mysofa_close")
        .allowlist_function("mysofa_getfilter_float")
        .allowlist_function("mysofa_s2c")
        .allowlist_function("mysofa_c2s")
        .allowlist_function("mysofa_get_sampling_rate") // if this is a function
        .blocklist_function("mysofa_getfilter_float") // Blocklist the function to manually define it
        .raw_line("unsafe extern \"C\" { pub fn mysofa_getfilter_float(easy: *mut MYSOFA_EASY, x: f32, y: f32, z: f32, IRleft: *mut f32, IRright: *mut f32, delayLeft: *mut f32, delayRight: *mut f32,) -> ::std::os::raw::c_int; }") // Manually define with correct return type
        // Finish the builder and generate the bindings.
        .generate()
        // Unwrap the Result and panic on failure.
        .expect("Unable to generate bindings");

    // Write the bindings to the $OUT_DIR/bindings.rs file.
    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("mysofa_bindings.rs"))
        .expect("Couldn't write bindings!");
}
