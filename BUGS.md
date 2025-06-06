# Known Issues & Development Status

## Current Development Blocker

*   **Title:** Environment/Toolchain Incompatibility with `nih-plug`
*   **Status:** `[Open]`
*   **Details:**
    *   Current environment: Rustc 1.75.0. Latest `nih-plug` requires Rustc 1.80+.
    *   Attempts to use older `nih-plug` versions via `tag` or `rev` failed due to git fetching errors (`couldn't find remote ref` / `revspec not found`).
    *   This blocks compilation/bundling in the CI environment.
    *   Development proceeds with structural changes; compilation/testing is deferred to the user's local environment.
*   **Impact:** Blocks automated testing & CI builds.

## Development Status Summary

**Compilation Status:** `⚠️ Blocked in CI (Rustc 1.75 vs nih-plug 1.80+; git fetch issues). User compiles locally.`
**Testing Status:** `⚠️ Unit tests in CI blocked. DSP logic tests might run standalone.`
**Framework/DSP/SOFA Integration:** `✅ Structurally integrated. User compiles locally.`

**Next Development Priorities:**
0.  **User to test local compilation of integrated modules.**
1.  User to test CLAP bundling & host loading locally.
2.  Refine `process()` DSP logic.
3.  UI development (pending local compilation).
4.  Finalize SOFA loading (structural part done).

**Development Velocity:** Structural changes only in CI. Functional testing depends on user's local setup.
---
*Older resolved issues (Cargo.toml example, NIH-Plug API signatures, earlier macro issues, SOFA error handling) are omitted for brevity but were previously addressed.*