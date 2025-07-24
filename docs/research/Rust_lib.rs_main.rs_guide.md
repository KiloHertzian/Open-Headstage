
The Rust lib.rs and main.rs Canon: A Guide to Crate Structure and Visibility


Part 1: The Core Concept: A Package of Two Crates

The most common source of confusion when structuring a Rust project with both a library and a binary component stems from a misunderstanding of the relationship between a package and a crate. Mastering this distinction is the first and most critical step toward resolving import errors and structuring projects idiomatically.

Deconstructing Rust's Terminology: Packages and Crates

In the Rust ecosystem, the terms "package" and "crate" have precise, distinct meanings that are foundational to how the compiler and the build tool, Cargo, operate.1
A Package is a project managed by Cargo, defined by the presence of a Cargo.toml manifest file. It represents a bundle of one or more crates that provide a cohesive set of functionality. A package is the fundamental unit for building, testing, and sharing code; it is what you might publish to the central repository, crates.io, or include in a version control system.2
A Crate, on the other hand, is a compilation unit. It is the smallest amount of code that the Rust compiler, rustc, considers at a time.2 When
rustc compiles code, it produces either a binary executable or a library file (an .rlib or .so/.dll/.dylib). These outputs correspond directly to crates. Crates contain a tree of modules, which control the organization, scope, and privacy of your code.1
The central rule governing their relationship is this: a single package can contain at most one library crate, but it can contain as many binary crates as you like.2 This rule is the architectural principle that underpins the
lib.rs/main.rs project structure.

The lib.rs and main.rs Duality: Two Crates in One Package

When a Rust package contains both a src/lib.rs and a src/main.rs, it is not a single entity with two entry points. Instead, Cargo, by convention, treats it as a package containing two separate crates that happen to share the same name and source directory.6
The Library Crate: The file src/lib.rs is the crate root for the package's library crate. All modules defined within or referenced from lib.rs belong to this library crate. If your package is named my_plugin in Cargo.toml, this will compile into a library also named my_plugin.2
The Binary Crate: The file src/main.rs is the crate root for a binary crate. The main function within this file serves as the entry point for the executable. This binary crate implicitly shares the same name as the package.2
The most crucial concept to internalize is that the binary crate has an implicit dependency on the library crate of the same package.7 From the perspective of the code in
main.rs, the library defined in lib.rs is treated no differently than an external dependency like serde or rand that you would list in Cargo.toml. They are compiled as separate units, and the binary links against the compiled library.

The Mental Model Shift from Other Languages

The persistent unresolved import errors often arise from applying mental models from other programming languages, such as Python, Node.js, or Java, to Rust's unique architecture. In many languages, all files within a project directory are considered part of a single, unified namespace or module system. Importing is often a runtime operation that resolves file paths.
Rust operates on a fundamentally different principle: the separation into crates is a rigid, compile-time boundary. An attempt to use use crate::... or mod lib; from within main.rs is a direct symptom of this mismatched mental model.7
Let's trace the logic to see why these intuitive approaches fail:
The Flawed Premise: A developer assumes that because main.rs and lib.rs are in the same src directory, they share a common internal module tree. Based on this, they try to use crate::my_function to access a function from lib.rs.
Rust's Compile-Time Reality: The Rust compiler, rustc, processes one crate at a time.4 When it is invoked to compile the binary crate, its entire world is defined by the crate root
src/main.rs. The crate path prefix, therefore, refers to the root of the binary crate's module tree, which is main.rs itself. At this stage, the compiler has no intrinsic knowledge of the source code residing in lib.rs.
The Cargo Bridge: The build tool, Cargo, is the orchestrator that connects these two separate worlds. It first invokes rustc to compile the library crate from src/lib.rs into a reusable .rlib file. Then, it invokes rustc a second time to compile the binary crate from src/main.rs. During this second compilation, Cargo explicitly tells the compiler that the previously compiled library is an external dependency available for use.
The Correct Conclusion: From the perspective of the code inside main.rs, the library portion of its own package is an external entity. Consequently, it must be accessed using the same mechanism as any other external crate: by its public name, which is the package name defined in Cargo.toml. This is why internal pathing keywords like crate or super fail, and why an external-style import is the one and only canonical solution.

Part 2: The Canonical Solution: A Step-by-Step Implementation

With the correct mental model established, the practical solution becomes straightforward and logical. It involves ensuring the library exposes a public API and the binary consumes it correctly.

Step 1: Defining the Public API in lib.rs

The first step is to define the public Application Programming Interface (API) of your library. Any function, struct, enum, trait, or module that you intend to use from main.rs (or any other external consumer) must be explicitly declared as public using the pub keyword.11
In Rust, items are private by default. This principle, known as "privacy by default," is a cornerstone of Rust's design, ensuring that implementation details are encapsulated and cannot be accidentally relied upon by outside code.13 If an item in
lib.rs is not marked pub, it is invisible to main.rs, regardless of the import path used.
For example, to expose a configuration struct and a primary function, your lib.rs would look like this:

Rust


// In src/lib.rs

// This struct is part of the public API.
pub struct Config {
    pub setting: String,
}

// This function is also part of the public API.
pub fn run_app(config: &Config) {
    println!("Running the application with setting: {}", config.setting);
}

// This function is private and cannot be accessed from main.rs.
fn internal_helper() {
    //... implementation detail
}



Step 2: Consuming the Library in main.rs

Once the public API is defined in lib.rs, you can consume it from main.rs. This is done with a use statement that references the package name as the crate path. The package name is the one specified in your Cargo.toml file under the [package].name key.6
If your package name in Cargo.toml is my-cli-tool, Rust's module system requires you to use underscores in the import path. Therefore, you would write use my_cli_tool::....15
The correct way to import the Config struct and run_app function from the previous example would be:

Rust


// In src/main.rs

// Use the package name to import public items from the library crate.
use my_cli_tool::{Config, run_app};

fn main() {
    let config = Config {
        setting: "enabled".to_string(),
    };
    run_app(&config);
}


It is critical to avoid the common pitfalls at this stage:
DO NOT write mod lib; in main.rs. The mod keyword is for declaring a module as part of the current crate's hierarchy, not for importing from another crate.7
DO NOT use use crate::... to access items from lib.rs. As established, crate refers to the root of the binary crate (main.rs), not the library crate.16

Step 3: The Role of Cargo.toml - Convention Over Configuration

For a standard project layout containing just src/lib.rs and src/main.rs, no special configuration is required in Cargo.toml.2
Cargo operates on a "convention over configuration" basis. Its target auto-discovery mechanism automatically detects the presence of src/lib.rs and src/main.rs and understands their respective roles as library and binary crate roots.9 It will compile both and correctly link the binary against the library without any explicit instructions from you.
You only need to add [lib] or [[bin]] sections to Cargo.toml if you wish to deviate from these conventions (e.g., your source files are located in non-standard directories) or if you need to customize target-specific build settings. For the vast majority of projects following this pattern, the default Cargo.toml generated by cargo new is sufficient.
The "magic" that allows main.rs to find the library via its package name is not a feature of the Rust language itself, but a function of the Cargo build tool. When Cargo compiles main.rs, it automatically passes a flag to rustc, such as --extern my_cli_tool=path/to/compiled/my_cli_tool.rlib. This command-line argument explicitly informs the compiler that an external crate named my_cli_tool is available and provides the path to its compiled artifact.17 Understanding this interaction demystifies the process by clearly separating the roles:
rustc handles compilation within a single crate's context, while cargo orchestrates the multi-crate build process and manages the dependencies between them.

Part 3: The Definitive Example: A Complete, Compilable Project

To solidify these concepts, here is a complete, minimal, and verifiable project that correctly implements the library/binary pattern. This example is guaranteed to compile and run with cargo run, providing a reliable foundation for your own projects.

Project Setup

First, create a new binary package and then add the library file.
Create the project:
cargo new ai_plugin_example
Navigate into the new directory:
cd ai_plugin_example
Create the empty library file:
touch src/lib.rs
The resulting file structure should be:



ai_plugin_example/
├── Cargo.toml
└── src/
    ├── lib.rs
    └── main.rs



Cargo.toml

The Cargo.toml file is generated by cargo new and requires no modifications.

Ini, TOML


[package]
name = "ai_plugin_example"
version = "0.1.0"
edition = "2021"

# See more keys and their definitions at https://doc.rust-lang.org/cargo/reference/manifest.html

[dependencies]



src/lib.rs (The Library Crate)

This file defines the core logic. Note the use of pub to expose the MyPlugin struct, its name field, and the run_plugin_logic function to external consumers like main.rs.

Rust


//! The core library for our AI Plugin example.
//! This crate contains the primary logic and data structures.

/// A simple plugin structure that holds configuration.
/// The struct itself and its fields must be public to be accessible.
pub struct MyPlugin {
    pub name: String,
}

impl MyPlugin {
    /// A public constructor for creating a new plugin instance.
    pub fn new(name: &str) -> Self {
        Self { name: name.to_string() }
    }
}

/// A public function that performs a core task using a plugin instance.
/// This function is part of the library's public API.
pub fn run_plugin_logic(plugin: &MyPlugin) {
    println!("Running logic for plugin: {}", plugin.name);
}



src/main.rs (The Binary Crate)

This file serves as the executable entry point. It imports the necessary items from the ai_plugin_example library crate and orchestrates the application flow.

Rust


//! The binary entry point that uses our library.
//! This crate is a consumer of the `ai_plugin_example` library.

// Correctly import items from the library crate using the package name.
// This is the canonical way to link the binary to the library.
use ai_plugin_example::{MyPlugin, run_plugin_logic};

fn main() {
    println!("AI Plugin Application starting...");

    // Instantiate a struct defined in the library.
    let plugin = MyPlugin::new("Image Recognition");

    // Call a function defined in the library.
    run_plugin_logic(&plugin);

    println!("AI Plugin Application finished.");
}



Verification

To compile and run the project, execute the following command in your terminal from the ai_plugin_example directory:
cargo run
The command will first compile the ai_plugin_example library, then compile the ai_plugin_example binary (linking against the library), and finally run the executable. The expected output is:



   Compiling ai_plugin_example v0.1.0 (/path/to/ai_plugin_example)
    Finished dev [unoptimized + debuginfo] target(s) in 0.50s
     Running `target/debug/ai_plugin_example`
AI Plugin Application starting...
Running logic for plugin: Image Recognition
AI Plugin Application finished.


This successful execution confirms that the structure is correct and the communication between the two distinct crates is working as intended.11

Part 4: Deconstructing Common Failures: A Troubleshooting Guide

Even with the canonical structure, certain errors can arise. Understanding their root causes is key to efficient debugging.

Error Deep Dive: unresolved import

This is by far the most common error encountered in this scenario. It almost always stems from one of three distinct misunderstandings.

Cause 1: The Item is Not pub

Symptom: error[E0432]: unresolved import \my_project::MyPlugin`followed bynote: no `MyPlugin` in the root`.
Explanation: This error occurs when your use statement in main.rs is syntactically correct, but the item you are trying to import (MyPlugin in this case) was not declared with the pub keyword in lib.rs. The binary crate, as an external consumer, is subject to the library's privacy rules. Private items are completely invisible outside their defining module, so the compiler correctly reports that it cannot find the item.19
Fix: Ensure that the struct, function, or module you need to access from main.rs is explicitly marked pub in lib.rs.

Cause 2: The use Path is Incorrect

Symptom: error[E0432]: unresolved import \crate::MyPlugin`followed byhelp: a similar name exists in the module: `my_project``.
Explanation: This error is a direct result of the mismatched mental model discussed in Part 1. Using use crate::MyPlugin; instructs the compiler to look for MyPlugin at the root of the current crate. When compiling the binary, the current crate is the one defined by main.rs. Since MyPlugin is not defined there, the import fails.20
Fix: Always use the package name as the root of the path when importing from the library into the binary: use my_project::MyPlugin;.

Cause 3 (The Conceptual Error): Misusing mod

Symptom: Writing mod lib; in main.rs often doesn't produce a clean unresolved import error but instead leads to a cascade of confusing, seemingly unrelated errors originating from within your library code.
Explanation: The keywords mod and use serve fundamentally different purposes. use brings an existing path into the local scope for convenience. mod, however, is a declaration that builds the module tree. When you write mod lib; in main.rs, you are instructing the compiler to do the following:
Find a file named lib.rs (or lib/mod.rs).
Parse its contents.
Incorporate those contents into the binary crate's module tree under a new, private module named lib.
This action completely subverts the intended package structure. The compiler now attempts to compile the library's source code as if it were part of the binary. If lib.rs contains its own module declarations (e.g., mod sub_module;), the compiler will now look for sub_module.rs relative to main.rs, which will likely fail. Furthermore, any use crate::... paths inside the library's submodules will now be resolved relative to the binary's crate root (main.rs), breaking all internal library paths and leading to chaos.7
Fix: Never use mod lib; to connect main.rs to lib.rs. The connection is implicit and managed by Cargo. Use use package_name::...; to access the library's public API.

Error Deep Dive: ...is only public within the crate...

This family of errors relates to Rust's strict visibility and API boundary rules.
Symptom: error: \MyInternalStruct` is private... `MyInternalStruct` is only public within the crate, and cannot be re-exported`.
Explanation: This error typically occurs when you try to pub use an item from a private module. For an item to be truly public and accessible to an external crate, there must be a continuous chain of pub visibility from the crate root all the way to the item itself. If any parent module in that path is private, the chain is broken, and the item cannot be exported.13
Consider this example in lib.rs:
Rust
// This module is private by default.
mod internal {
    // This struct is public within the `internal` module.
    pub struct MyThing;
}

// This re-export will FAIL because `internal` is private.
// The compiler prevents you from making a private path public.
pub use internal::MyThing;


Fix: Make the parent module public as well: pub mod internal {... }.
A related and critical error is leaking private types in public function signatures.
Symptom: error[E0446]: private type \MyThing` in public interface`.
Explanation: This happens if you have a public function whose signature involves a private type, for example, by returning it: pub fn get_thing() -> MyThing {... } where MyThing is a private struct. The function's signature is a public contract. If this contract promises to return a value of a type that the caller (main.rs) is not allowed to see or name, the contract is incoherent and unusable. Rust's compiler enforces API boundary integrity by preventing such "private type leaks".23
Fix: Ensure that all types used in the arguments and return values of a pub function are at least as visible as the function itself. This usually means making the types pub as well.

Part 5: Beyond the Basics: Evolving Your Project Structure

The lib.rs/main.rs pattern is not merely a workaround for compiler rules; it is the foundation of idiomatic application design in Rust. Understanding how to leverage and extend it allows your projects to scale gracefully.

The "Thin Binary, Fat Library" Philosophy

This structure encourages a design philosophy where the library (lib.rs) is "fat" and the binary (main.rs) is "thin".5
Fat Library: lib.rs and its submodules should contain all the core logic, data structures, and business rules of your application. This is the reusable, testable heart of your project.
Thin Binary: main.rs should act as a lightweight wrapper or entry point. Its primary responsibilities are to handle concerns specific to an executable context, such as parsing command-line arguments, reading configuration files, setting up logging, handling standard input/output, and calling into the library to execute the core logic.
This separation provides significant advantages:
Testability: Core logic in lib.rs can be easily covered by unit tests (#[test]) and integration tests. It is significantly more difficult and cumbersome to test logic that is tightly coupled with the main function.
Reusability: The library can be published to crates.io and used as a dependency in other Rust projects. The binary simply becomes one of many potential consumers of your library's API.
Separation of Concerns: It creates a clean architectural boundary between the application's core domain and its execution environment, making the code easier to reason about, maintain, and evolve.

Scaling Up: Multiple Binaries

As a project grows, you may need multiple executables that share the same core library. For instance, a web service might have a binary for running the server and another for running database migrations. Cargo supports this elegantly via the src/bin/ directory.2
Any file placed in src/bin/ is treated as a separate binary crate. For example:
src/bin/server.rs will be compiled into an executable named server.
src/bin/cli_tool.rs will be compiled into an executable named cli_tool.
Each of these binary crates, just like src/main.rs, automatically has an implicit dependency on the src/lib.rs library crate within the same package. They can all use my_package_name::... to access the shared logic. You can compile and run a specific binary using the --bin flag: cargo run --bin server.

The Next Level: Cargo Workspaces

For very large, multi-component systems, Cargo provides workspaces. A workspace is a set of related packages that are developed in tandem and share a single Cargo.lock file and target directory. This is the ideal structure for projects composed of several distinct but interdependent packages, such as a core library, a procedural macro crate, a web server, and a client library, all managed within a single repository.12

Rust Project Structure Patterns and Use Cases

The following table provides a decision-making guide for choosing the appropriate project structure. It illustrates a clear evolutionary path from a simple script to a complex, multi-package system, helping you select the right architecture for your needs.
Structure
Crate Layout
Primary Use Case
When to Use It
Single Binary
src/main.rs only
Simple command-line tools, scripts, or single-purpose applications.
When your project is small, has a single entry point, and you don't foresee needing to reuse its logic in other programs or test it as a separate library.
Library + Binary
src/lib.rs + src/main.rs
Standard application development. The idiomatic pattern for most executables.
This should be your default. Use it when building any non-trivial application. It enforces a clean separation of core logic from the application's entry point, dramatically improving testability and reusability.
Library + Multiple Binaries
src/lib.rs + src/bin/tool1.rs, src/bin/tool2.rs...
A project that provides a core library and several related command-line tools (e.g., a server, a client, a migration tool).
When your core logic is shared, but you need to expose it through multiple different executables with distinct command-line interfaces or behaviors.
Cargo Workspace
A root Cargo.toml with [workspace], and multiple sub-directories, each a separate package.
Large, multi-component systems where several packages (libraries and binaries) are developed and versioned in lockstep.
When your project is complex enough to be composed of several distinct, interdependent packages (e.g., a core library, a procedural macro, a web server, and a client library) that you want to manage in a single repository.

Works cited
Managing Growing Projects with Packages, Crates, and Modules - The Rust Programming Language, accessed July 23, 2025, https://doc.rust-lang.org/book/ch07-00-managing-growing-projects-with-packages-crates-and-modules.html
Packages and Crates - The Rust Programming Language, accessed July 23, 2025, https://doc.rust-lang.org/book/ch07-01-packages-and-crates.html
Packages and Crates - The Rust Programming Language, accessed July 23, 2025, https://carols10cents.github.io/book/ch07-01-packages-and-crates.html
Confused about Package vs. Crate terminology. : r/rust - Reddit, accessed July 23, 2025, https://www.reddit.com/r/rust/comments/lvtzri/confused_about_package_vs_crate_terminology/
rust - What is the exact difference between a Crate and a Package? - Stack Overflow, accessed July 23, 2025, https://stackoverflow.com/questions/68250956/what-is-the-exact-difference-between-a-crate-and-a-package
How to define a separate lib.rs in the same folder with main.rs - help ..., accessed July 23, 2025, https://users.rust-lang.org/t/how-to-define-a-separate-lib-rs-in-the-same-folder-with-main-rs/106449
Main.rs and lib.rs at same level - help - The Rust Programming Language Forum, accessed July 23, 2025, https://users.rust-lang.org/t/main-rs-and-lib-rs-at-same-level/42499
Cargo build shows unresolved import - help - The Rust Programming Language Forum, accessed July 23, 2025, https://users.rust-lang.org/t/cargo-build-shows-unresolved-import/45445
Cargo Targets - The Cargo Book - Rust Documentation, accessed July 23, 2025, https://doc.rust-lang.org/cargo/reference/cargo-targets.html
Structural differencebetween cargo --bin and --lib - The Rust Programming Language Forum, accessed July 23, 2025, https://users.rust-lang.org/t/structural-differencebetween-cargo-bin-and-lib/102640
How to split up a rust project into main.rs and lib.rs? - Stack Overflow, accessed July 23, 2025, https://stackoverflow.com/questions/70092179/how-to-split-up-a-rust-project-into-main-rs-and-lib-rs
rust - Package with both a library and a binary? - Stack Overflow, accessed July 23, 2025, https://stackoverflow.com/questions/26946646/package-with-both-a-library-and-a-binary
Visibility and privacy - The Rust Reference, accessed July 23, 2025, https://doc.rust-lang.org/reference/visibility-and-privacy.html
Paths for Referring to an Item in the Module Tree - The Rust Programming Language, accessed July 23, 2025, https://doc.rust-lang.org/beta/book/ch07-03-paths-for-referring-to-an-item-in-the-module-tree.html
Crates and Modules - The Rust Programming Language - MIT, accessed July 23, 2025, https://web.mit.edu/rust-lang_v1.25/arch/amd64_ubuntu1404/share/doc/rust/html/book/first-edition/crates-and-modules.html
Rust modules confusion when there is main.rs and lib.rs - Stack Overflow, accessed July 23, 2025, https://stackoverflow.com/questions/57756927/rust-modules-confusion-when-there-is-main-rs-and-lib-rs
Path and module system changes - The Rust Edition Guide, accessed July 23, 2025, https://doc.rust-lang.org/edition-guide/rust-2018/path-changes.html
Library crates and Multiple Binary crates How/why to use? : r/rust - Reddit, accessed July 23, 2025, https://www.reddit.com/r/rust/comments/fxprtf/library_crates_and_multiple_binary_crates_howwhy/
Controlling Visibility with pub - The Rust Programming Language - MIT, accessed July 23, 2025, https://web.mit.edu/rust-lang_v1.25/arch/amd64_ubuntu1404/share/doc/rust/html/book/second-edition/ch07-02-controlling-visibility-with-pub.html
Unresolved import issues with the bin directory : r/learnrust - Reddit, accessed July 23, 2025, https://www.reddit.com/r/learnrust/comments/1agnm1q/unresolved_import_issues_with_the_bin_directory/
Problem with unresolved import - help - The Rust Programming Language Forum, accessed July 23, 2025, https://users.rust-lang.org/t/problem-with-unresolved-import/62408
Unresolved imports for sub module following zero to production book : r/learnrust - Reddit, accessed July 23, 2025, https://www.reddit.com/r/learnrust/comments/18rc505/unresolved_imports_for_sub_module_following_zero/
How do I make an Rust item public within a crate, but private outside it? - Stack Overflow, accessed July 23, 2025, https://stackoverflow.com/questions/41666235/how-do-i-make-an-rust-item-public-within-a-crate-but-private-outside-it
Public type in private mod is not documented - The Rust Programming Language Forum, accessed July 23, 2025, https://users.rust-lang.org/t/public-type-in-private-mod-is-not-documented/79970
Why does Rust think my private type must be public unless I use pub(crate)?, accessed July 23, 2025, https://stackoverflow.com/questions/65756914/why-does-rust-think-my-private-type-must-be-public-unless-i-use-pubcrate
thoughts on the `src/main.rs` and `src/lib.rs` pattern · rust-lang api-guidelines - GitHub, accessed July 23, 2025, https://github.com/rust-lang/api-guidelines/discussions/167
Understanding Packages and Crates in Rust - Reddit, accessed July 23, 2025, https://www.reddit.com/r/rust/comments/mxms1n/understanding_packages_and_crates_in_rust/
How can I build multiple binaries with Cargo? - Stack Overflow, accessed July 23, 2025, https://stackoverflow.com/questions/36604010/how-can-i-build-multiple-binaries-with-cargo
