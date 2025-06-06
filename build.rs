extern crate bindgen;

use std::env;
use std::path::PathBuf;

fn main() {
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
