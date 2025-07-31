# Known Issues & Development Status

This document tracks known bugs, limitations, and the overall development status of the Open Headstage project.

---

## Resolved Issues & Lessons Learned

### The Persistence Pitfall: A Deep Dive into `nih-plug` State Management (Resolved)

*   **Original Problem:** Multiple attempts to implement full state persistence for the standalone application failed. The initial implementation only saved audio device settings. Subsequent attempts to save the full state using `params.serialize_fields()` also failed, as did a more complex approach involving iterating through the `param_map()`. This resulted in a series of frustrating and repetitive compilation errors.
*   **Root Cause Analysis:** The core of the problem was a deep and persistent misunderstanding of the `nih-plug` state management and parameter system. I made several critical, incorrect assumptions:
    1.  **`serialize_fields` is Not for Parameters:** I incorrectly assumed that `params.serialize_fields()` would save the entire state of the plugin, including all `FloatParam`, `BoolParam`, etc. values. **This is false.** This function **only** serializes fields explicitly marked with the `#[persist]` attribute. It is intended for non-parameter data like window size or file paths, not for the user-facing parameter values themselves.
    2.  **`Param` Types are Not Directly Serializable:** I tried to solve the problem by creating a `FullState` struct that would hold the serialized `#[persist]` data and a `Vec` of parameter values. This failed because the `Param` types themselves (and the `ParamPtr` enum that points to them) are not designed to be serialized with `serde`. They are complex objects with internal state, not simple data containers.
    3.  **The `param_map()` API is for Host Communication:** I attempted to manually iterate through the `param_map()` to get and set parameter values. This led to a cascade of compiler errors because I was misusing the raw pointers (`*const FloatParam`) and trying to call methods that don't exist on them (`plain_value`, `set_plain_value`). This API is primarily for the framework's internal use in communicating with the plugin host, not for direct state management.
    4.  **`set_value()` has a context:** The correct method to programmatically change a parameter's value is `set_value()`. However, my attempts to call this in the `default()` constructor failed. This is because `set_value()` can only be called on an already-initialized parameter. The correct place to set the *initial* value is in the `FloatParam::new()` constructor itself.
*   **The Correct, Simple, and Robust Solution:** The simplest and most effective way to implement state persistence is to use a dedicated, serializable struct that mirrors the *values* of the parameters, and then manually copy the data to and from the `Params` object at the correct times.
    1.  **Create a `StandaloneConfig` Struct:** This struct should contain simple, serializable fields for every single parameter in the `OpenHeadstageParams` struct (e.g., `output_gain: f32`, `master_bypass: bool`, etc.).
    2.  **Implement `save_standalone_config()`:** This function reads the current value of every parameter from the `params` object using the `.value()` method and saves it into an instance of the `StandaloneConfig` struct. This struct is then serialized to JSON and written to a file.
    3.  **Refactor `OpenHeadstageParams::default()` to `::new(config)`:** The `impl Default` for `OpenHeadstageParams` was removed. It was replaced with a new constructor, `OpenHeadstageParams::new(config: StandaloneConfig)`. This constructor takes the loaded configuration as an argument and uses its values directly in the `FloatParam::new(..., config.output_gain, ...)` calls. This is the architecturally correct way to set the initial state of a parameter.
    4.  **Update `OpenHeadstagePlugin::default()`:** On startup, this function is now responsible for loading and deserializing the `StandaloneConfig` from the JSON file (or creating a default one if the file doesn't exist) and then passing it to `OpenHeadstageParams::new()` to create the fully-initialized parameters object.
    5.  **Connect to the UI:** The "Save Settings" button in the UI calls the `save_standalone_config()` function.
*   **Definitive Lesson Learned:**
    1.  **Separate State from Parameters:** The `OpenHeadstageParams` struct is for defining the plugin's parameters for the host. The `StandaloneConfig` struct is for saving and loading the *values* of those parameters for the standalone application. Do not conflate the two.
    2.  **Manual Mapping is Required:** There is no magic, one-shot function to save and load the entire state of a `nih-plug` plugin. You must manually create a serializable struct and write the code to copy the values between it and the `Params` object.
    3.  **Initialize with `::new()`, not `set_value()`:** The initial value of a parameter must be set in its constructor (e.g., `FloatParam::new()`). The `set_value()` method is for changing the value of an *already existing* parameter, and is not the correct tool for initialization.
    4.  **The Simplest Solution is Often the Right One:** My attempts to create a "clever" solution using `serialize_fields` and `param_map` were overly complex and wrong. The correct solution is a straightforward, if verbose, manual mapping. When a complex approach fails repeatedly, it is a strong signal to simplify.

### Architectural Whiplash: Navigating a Major `nih-plug` API Upgrade (Resolved)

*   **Original Problem:** A task that began as implementing an audio device selector spiraled into a multi-day debugging session after the project's core dependency, `nih-plug`, was updated from an old git revision to the modern `master` branch. The build was plagued by a cascade of dependency conflicts, trait implementation errors (specifically `serde::Serialize`), and calls to non-existent or private APIs, leading to significant thrashing and repeated failed build attempts.
*   **Root Cause Analysis:** This was a "perfect storm" of issues stemming from a single strategic error: **failing to fully internalize the new API and its architectural patterns before attempting to write code.** My mental model was an incorrect mix of old and new information from different research documents, leading to three critical failures:
    1.  **Dependency Mismatch:** I treated `nih-plug`'s dependencies (`baseview`, `egui-baseview`, etc.) as independent crates that could be pulled from `crates.io`. This was wrong. The `nih-plug` ecosystem is a set of tightly-coupled git dependencies that require specific, compatible revisions and feature flags.
    2.  **Persistence Model Confusion:** I conflated the state managed *by a host* (automatable parameters) with the configuration managed *by our standalone binary*. This led me to incorrectly try and `#[derive(Serialize)]` on the entire `Params` struct. This is impossible, as `nih-plug`'s `Param` types are not designed to be serialized directly with `serde`.
    3.  **Relying on Outdated Documentation:** I was working from multiple research documents of different ages. The solutions for the older API were completely incompatible with the newer one, but I kept trying to mix them, leading to calls to functions and types that no longer existed.
*   **Resolution (The Corrected Workflow):** The project was only unblocked by adopting a more disciplined, "source of truth" oriented workflow:
    1.  **Single Source of Truth for Dependencies:** The `Cargo.toml` from the local, up-to-date `nih-plug` repository clone was treated as the **canonical source** for all dependency versions, git revisions, and feature flags. Mirroring this configuration exactly solved all dependency hell issues immediately.
    2.  **Separation of Concerns for State:** A dedicated, serializable `StandaloneConfig` struct was created to hold *only* the settings for the standalone application (file paths, device names). The main `Params` struct was correctly left untouched by `serde`. The `OpenHeadstagePlugin::default()` constructor became the bridge, loading the `StandaloneConfig` from a JSON file and using its values to populate the fields in the `Params` struct at startup.
    3.  **Prioritizing the Latest Documentation:** The newest research document (`API Evolution...`) was identified and used as the **exclusive** guide, and all older information was disregarded. This provided the correct, modern API patterns that led to the final successful build.
*   **Lesson Learned:**
    1.  **Declare "Documentation Bankruptcy":** When upgrading a core framework, treat all previous knowledge and documentation as potentially outdated. Identify the single, most current and relevant example or guide (often the framework's own example code) and stick to it exclusively. Mixing information from different API versions is a recipe for disaster.
    2.  **Dependencies are an Ecosystem:** For frameworks like `nih-plug`, dependencies are not a-la-carte. Treat the framework's own `Cargo.toml` as a blessed "lockfile" for its ecosystem and mirror its versions and git revisions exactly.
    3.  **Separate Config from Params:** The `Params` struct is for the *host*. The standalone's configuration is for the *binary*. Create a separate, simple, serializable struct for the standalone's config and use it to *populate* the `Params` struct at startup. Never try to serialize the `Params` struct directly.

### UI Layout Refactoring (Resolved)

*   **Original Problem:** A request was made to change the slide-out PEQ panel into a permanently visible, static part of the main UI, preserving the expanded layout and proportions.
*   **Root Cause Analysis:** The initial attempt involved a significant refactoring of the `egui` layout from a `CentralPanel` with a conditional `SidePanel` to a horizontal layout containing two distinct containers. This approach, while logical, failed to replicate the exact sizing and proportions of the original `SidePanel`, resulting in a visually incorrect layout that was rejected. A second attempt to simply force the `SidePanel` to be visible by default also failed to produce the desired outcome.
*   **Resolution:** The most effective and simplest solution was to revert all layout changes to the original, working implementation. The code was restored to the state with a conditional `SidePanel`, and the user's request was re-evaluated and satisfied by ensuring the original, preferred behavior was stable.
*   **Lesson Learned:**
    1.  **Preserve Working States:** When a UI layout is considered good, preserve it. Drastic refactoring to achieve a seemingly small change can have unintended consequences on proportions and aesthetics. `git restore` is a critical tool for quickly reverting failed experiments.
    2.  **The Simplest Change is Often Best:** The goal was to have the panel always visible. The initial thought was a major layout refactor. However, a much simpler (though ultimately also reverted) approach was to just change the default state of the boolean controlling the panel's visibility. When a complex solution fails, reconsider the simplest possible approach.
    3.  **User Feedback is Paramount for UI:** UI/UX is subjective. A technically correct implementation may not be what the user wants. The "NO GO" from the user was a clear signal to stop the current path and revert to a known-good state before trying a different approach.

### JACK Connection Failure (Resolved)

*   **Original Problem:** The standalone application would consistently fail to connect to the JACK audio server on startup, logging errors like `Cannot connect to server socket`.
*   **Root Cause Analysis:** The issue was confirmed to be an environment problem, not a bug in the application's code. The JACK audio server was not running on the host system. The application was correctly detecting this and falling back to the ALSA backend as intended.
*   **Resolution:** The issue is resolved by ensuring the JACK server is running before launching the application. This can be done using a tool like `qjackctl` or by running `jackd` from the command line. No code changes were necessary.
*   **Lesson Learned:**
    1.  **Verify the Environment First:** When an application fails to connect to an external service, the first step is to verify that the service is running and accessible. In this case, a simple `pgrep jackd` confirmed the absence of the JACK server.
    2.  **Fallback Mechanisms are Working:** The application's ability to gracefully fall back to the ALSA backend when JACK is unavailable demonstrates that its audio backend handling is robust. The error messages were not a sign of a crash, but of a successful and informative failure detection.

### `egui` Slider Reset Logic (Resolved)

*   **Original Problem:** The double-click-to-reset functionality on the main UI sliders was not working. Multiple attempts to fix this by manually handling the `double_clicked()` and `changed()` events failed, leading to incorrect behavior or compilation errors.
*   **Root Cause Analysis:** The core issue was fighting the `nih-plug` framework instead of using its provided tools. The manual implementation attempted to re-create logic that was already built into the framework's own widgets. The `egui::Slider` is a generic widget, but `nih-plug` provides a "smart" `nih_plug_egui::widgets::ParamSlider` that is deeply integrated with the parameter system. Furthermore, when attempting to use `ParamSlider`, the incorrect constructor (`::new()`) was used instead of the correct one (`::for_param()`), which requires passing the `ParamSetter`.
*   **Resolution:** The manual `egui::Slider` and all associated event-handling logic were removed entirely. They were replaced with the idiomatic `nih_plug_egui::widgets::ParamSlider`.
    1.  The `use nih_plug_egui::widgets;` statement was added.
    2.  All sliders were changed to `ui.add(widgets::ParamSlider::for_param(&params.my_param, setter));`.
    3.  This widget correctly handles user interactions, including double-click-to-reset, and communicates with the `nih-plug` parameter system automatically. The `.with_value_to_string()` formatter on the `FloatParam` was also automatically respected, solving the dB display issue without any extra code.
*   **Lesson Learned:**
    1.  **Use the Framework's Widgets:** When a framework provides its own UI widgets (like `ParamSlider`), always prefer them over generic ones. They contain critical integration logic that is difficult and error-prone to replicate manually. The framework's widgets are the framework's intended API for UI interaction.
    2.  **Simplify to the Intended Path:** The repeated failures and increasing complexity of the manual solution were a strong signal that the approach was wrong. The solution was to simplify by removing all the manual code and reverting to the single, framework-provided component. If a solution feels like you're fighting the tool, you probably are.
    3.  **Check the Constructor Signature:** A quick look at the `ParamSlider` source or documentation would have revealed the correct `for_param(&param, &setter)` constructor, preventing the final build failure. The compiler's error message was the ultimate guide here.



### `egui-file-dialog` Usage Pattern (Resolved)

*   **Original Problem:** The file dialog windows for loading SOFA files and AutoEQ profiles stopped appearing, even though the `cargo tree` output showed no version conflicts between `egui`, `nih-plug`, and `egui-file-dialog`.
*   **Root Cause Analysis:** The issue was a subtle but critical logic error in how the `egui-file-dialog` library was being used. The code was calling `file_dialog.update(egui_ctx)` on one line, and then later calling `file_dialog.take_picked()` to check for a result. The `update()` method, however, returns a `Dialog` object which contains the result of the interaction for that frame. The correct pattern is to chain the method call to get the result directly from the object returned by `update()`. By not using the return value of `update()`, the result was being discarded, and the subsequent call to `take_picked()` would never find a selected file.
*   **Resolution:** The code was refactored to follow the correct usage pattern documented in the `egui-file-dialog` examples.
    *   The separate call to `state.file_dialog.update(egui_ctx);` was removed.
    *   The result-checking logic was changed from `if let Some(path) = state.file_dialog.take_picked()` to `if let Some(path) = state.file_dialog.update(egui_ctx).picked()`.
    *   Additionally, the code was corrected to use `path.to_path_buf()` when sending the result to the background task, as the `picked()` method returns a `Path` reference, while the task expected an owned `PathBuf`.
*   **Lesson Learned:**
    1.  **API Return Values Are Critical:** Always check the return values of library functions. The `egui-file-dialog` `update()` method is not just for updating internal state; its return value is the primary way to retrieve the result of the user's interaction for that frame.
    2.  **Consult Examples for Regressions:** When a previously working feature breaks, and dependency versions seem correct, the next step is to re-verify the implementation against the library's official and current usage examples. APIs can have subtle changes in their expected usage patterns between versions that are not always captured by the type system alone.
    3.  **Ownership Matters:** Pay close attention to the types returned by third-party libraries. The dialog returned a `&Path`, but the background task required a `PathBuf`. Implicitly relying on type coercion can hide bugs; explicit conversion with `.to_path_buf()` is safer.

### `nih-plug` GUI Parameter Setting (Resolved)

*   **Original Problem:** After successfully building a standalone executable, the application would log repeated warnings like `GuiContext::set_parameter() was called for parameter '...' without a preceding begin_set_parameter() call` whenever a UI control was used.
*   **Root Cause Analysis:** The issue stemmed from a deep misunderstanding of how `nih-plug`'s `egui` widgets interact with the `ParamSetter` context.
    1.  **Initial State:** The code did not wrap any UI controls with `begin_set_parameter()` or `end_set_parameter()`, causing the initial warnings.
    2.  **Incorrect Fix (Over-correction):** The first attempt to fix this involved wrapping every `widgets::ParamSlider` and `egui::ComboBox` with `begin/end` calls. This was incorrect and led to a new set of warnings: `begin_set_parameter() was called twice`.
    3.  **The Revelation (Reading the Source):** A direct inspection of the `nih_plug_egui/src/widgets/param_slider.rs` source code revealed the truth: the built-in widgets (`ParamSlider`, etc.) are "smart" and manage the `begin/end` calls **automatically** when the user interacts with them. My manual wrapping was conflicting with this internal behavior.
    4.  **Final Bug:** Even after removing the manual wrappers, the warnings persisted. This was traced to using the wrong widget for a boolean parameter (`widgets::ParamSlider` for a `BoolParam`) and later, to not wrapping programmatic parameter changes (like applying a preset) in `begin/end` calls.
*   **Resolution:** The final, correct solution involved several steps:
    1.  **Trust the Widgets:** All manual `begin/end` wrappers were removed from the `ParamSlider` and `ComboBox` widgets. They are designed to be used directly.
    2.  **Use the Right Widget:** The `widgets::ParamSlider` being used for the `eq_enable` `BoolParam` was replaced with a standard `egui::toggle_value`, and the parameter change was wrapped manually in `begin/end` calls.
    3.  **Wrap Manual Calls:** The warnings were ultimately originating from any button click or UI event that set parameter values programmatically. The fix was to wrap these blocks of manual `setter.set_parameter(...)` calls in the appropriate `begin/end` transactions.
*   **Lesson Learned:**
    1.  **The `nih-plug` Widgets Are Smart (Mostly):** The widgets provided by `nih_plug_egui` are deeply integrated with the parameter system. They handle the gestures of beginning, changing, and ending a parameter adjustment automatically. Do not manually wrap them.
    2.  **Differentiate UI-driven vs. Programmatic Changes:** The `begin/end` calls are necessary only when you are setting a parameter's value directly in your own code (e.g., applying a preset or using a standard `egui` widget), not when a user is interacting with a standard `nih-plug` widget.
    3.  **When in Doubt, Read the Source:** The cycle of conflicting warnings was a strong signal that my mental model of the framework was wrong. The definitive answer was not in the examples or high-level docs, but in the implementation of the widget itself.
    4.  **Use the Correct Widget for the Parameter Type:** `ParamSlider` is for `FloatParam` and `IntParam`. Using it with a `BoolParam` will cause unexpected behavior and warnings. Use standard `egui` widgets and manual `setter` calls for boolean parameters.

### `clap-validator` Recursion Warning & Background Task Architecture (Resolved)

*   **Original Problem:** The `clap-validator` tool reported a persistent warning: `The plugin recursively called 'clap_host::on_main_thread()'. Aborted after ten iterations.` This indicated an incorrect implementation of background tasks that, while not a crash, prevented the plugin from loading in some hosts.
*   **Root Cause Analysis:** The issue stemmed from a misunderstanding of the `nih-plug` threading model for background tasks initiated from the GUI.
    1.  An initial attempt assumed the GUI's `AsyncExecutor` could directly spawn background tasks. This was incorrect; the `create_sender()` method did not exist on that context.
    2.  A second attempt used a `crossbeam_channel` to send a task from the GUI to the background thread, and another channel to send the result back to the GUI. This caused the recursion warning. The background task would send the result, the GUI would receive it and update its state, triggering a redraw. The validator interpreted this immediate, event-driven redraw as a recursive call originating from the main thread.
*   **Resolution:** The architecture was refactored to follow the correct, unidirectional event flow and use shared state for results, breaking the recursive cycle.
    1.  **GUI-to-Audio-Thread Communication:** A `crossbeam_channel::Sender<Task>` is held by the `EditorState` (the GUI). When the user clicks a button to start a task, the GUI sends a `Task` enum variant over this channel.
    2.  **Audio-Thread-to-Background-Execution:** The main `OpenHeadstagePlugin` struct holds the `Receiver<Task>`. In its `process()` method (the real-time audio thread), it checks the receiver for new messages using `try_recv()`.
    3.  **Safe Task Execution:** Upon receiving a `Task` message, the `process()` method uses its `ProcessContext` to safely execute the task on a background thread via `context.execute_background(task)`. This is the *only* correct way to initiate a background task from a GUI interaction.
    4.  **Result Handling (Polling, not Pushing):** To return data (like parsed EQ settings) to the GUI without a direct callback, an `Arc<Mutex<Option<Vec<BandSetting>>>>` is used. This `Arc<Mutex<>>` is cloned and shared between the `EditorState` and the `Task` itself.
        *   The `task_executor` (background thread) performs the work, locks the mutex, and places the result inside the `Option`.
        *   The `editor` closure (GUI thread) polls this `Arc<Mutex<>>` on every redraw. It uses `lock().take()` to check for and retrieve the result. This polling mechanism decouples the background task from the GUI redraw, breaking the recursion. The GUI pulls data when it's ready, rather than having the data pushed to it.
*   **Lesson Learned:**
    1.  **The `nih-plug` Threading Pattern is Strict:** The only sanctioned way to trigger a background task from a GUI action is: **GUI -> Audio Thread -> Background**. The GUI must send a message to the audio thread, and the audio thread must use its `ProcessContext` to launch the task. There are no shortcuts.
    2.  **Avoid Direct Callbacks from Background to GUI:** Sending a message or using a direct callback from a background task to the GUI is an anti-pattern that can cause event loops. The `clap-validator` is sensitive to this and will correctly flag it as recursion.
    3.  **Polling Shared State is the Safe Return Path:** For a background task to return data to the GUI, it should write the result to a shared, thread-safe container (like an `Arc<Mutex<>>`). The GUI should then poll this container during its redraw cycle to check for and retrieve the data. This decouples the threads and prevents the recursion warning.

### Rust 2024 Edition Upgrade with `bindgen` (Resolved)

*   **Original Problem:** The upgrade to the Rust 2024 edition failed with compilation errors stating `extern blocks must be unsafe`. The errors originated from the FFI bindings file generated by the `bindgen` crate in `build.rs`.
*   **Root Cause Analysis:** The Rust 2024 edition enforces that all `extern "C" { ... }` blocks, which define foreign function interfaces, must be explicitly marked as `unsafe`. The `bindgen` configuration in our `build.rs` file was using a `.raw_line()` to manually inject a function definition, and this manually written line did not include the `unsafe` keyword. The standard `cargo fix --edition` tool did not automatically fix this because it was a string literal inside the build script, not source code it was analyzing directly.
*   **Resolution:**
    1.  The `bindgen` dependency in `Cargo.toml` was updated to a newer version as a first step, though this did not resolve the issue on its own.
    2.  The core of the fix was to modify the `.raw_line()` call in `build.rs` to prepend `unsafe` to the `extern "C"` block string.
*   **Lesson Learned:**
    1.  **`cargo fix` has limits:** The `cargo fix --edition` tool is powerful but cannot fix everything. It does not parse string literals in build scripts. Manually generated code, especially via `raw_line` in `build.rs`, must be audited and updated by hand during an edition migration.
    2.  **Build Scripts are Source Code:** The build script (`build.rs`) is a critical part of the compilation process and is subject to the same language rules and changes as the rest of the codebase. When a new edition introduces new requirements (like `unsafe extern`), build scripts must also be updated to comply.
    3.  **Isolate the Source:** The compiler error clearly pointed to the generated `mysofa_bindings.rs` file in the `target` directory. This was the key clue that the problem was not in the application source code (`src/...`) but in the code *generation* step, leading directly to an inspection of `build.rs`.

### Environment/Toolchain Incompatibility with `nih-plug` (Resolved)

*   **Original Problem:** The project failed to build in the CI environment because the available Rust compiler (1.75.0) was too old for the version of `nih-plug` being used, which required Rust 1.80+. Attempts to pin `nih-plug` to an older version failed due to git fetching errors in the CI.
*   **Resolution:** The CI environment's Rust toolchain was updated to a newer version (1.87.0). This immediately resolved the compilation failures.
*   **Lesson Learned:**
    1.  **Toolchain is Key:** The Rust compiler version is a critical dependency, especially when using cutting-edge libraries that pull from git repositories.
    2.  **CI Configuration:** The `rust-toolchain` file or the CI workflow script (`.github/workflows/rust_ci.yml`) is the canonical source for the required Rust version and must be kept in sync with the project's needs.
    3.  **Clear Error Messages:** The final build success was only possible after getting clear error messages. When builds fail, identifying the root cause (e.g., compiler version vs. code error) is the most important step.

### Cargo Feature Resolution for UI Dependencies (Resolved)

*   **Original Problem:** The project failed to compile with persistent errors related to `nih-plug` features, specifically the `gui` feature. The error message "the package `open-headstage` depends on `nih_plug`, with features: `gui` but `nih_plug` does not have these features" was misleading and caused a series of incorrect fix attempts, including using local path dependencies which introduced workspace conflicts.
*   **Resolution:**
    1.  The local `nih-plug` repository clone was removed to eliminate path and workspace conflicts.
    2.  The `Cargo.toml` dependencies for `nih_plug` and `nih_plug_egui` were changed from local paths to specific versions from `crates.io`.
    3.  The `gui` feature was correctly enabled directly on the `nih_plug` dependency line in `Cargo.toml`: `nih_plug = { version = "0.5.1", features = ["gui"] }`.
    4.  The `ui` feature in the local `[features]` section was corrected to only include the optional UI-specific crates: `ui = ["nih_plug_egui", "rfd"]`.
*   **Lesson Learned:**
    1.  **`crates.io` is the Source of Truth:** When facing intractable dependency or feature issues with a local git checkout, revert to a stable version from `crates.io`. This simplifies the build environment and eliminates potential inconsistencies in the local repository's `Cargo.toml`.
    2.  **Enabling Dependency Features:** To enable a feature for a dependency, it **must** be done on the dependency's line in `Cargo.toml`. The `[features]` section of the current crate is for defining its *own* features, which can then be used to conditionally enable dependencies or code via `cfg` attributes. The two are not interchangeable.
    3.  **Deconstruct the Error:** The error message was confusing. While it said `nih_plug` didn't have the `gui` feature, the real issue was the complex interaction of local paths, workspace conflicts, and an incorrectly defined local feature. Simplifying the situation (by removing the local path) was key to revealing the true, much simpler, solution.

### `nih-plug` Parameter System & Threading (Resolved)

*   **Original Problem:** A series of compilation errors arose from a misunderstanding of the `nih-plug` parameter system and thread-safety requirements.
    *   `StringParam` was used for the SOFA file path, but this parameter type does not exist in the current version of `nih-plug`.
    *   The `task_executor` function signature was incorrect, causing ownership and lifetime errors when trying to share plugin state (`MySofa`, `ConvolutionEngine`) with the background thread.
    *   Attempts to `Clone` `MySofa` failed because it contains a raw pointer and is not designed to be cloned.
*   **Resolution:**
    1.  **Replaced `StringParam`:** The non-existent `StringParam` was replaced with a `#[persist]` field: `pub sofa_file_path: Arc<RwLock<String>>`. This correctly handles persistent, non-parameter state.
    2.  **Corrected Thread Safety:** To safely share state between the audio processor, the UI, and the background task executor, `Arc<RwLock<>>` (for the SOFA path string) and `Arc<Mutex<>>` (for the `MySofa` loader) were used. This ensures safe interior mutability across threads.
    3.  **Fixed `task_executor`:** The `task_executor` was corrected to return a closure that captures the necessary thread-safe state (`Arc`-wrapped handles), resolving the ownership issues.
    4.  **Made `ConvolutionEngine` Clonable:** The `ConvolutionEngine` struct was made clonable (`#[derive(Clone)]`) to allow it to be moved into the background closure, although this was later refacored to avoid unnecessary cloning.
*   **Lesson Learned:**
    1.  **Parameters vs. Persistent State:** Use `nih-plug`'s `Param` types (`FloatParam`, etc.) only for values that are automated and controlled by the host. For other state that needs to be saved, like file paths or editor state, use `#[persist]` on a thread-safe container like an `Arc<RwLock<T>>`.
    2.  **Threading is Explicit:** When sharing state between the UI/background threads and the real-time audio thread, standard Rust thread-safety patterns (`Arc`, `Mutex`, `RwLock`) are required. The plugin's main struct fields must be designed for this from the start if they are to be shared.
    3.  **Consult the Source:** When a type or feature seems to be missing or causes confusing errors, the quickest path to a solution is to inspect the library's source code (`src/params.rs` in this case) to see the available types and their intended usage. `grep` is an invaluable tool for this.
    4.  **Compiler Errors Guide Architecture:** The series of ownership, lifetime, and      trait implementation errors were not just syntax problems; they were pointing to a          fundamental architectural issue in how state was being shared. Resolving them required      changing the data structures themselves, not just the function calls.

### CI Build Failures due to Missing System Dependencies (Resolved)

*   **Original Problem:** After successfully compiling locally, the CI build failed repeatedly with `pkg-config` errors. The build script could not find system libraries required by the GUI toolkit, such as `glib-2.0`, `atk`, and `gdk-3.0`.
*   **Resolution:** The issue was resolved by adding the main GTK3 development meta-package, `libgtk-3-dev`, to the `apt-get install` command in the `.github/workflows/rust_ci.yml` file. This single package automatically pulled in all the necessary dependencies (`glib`, `atk`, `gdk`, `pango`, etc.) that the UI crates rely on.
*   **Lesson Learned:**
    1.  **Install Comprehensive Dev Packages:** When dealing with large C/C++-based libraries or toolkits like GTK, it is more robust to install the primary development package (e.g., `libgtk-3-dev`) rather than trying to install each required sub-library individually. This avoids a "whack-a-mole" scenario where fixing one missing dependency reveals another.
    2.  **Local vs. CI Environments:** A successful local build does not guarantee a successful CI build. The CI environment is clean and will expose any missing system dependencies that might already be installed on a local development machine. The CI configuration must be treated as the canonical list of build requirements.

### GUI File Dialog Failures (Crash on Click)

*   **Original Problem:** The plugin, when loaded in a Qt-based host like Carla, would crash instantly upon clicking the "Select SOFA File" button.
*   **Root Cause Analysis:** The crash was caused by using the `rfd` (Rust File Dialog) crate directly within the plugin's GUI. `rfd` attempts to open a native GTK file dialog, which conflicts with the host's Qt event loop, leading to a segmentation fault. This is a fundamental incompatibility between GUI toolkits.
*   **Failed Solutions & Lessons:**
    1.  **Incorrect API Usage:** An attempt to use `nih-plug`'s host-based file dialogs failed because of incorrect API knowledge. The methods I tried to call (`show_open_dialog`, `show_file_dialog` on `AsyncExecutor`) do not exist. **Lesson:** The `nih-plug` book and official examples are the only reliable source for API usage. Do not guess or assume.
    2.  **Dependency Version Hell:** An attempt to use the `egui-file-dialog` crate failed due to a cascade of dependency version conflicts. The version of `egui-file-dialog` used an older version of `egui` than `nih-plug-egui` did, leading to dozens of compiler errors. **Lesson:** The `nih-plug` ecosystem is very sensitive to dependency versions. Before adding any new `egui`-related crate, use `cargo tree` to verify that it uses the *exact same version* of `egui` as `nih-plug-egui`. If not, it is incompatible.
    3.  **Misunderstanding of State vs. Parameters:** An attempt to change the file path from a persistent `Arc<RwLock<String>>` to a `StringParam` was incorrect. **Lesson:** As documented previously, `Param` types are for host-automatable values. Simple persistent state that is not automatable should use the `#[persist]` attribute on a standard thread-safe type.
*   **Final Lesson:** GUI integrations are a major source of complexity and failure. The path forward requires a slow, careful, example-driven approach. The immediate problem (the crash) is resolved by removing the call to `rfd`, but a robust file dialog solution will require finding a compatible version of `egui-file-dialog` or writing a custom implementation that respects the `nih-plug` architecture.

### Navigating `nih-plug` Git Dependencies & Feature Flags (Resolved)

*   **Outdated Info Warning:** This issue was resolved based on older versions of `nih-plug` and `egui`. Since the project has been upgraded to newer versions, the specific version numbers, commit hashes, and conflicts described here may no longer be relevant, although the general lessons about dependency management still apply.
*   **Original Problem:** A simple desire to upgrade the `egui` version to `0.32.0` led to a cascade of build failures. The core of the problem was a deep misunderstanding of how the `nih-plug` ecosystem handles dependencies and features, compounded by misleading information in previous "Lessons Learned" entries.
*   **Root Cause Analysis:** The failure was a result of several incorrect assumptions:
    1.  **`crates.io` vs. Git:** The initial assumption was that `nih-plug` and its related crates were on `crates.io`. This was false. They are exclusively git-based dependencies.
    2.  **Branch Naming:** Attempts to fix the git dependency by guessing branch names (`master`, `main`) failed because the repository's default branch was not what was expected, and cargo's cache was holding onto old, incorrect information.
    3.  **Feature Flag Location:** A key error was trying to enable the `gui` feature on the main `nih_plug` crate. The `gui` feature does not exist there. It is a feature of the `nih_plug_egui` crate, which is enabled by the project's own `ui` feature.
    4.  **Version Mismatches:** The final blocker was a version mismatch between the `egui` version required by the `nih-plug` git commit and the version required by the `egui-file-dialog` crate.
*   **Resolution (The Correct Procedure):**
    1.  **Use a Specific Commit Hash:** The most robust way to specify a git dependency is with a full commit hash in the `rev` field. This avoids all ambiguity with branch names. The command `git ls-remote https://github.com/robbert-vdh/nih-plug.git HEAD` was used to find the latest commit hash.
    2.  **Correct Feature Enablement:** The `Cargo.toml` was corrected to enable features on the correct crates. The `nih-plug` dependency itself only needs the `standalone` feature. The project's own `[features]` section defines the `ui` feature, which in turn enables the optional `nih_plug_egui` dependency.
    3.  **Align Dependency Versions:** The `egui` version conflict was resolved by downgrading `egui-file-dialog` to a version (`0.10.0`) that was compatible with the version of `egui` used by the `nih-plug` git commit. The explicit dependency on `egui` was removed from `Cargo.toml`, allowing the version to be determined by `nih_plug_egui`.
*   **Definitive Lessons Learned:**
    1.  **`nih-plug` is Git-Only:** Acknowledge that `nih-plug` and its ecosystem are not on `crates.io`. All related dependencies must be handled as `git` dependencies.
    2.  **Use `rev` with a Full Commit Hash:** When using a `git` dependency, always prefer specifying a full commit hash with `rev = "..."`. This is the most stable and reproducible method. Use `git ls-remote` to find the latest hash.
    3.  **Features Belong to Specific Crates:** Understand where features are defined. The `gui` functionality comes from `nih_plug_egui`, not `nih_plug`. Enable optional dependencies through your own crate's `[features]` section.
    4.  **`cargo tree` is Your Best Friend:** When facing dependency conflicts, `cargo tree -i <crate_name>` is the essential tool to see which versions are being pulled in and by which other dependencies. This is the key to resolving version mismatch errors.

### `egui` Dependency Version Conflict (Resolved)
*   **Outdated Info Warning:** This issue was resolved based on older versions of `nih-plug` and `egui`. Since the project has been upgraded to newer versions, the specific version numbers and conflicts described here may no longer be relevant, although the general lessons about dependency management still apply.
*   **Note:** This entry is now considered partially misleading. The core lesson about using `cargo tree` is correct, but the resolution incorrectly implies that `nih-plug` is on `crates.io`. The new entry, "Navigating `nih-plug` Git Dependencies & Feature Flags", contains the complete and correct procedure.
*   **Original Problem:** After deciding to use `egui-file-dialog`, the build failed with dozens of errors related to mismatched types and missing methods. The root cause was a dependency conflict: `nih-plug` (from git) used `egui` v0.31.1, while the chosen version of `egui-file-dialog` (v0.5.0) used `egui` v0.27.2.
*   **Resolution:**
    1.  Used `cargo tree` to explicitly identify the two conflicting `egui` versions.
    2.  Used web search to browse the `egui-file-dialog` repository and its `Cargo.toml` files for different tagged versions.
    3.  Identified that `egui-file-dialog` v0.10.0 depends on `egui` v0.31.0, which is compatible with `nih-plug`'s requirement.
    4.  Updated `Cargo.toml` to use this specific compatible version.
*   **Lesson Learned:**
    1.  **`cargo tree` is Essential:** For any dependency-related issue, especially in a complex ecosystem like `egui`, `cargo tree` is the first and most important diagnostic tool. It makes version conflicts immediately obvious.
    2.  **Manual Version Vetting:** When depending on libraries from git, you cannot rely on `crates.io`'s automatic semantic versioning. You must manually check the `Cargo.toml` of the git dependency to find its exact requirements, and then find compatible versions of any related libraries you wish to add.
    3.  **Check the Source:** The `Cargo.toml` of a crate is the ultimate source of truth for its dependencies. Browsing the file for specific tags in the git repository is a reliable way to find compatible versions.

### VST3 Bundling Failure (Resolved)

*   **Original Problem:** The VST3 plugin was not being generated by `cargo build` and was not loading in Carla, even though the CLAP plugin passed validation. The `cargo xtask bundle` command, which is the standard `nih-plug` way to create bundles, was also failing silently.
*   **Resolution:**
    1.  Created a `bundler.toml` file with the required metadata for the plugin. This did not fix the `xtask` issue.
    2.  Bypassed the `xtask` system entirely and created the VST3 bundle manually by creating the `.vst3` directory structure and copying the compiled `.so` file into it.
*   **Lesson Learned:**
    1.  **`xtask` is not guaranteed:** The `cargo xtask` system is a convention, not a requirement. If it fails, it may be due to subtle configuration or versioning issues.
    2.  **Manual Bundling is a Viable Alternative:** The VST3 and CLAP formats are just specific directory structures. Understanding this allows for manual creation of the bundles, which is a robust fallback when the automated tooling fails.
    3.  **Isolate the Problem:** The `clap-validator` tool was critical in proving that the core plugin code was correct, which allowed me to focus on the VST3 packaging as the source of the problem.

### CLAP Bundling Failure & Validation (Resolved)

*   **Original Problem:** The `cargo xtask bundle` command, intended to create the `.clap` plugin bundle, was failing silently (exiting with code 0 but producing no output).
*   **Resolution:**
    1.  A review of internal documentation, specifically `docs/research/CLAP Plugin Development Documentation.md`, confirmed that `cargo xtask bundle` is the correct command but can be unreliable. The document also implies that manual bundling is a valid alternative.
    2.  Following the lesson from the previous VST3 bundling failure, the `xtask` system was bypassed in favor of manual bundling.
    3.  The `.clap` bundle was created by making a directory named `open-headstage.clap`.
    4.  The compiled shared library (`libopen_headstage.so`) was copied into this directory.
    5.  The `clap-validator` tool was then run. It initially failed because it was pointed at the `open-headstage.clap` directory.
    6.  The final, successful validation was achieved by pointing the validator directly at the shared library *inside* the bundle: `clap-validator validate open-headstage.clap/libopen_headstage.so`.
*   **Lesson Learned:**
    1.  **Consult Internal Docs First:** The solution was hinted at in existing project documentation. Always search the `docs/` directory before starting external searches or trial-and-error debugging.
    2.  **`xtask` can fail silently for CLAP too:** The unreliability of the `xtask` bundler is not specific to one format. Manual bundling is a consistent workaround.
    3.  **Validator Targets the Library:** Plugin validators (`clap-validator`, etc.) operate on the compiled library file (`.so`, `.dll`), not the bundle directory that contains it.
*   **Supporting Documentation:** `docs/research/CLAP Plugin Development Documentation.md`

### Architectural Misunderstanding of `nih-plug` Testing & Benchmarking (Resolved)

*   **Original Problem:** All attempts to create a `criterion`-based benchmark for the plugin's `process()` function failed with compilation errors. The errors indicated that the testing utility functions being used (`create_test_plugin`, `create_test_buffer`) did not exist and that core structs like `AuxiliaryBuffers` could not be instantiated.
*   **Root Cause Analysis:** The attempts were based on a fundamental misunderstanding of the `nih-plug` testing philosophy. I was assuming a conventional, in-process unit testing model where the test code is responsible for mocking a host environment and creating test objects. This is incorrect. The `nih-plug` framework has evolved to a higher-fidelity, out-of-process integration testing model.
*   **The Correct Paradigm (Standalone Execution):**
    1.  The canonical way to test, debug, and benchmark a `nih-plug` plugin is to compile it as a standalone application.
    2.  This is enabled by the `standalone` feature flag in `Cargo.toml` and the `nih_export_standalone()` macro.
    3.  This standalone application acts as a minimal host that connects to a real audio backend like the JACK Audio Connection Kit.
    4.  In this model, the developer does not create mock `Buffer` or `AuxiliaryBuffers` objects. Instead, the plugin *receives* real buffers from the JACK server via the standalone host wrapper.
*   **Resolution:** The problem was resolved by abandoning the flawed in-process benchmark attempt and adopting the standalone execution model. A new research document, `docs/research/Benchmarking-nih-plug.md`, was created to document this correct paradigm.
*   **Lesson Learned:**
    1.  **Verify the Paradigm, Not Just the API:** Before trying to use a framework's API, first understand its underlying philosophy and architecture. My focus on finding specific functions led me astray because the entire paradigm I was assuming was wrong.
    2.  **The `standalone` Feature is the Key to Testing:** The `standalone` feature is not merely for convenience; it is the entry point to the entire testing, debugging, and profiling workflow for `nih-plug`.
    3.  **Trust the Source, Not Outdated Docs:** The discrepancy arose from relying on outdated information. A forensic analysis of the current `master` branch source code was the only way to uncover the truth. The official, bundled examples are the most reliable source of current best practices.
    4.  **If It Feels Too Hard, You're Probably Doing It Wrong:** The immense difficulty I had trying to manually construct test objects was a signal that I was fighting the framework's design. The canonical method is much simpler because it delegates the hard work to the framework and the JACK server.

### Standalone Build Failure due to `crate-type` (Resolved)

*   **Original Problem:** After correctly configuring the project to build a standalone executable (`src/main.rs`), the build repeatedly failed with an `unresolved import` error. The binary crate (`main.rs`) was unable to find the library crate (`lib.rs`), even though the `use` statement (`use open_headstage::...`) was correct according to the hyphen-to-underscore rule.
*   **Root Cause Analysis:** The `Cargo.toml` file contained a `[lib]` section that explicitly set the library's `crate-type` to `["cdylib"]`. This instruction tells `rustc` to *only* compile the library as a C-style dynamic library, which is suitable for being loaded by a plugin host. However, it prevents the compiler from also generating the standard Rust library file (`.rlib`) that is required for other Rust crates (like our `main.rs` binary) to link against it. The binary had no library file to link to, causing the "unresolved import" error at the linking stage.
*   **Resolution:** The `crate-type` in `Cargo.toml` was modified to `["cdylib", "rlib"]`. This instructs Cargo to produce *both* the C-dynamic library for plugin hosts and the Rust library for other Rust crates.
*   **Lesson Learned:**
    1.  **`crate-type` is a Critical Override:** Explicitly setting `crate-type` in `Cargo.toml` completely overrides the default build outputs. If you need to link a Rust binary against your library (for testing, standalone executables, etc.), you **must** include `"rlib"` in the `crate-type` array.
    2.  **The Build System is the Final Arbiter:** When source code appears correct but linking fails, the issue is almost certainly in the build configuration (`Cargo.toml`). The "unresolved import" error, in this case, was a symptom of a linking failure, not a module path error.
    3.  **Follow the Checklist:** This issue was the final, most obscure item on the troubleshooting checklist from the `Rust Import Error Deep Dive.md` document. A systematic, step-by-step diagnosis is essential for solving complex build problems.

### Strategic Project Phasing & Planning

*   **Original Problem:** The project's `TODO.md` was structured chronologically based on when ideas were added, not by strategic priority. This led to a plan where complex, low-level DSP work was scheduled before high-value, user-facing features that could be implemented with existing code.
*   **Resolution:** The `TODO.md` was completely re-ordered into a more logical sequence.
    1.  **Legal & Licensing:** Moved to the beginning to de-risk the project early.
    2.  **High-Value Features (AutoEQ):** Prioritized to deliver maximum user value with minimal new engineering.
    3.  **Complex Core Features (Advanced EQ):** Grouped together into a dedicated phase.
    4.  **Optimization:** Moved to the end, as it should be done after all features are implemented.
*   **Lesson Learned:**
    1.  **Prioritize by Value vs. Effort:** The most effective project plan prioritizes tasks that deliver the highest user value for the lowest implementation effort. The AutoEQ integration is a perfect example of this.
    2.  **Foundation First:** Foundational, risk-reducing tasks like legal review and license management should be addressed early, even if they aren't user-facing features.
    3.  **Regularly Re-evaluate the Plan:** A project plan is a living document. It should be reviewed and re-prioritized regularly to adapt to new information, completed tasks, and a better understanding of the remaining work.

### Strategic Licensing & The "Open Core" Model

*   **Situation:** The project initially aimed to support the VST3 plugin format. A routine dependency audit revealed that the `vst3-sys` crate is licensed under `GPLv3` due to the VST3 SDK's own licensing terms. This created a direct conflict with the goal of maintaining long-term flexibility and the potential for future commercialization.

*   **Lesson Learned 1: Plugin format dictates the project's license.** The choice to support a specific plugin format is not just a technical decision but a critical legal one. The VST3 SDK's GPLv3 license option forces any open-source project using it to adopt the GPLv3 license as well. This "copyleft" nature must be evaluated at the very beginning of a project, as it has profound implications for all future development and distribution.

*   **Lesson Learned 2: Prioritize permissive licenses for future flexibility.** For a project that may have future commercial ambitions (even uncertain ones), a permissive license (like Apache-2.0 or MIT) is the superior choice. It allows the copyright holder to re-license the code for commercial products later. A copyleft license like GPLv3 permanently closes this door. We specifically chose **Apache-2.0** over MIT for its explicit patent grant clause, which offers stronger protection against patent-related lawsuits for both the project and its users.

*   **Lesson Learned 3: The "Open Core" model requires a CLA.** The strategy to balance a free, open-source project with future proprietary modules is the "Open Core" model. The key legal tool to enable this is a **Contributor License Agreement (CLA)**. A CLA ensures that the project owner retains full rights to re-license all contributed code, which is essential for creating a commercial version without needing to get permission from every past contributor.
