# Known Issues & Development Status

This document tracks known bugs, limitations, and the overall development status of the Open Headstage project.

---

## Resolved Issues & Lessons Learned

### Intractable Real-Time Allocation Crashes (Resolved)

*   **Original Problem:** The application suffered from a series of seemingly unrelated, intractable memory allocation crashes on the real-time audio thread. These crashes manifested in different libraries (`rustfft`, `biquad`) but shared the same root cause: performing expensive, allocating operations inside the `process()` loop.
*   **Root Cause Analysis:** This was a systemic architectural failure rooted in a misunderstanding of real-time audio safety.
    1.  **`rustfft` Allocation:** The initial crashes were in `rustfft`. Deep research revealed that the convenient `Fft::process()` method **always allocates a temporary scratch buffer**. The correct, allocation-free method is `Fft::process_with_scratch()`, which requires the caller to provide a pre-allocated scratch buffer. My initial fixes failed because I was resizing the *data* buffer, not providing the required *scratch* buffer.
    2.  **`biquad` Coefficient Allocation:** After fixing the FFT crash, a new allocation crash appeared in the `biquad` library. This was traced to calling `update_band_coeffs()` on every audio sample. The coefficient calculation function (`Coefficients::from_params`) allocates memory.
*   **Resolution (The Final, Correct Architecture):** The entire audio processing pipeline was refactored to strictly separate expensive, one-time calculations from the per-sample processing loop.
    1.  **Command Queue for State Changes:** A `crossbeam_channel` was introduced. The UI thread is the *only* place that can change parameters using the `ParamSetter`. When it does so (e.g., applying a loaded EQ profile), it also sends a `ParamChange` message to the audio thread.
    2.  **One-Time Updates in `process()`:** The `process()` loop's first action is to check the channel for new messages. If a message is received, it performs the expensive, allocating operations *once* (e.g., calling `update_band_coeffs`). Because this is a single, user-initiated event and not a continuous operation, it does not violate real-time safety.
    3.  **Allocation-Free Per-Sample Loop:** The main body of the `process()` loop now only contains operations that are guaranteed to be allocation-free: reading smoothed parameter values and calling the DSP `process_block` methods.
*   **Lesson Learned:**
    1.  **The `process()` loop is sacred:** Absolutely no function that can allocate memory (including library functions that calculate coefficients, resize buffers, etc.) should be called on every audio sample.
    2.  **Read Library Docs Carefully:** The `rustfft` documentation was clear about `process()` vs. `process_with_scratch()`. A failure to read it carefully cost significant time.
    3.  **Use a Command Queue for UI-to-Audio Communication:** The correct pattern for applying complex, user-initiated changes to the audio thread is to use a message queue. The UI sets the parameters and sends a message. The audio thread receives the message and performs the one-time heavy lifting to update its internal state (e.g., filter coefficients).

### AutoEQ Parser Failure (Resolved)

*   **Original Problem:** The application failed to parse AutoEQ `.txt` files, logging "missing field 'Filter-Type'" errors.
*   **Root Cause Analysis:** The initial implementation incorrectly assumed the `.txt` files were in a CSV format and used a `csv` reader with `serde`. A direct inspection of the file revealed it was a custom, space-delimited format. A subsequent rewrite had an off-by-one error when indexing the split string, causing "invalid float literal" errors.
*   **Resolution:** The parser was completely rewritten to be a simple, line-by-line manual parser.
    1.  It iterates over lines, checking for "Preamp:" or "Filter" prefixes.
    2.  It uses `split_whitespace()` to tokenize the line.
    3.  It parses the required values from the correct token indices.
*   **Lesson Learned:**
    1.  **Never Assume a File Format:** Always inspect a sample of the data file before writing a parser. The `.txt` extension was not indicative of a simple CSV.
    2.  **Manual Parsing is Sometimes Simpler:** For simple, custom formats, a manual line-by-line parser can be more robust and easier to debug than trying to force a library like a CSV reader to fit a format it wasn't designed for.
    3.  **Index with Care:** Off-by-one errors are common in manual parsing. Double-check token indices against a sample line of the source data.

---
*The previous "Resolved Issues" have been preserved below for historical context.*

### The Persistence Pitfall: A Deep Dive into `nih-plug` State Management (Resolved)
...