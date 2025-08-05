<canon>
**Directive:** This document is a living history of the project. Completed tasks and phases must never be removed. They serve as a record of progress and decision-making.
</canon>

**Verification Protocol:** Before any task involving user-facing changes (UI, UX, or core functionality) is marked as complete (`[x]`), the user (Project Lead) must give an explicit "GO" after a verification run. A "NO GO" will result in the task being re-opened to address the feedback.

# TODO List

This file tracks the development tasks for the Open Headstage project.

**NOTE:** Tasks marked as complete (`[x]`) are functionally integrated but may require further verification and hardening as the project evolves.

---
## Current Priority

- [ ] **[UI-BUG] Restore Missing SOFA and PEQ Controls:**
    - **Description:** In the process of integrating the AutoEQ search feature, the original controls in the "Headphone Equalization" section (the "Select SOFA File" button and the "Edit Parametric EQ" button) were accidentally removed. They need to be restored.
    - **Priority:** Highest.

- [ ] **[CLEANUP] Remove Unused Imports and Variables:**
    - **Description:** The last build produced several warnings for unused imports and variables. Run `cargo fix` and perform a manual review to clean up the codebase.
    - **Priority:** Low.

---
## Completed Tasks (History)

- [x] **[FEATURE] Implement Dynamic AutoEQ Search and Update:**
    - **Description:** Implement the full-featured headphone EQ search system.
    - **Priority:** Highest.
    - **Sub-tasks:**
        - [x] **[BUILD]** Create a build script to scan the `PRESERVE/AutoEq` directory and generate a `headphone_index.json`.
        - [x] **[DATA]** Load the generated index at startup and store it in a thread-safe container.
        - [x] **[UI]** Implement a debounced search bar to filter the headphone index.
        - [x] **[TASK]** Create a background task to load and parse a selected `ParametricEQ.txt` file.
        - [x] **[TASK]** Create a background task to run `git pull` to update the local AutoEQ database.
        - [x] **[ARCH]** Implement a real-time safe message passing system (`ParamChange`) to apply loaded EQ profiles to the audio thread.
        - [x] **[BUG]** Fix all real-time allocation crashes related to `rustfft` and `biquad` coefficient calculation.
        - [x] **[BUG]** Rewrite the `autoeq_parser` to correctly handle the format's `Preamp:` line and space-delimited filter lines.
        - [x] **[UI]** Restore the main plugin GUI and integrate the search feature into a collapsible panel.
        - [x] **[UX]** Improve search results by prioritizing "oratory1990" and displaying the measurement source.
    - **Status:** DONE.

- [x] **[DEBUG] Verify Persistent Saving of Audio Backend Settings:**
...
