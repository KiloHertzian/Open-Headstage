// src/dsp/convolution.rs

/// Enum to identify one of the four convolution paths in a binaural setup.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConvolutionPath {
    LSL, // Left input to Left ear/output
    LSR, // Left input to Right ear/output
    RSL, // Right input to Left ear/output
    RSR, // Right input to Right ear/output
}

/// Manages four convolution paths for binaural processing.
///
/// This engine takes stereo input, convolves each input channel with two
/// separate impulse responses (IRs) per ear, and sums the results
/// to produce a stereo output.
///
/// Paths:
/// - LSL: Left input convolved with IR for Left ear.
/// - LSR: Left input convolved with IR for Right ear.
/// - RSL: Right input convolved with IR for Left ear.
/// - RSR: Right input convolved with IR for Right ear.
///
/// Output:
/// - Left Output  = Result(LSL) + Result(RSL)
/// - Right Output = Result(LSR) + Result(RSR)
#[derive(Debug)] // Added Debug for easier inspection
pub struct ConvolutionEngine {
    ir_lsl: Vec<f32>,
    ir_lsr: Vec<f32>,
    ir_rsl: Vec<f32>,
    ir_rsr: Vec<f32>,

    // Delay lines for direct convolution.
    // The length of each delay line is ir_len - 1.
    // Stores past input samples: delay_line[0] is x[n-1], delay_line[1] is x[n-2], etc.
    delay_lsl: Vec<f32>,
    delay_lsr: Vec<f32>,
    delay_rsl: Vec<f32>,
    delay_rsr: Vec<f32>,
}

impl ConvolutionEngine {
    /// Creates a new `ConvolutionEngine` with default IRs.
    ///
    /// Default IRs are passthrough for LSL (Left->Left) and RSR (Right->Right),
    /// and mute for LSR (Left->Right) and RSL (Right->Left), effectively
    /// creating a standard stereo passthrough.
    /// Delay lines are initialized according to these default IRs.
    pub fn new() -> Self {
        let ir_lsl = vec![1.0]; // Passthrough
        let ir_lsr = vec![0.0]; // Mute
        let ir_rsl = vec![0.0]; // Mute
        let ir_rsr = vec![1.0]; // Passthrough

        Self {
            delay_lsl: vec![0.0; ir_lsl.len().saturating_sub(1)],
            delay_lsr: vec![0.0; ir_lsr.len().saturating_sub(1)],
            delay_rsl: vec![0.0; ir_rsl.len().saturating_sub(1)],
            delay_rsr: vec![0.0; ir_rsr.len().saturating_sub(1)],
            ir_lsl,
            ir_lsr,
            ir_rsl,
            ir_rsr,
        }
    }

    /// Sets the impulse response (IR) for a specific convolution path.
    ///
    /// The corresponding delay line will be resized and cleared.
    /// If `ir_data` is empty, the path will effectively be muted (as if IR was `[0.0]`).
    ///
    /// # Arguments
    /// * `path` - The `ConvolutionPath` to set the IR for.
    /// * `ir_data` - A `Vec<f32>` containing the IR samples.
    pub fn set_ir(&mut self, path: ConvolutionPath, ir_data: Vec<f32>) {
        let (target_ir, target_delay_line) = match path {
            ConvolutionPath::LSL => (&mut self.ir_lsl, &mut self.delay_lsl),
            ConvolutionPath::LSR => (&mut self.ir_lsr, &mut self.delay_lsr),
            ConvolutionPath::RSL => (&mut self.ir_rsl, &mut self.delay_rsl),
            ConvolutionPath::RSR => (&mut self.ir_rsr, &mut self.delay_rsr),
        };

        *target_ir = ir_data;

        let delay_len = target_ir.len().saturating_sub(1);
        target_delay_line.clear();
        target_delay_line.resize(delay_len, 0.0);
    }

    fn convolve_path_direct(
        input_signal: &[f32],
        ir: &[f32],
        delay_line: &mut [f32],
        output_buffer: &mut [f32],
    ) {
        if ir.is_empty() {
            for val in output_buffer.iter_mut() {
                *val = 0.0;
            }
            return;
        }

        let ir_len = ir.len();
        let delay_len = delay_line.len();

        assert_eq!(
            input_signal.len(),
            output_buffer.len(),
            "Input and output buffer lengths must match."
        );
        assert_eq!(
            delay_len,
            ir_len.saturating_sub(1),
            "Delay line length mismatch for IR."
        );

        for i in 0..input_signal.len() {
            let mut accumulator = 0.0;
            let current_input_sample = input_signal[i];

            accumulator += current_input_sample * ir[0];

            for k in 1..ir_len {
                if k - 1 < delay_len {
                    // Should always be true if assert_eq passed
                    accumulator += delay_line[k - 1] * ir[k];
                }
            }
            output_buffer[i] = accumulator;

            // Refined delay line update
            if delay_len > 0 {
                delay_line.rotate_right(1); // Shift all elements one position to the right
                delay_line[0] = current_input_sample; // Insert current sample at the beginning
            }
        }
    }

    pub fn process_block(
        &mut self,
        input_left: &[f32],
        input_right: &[f32],
        output_left: &mut [f32],
        output_right: &mut [f32],
    ) {
        let block_size = input_left.len();
        assert_eq!(
            input_right.len(),
            block_size,
            "Input right channel length mismatch."
        );
        assert_eq!(
            output_left.len(),
            block_size,
            "Output left channel length mismatch."
        );
        assert_eq!(
            output_right.len(),
            block_size,
            "Output right channel length mismatch."
        );

        let mut lsl_out_block = vec![0.0; block_size];
        let mut lsr_out_block = vec![0.0; block_size];
        let mut rsl_out_block = vec![0.0; block_size];
        let mut rsr_out_block = vec![0.0; block_size];

        Self::convolve_path_direct(
            input_left,
            &self.ir_lsl,
            &mut self.delay_lsl,
            &mut lsl_out_block,
        );
        Self::convolve_path_direct(
            input_left,
            &self.ir_lsr,
            &mut self.delay_lsr,
            &mut lsr_out_block,
        );
        Self::convolve_path_direct(
            input_right,
            &self.ir_rsl,
            &mut self.delay_rsl,
            &mut rsl_out_block,
        );
        Self::convolve_path_direct(
            input_right,
            &self.ir_rsr,
            &mut self.delay_rsr,
            &mut rsr_out_block,
        );

        for i in 0..block_size {
            output_left[i] = lsl_out_block[i] + rsl_out_block[i];
            output_right[i] = lsr_out_block[i] + rsr_out_block[i];
        }
    }
}

// Implement Default trait
impl Default for ConvolutionEngine {
    fn default() -> Self {
        Self::new()
    }
}

// Unit Tests
#[cfg(test)]
mod tests {
    use super::*;

    const TOLERANCE: f32 = 1e-6;

    fn assert_approx_eq_slice(a: &[f32], b: &[f32], tolerance: f32, message: &str) {
        assert_eq!(
            a.len(),
            b.len(),
            "Slices have different lengths: {}",
            message
        );
        for (i, (val_a, val_b)) in a.iter().zip(b.iter()).enumerate() {
            assert!(
                (val_a - val_b).abs() < tolerance,
                "Mismatch at index {} for '{}': {} != {} (tolerance {})",
                i,
                message,
                val_a,
                val_b,
                tolerance
            );
        }
    }

    #[test]
    fn test_default_trait() {
        let engine_default = ConvolutionEngine::default();
        let engine_new = ConvolutionEngine::new();
        // Check if a few key fields are the same, assuming new() sets up correctly
        assert_eq!(
            engine_default.ir_lsl, engine_new.ir_lsl,
            "Default LSL IR matches new()"
        );
        assert_eq!(
            engine_default.ir_rsr, engine_new.ir_rsr,
            "Default RSR IR matches new()"
        );
        assert_eq!(
            engine_default.delay_lsl.len(),
            engine_new.delay_lsl.len(),
            "Default LSL delay len matches new()"
        );
        // A more thorough test would involve processing a block, but this checks construction.
        let input_l: Vec<f32> = vec![1.0, 2.0];
        let input_r: Vec<f32> = vec![0.1, 0.2];
        let mut output_l_default = vec![0.0; 2];
        let mut output_r_default = vec![0.0; 2];
        let mut output_l_new = vec![0.0; 2];
        let mut output_r_new = vec![0.0; 2];

        // Process with default-initialized engine
        let mut engine_default_mut = ConvolutionEngine::default();
        engine_default_mut.process_block(
            &input_l,
            &input_r,
            &mut output_l_default,
            &mut output_r_default,
        );

        // Process with new-initialized engine
        let mut engine_new_mut = ConvolutionEngine::new();
        engine_new_mut.process_block(&input_l, &input_r, &mut output_l_new, &mut output_r_new);

        assert_approx_eq_slice(
            &output_l_default,
            &output_l_new,
            TOLERANCE,
            "Output from default() matches new() for left channel",
        );
        assert_approx_eq_slice(
            &output_r_default,
            &output_r_new,
            TOLERANCE,
            "Output from default() matches new() for right channel",
        );
    }

    #[test]
    fn test_new_engine_defaults() {
        let engine = ConvolutionEngine::new();
        assert_eq!(engine.ir_lsl, vec![1.0], "Default LSL IR");
        assert_eq!(engine.ir_lsr, vec![0.0], "Default LSR IR");
        assert_eq!(engine.ir_rsl, vec![0.0], "Default RSL IR");
        assert_eq!(engine.ir_rsr, vec![1.0], "Default RSR IR");
        assert!(engine.delay_lsl.is_empty(), "Default LSL delay line");
        assert!(engine.delay_lsr.is_empty(), "Default LSR delay line");
        assert!(engine.delay_rsl.is_empty(), "Default RSL delay line");
        assert!(engine.delay_rsr.is_empty(), "Default RSR delay line");
    }

    #[test]
    fn test_set_ir_updates_ir_and_delay_line() {
        let mut engine = ConvolutionEngine::default(); // Use default for variety
        let test_ir = vec![0.1, 0.2, 0.3];
        engine.set_ir(ConvolutionPath::LSL, test_ir.clone());

        assert_eq!(engine.ir_lsl, test_ir, "LSL IR after set_ir");
        assert_eq!(
            engine.delay_lsl.len(),
            test_ir.len() - 1,
            "LSL delay line length after set_ir"
        );
        assert!(
            engine.delay_lsl.iter().all(|&x| x == 0.0),
            "LSL delay line initialized to zeros"
        );

        let identity_ir = vec![1.0];
        engine.set_ir(ConvolutionPath::RSR, identity_ir.clone());
        assert_eq!(engine.ir_rsr, identity_ir, "RSR IR after set_ir");
        assert_eq!(engine.delay_rsr.len(), 0, "RSR delay line for identity IR");

        engine.set_ir(ConvolutionPath::LSR, vec![]);
        assert!(
            engine.ir_lsr.is_empty(),
            "LSR IR after set_ir with empty vec"
        );
        assert!(engine.delay_lsr.is_empty(), "LSR delay line for empty IR");
    }

    #[test]
    fn test_identity_ir_single_path_passthrough() {
        let mut engine = ConvolutionEngine::default();
        engine.set_ir(ConvolutionPath::LSL, vec![1.0]);
        engine.set_ir(ConvolutionPath::LSR, vec![0.0]);
        engine.set_ir(ConvolutionPath::RSL, vec![0.0]);
        engine.set_ir(ConvolutionPath::RSR, vec![0.0]);

        let input_signal: Vec<f32> = (1..=5).map(|x| x as f32).collect();
        let silent_input: Vec<f32> = vec![0.0; input_signal.len()];
        let mut output_left = vec![0.0; input_signal.len()];
        let mut output_right = vec![0.0; input_signal.len()];

        engine.process_block(
            &input_signal,
            &silent_input,
            &mut output_left,
            &mut output_right,
        );

        assert_approx_eq_slice(
            &output_left,
            &input_signal,
            TOLERANCE,
            "Identity LSL path output_left",
        );
        assert_approx_eq_slice(
            &output_right,
            &silent_input,
            TOLERANCE,
            "Identity LSL path output_right (should be silent)",
        );

        let input_signal2: Vec<f32> = (6..=10).map(|x| x as f32).collect();
        let mut output_left2 = vec![0.0; input_signal2.len()];
        let mut output_right2 = vec![0.0; input_signal2.len()];
        engine.process_block(
            &input_signal2,
            &silent_input,
            &mut output_left2,
            &mut output_right2,
        );
        assert_approx_eq_slice(
            &output_left2,
            &input_signal2,
            TOLERANCE,
            "Identity LSL path second block",
        );
    }

    #[test]
    fn test_delay_ir_single_path() {
        let mut engine = ConvolutionEngine::default();
        let delay_ir = vec![0.0, 1.0];
        engine.set_ir(ConvolutionPath::LSL, delay_ir.clone());
        engine.set_ir(ConvolutionPath::LSR, vec![0.0]);
        engine.set_ir(ConvolutionPath::RSL, vec![0.0]);
        engine.set_ir(ConvolutionPath::RSR, vec![0.0]);

        let input_signal: Vec<f32> = vec![1.0, 2.0, 3.0, 0.0, 0.0];
        let expected_output: Vec<f32> = vec![0.0, 1.0, 2.0, 3.0, 0.0];

        let silent_input: Vec<f32> = vec![0.0; input_signal.len()];
        let mut output_left = vec![0.0; input_signal.len()];
        let mut output_right = vec![0.0; input_signal.len()];

        engine.process_block(
            &input_signal,
            &silent_input,
            &mut output_left,
            &mut output_right,
        );
        assert_approx_eq_slice(
            &output_left,
            &expected_output,
            TOLERANCE,
            "1-sample delay LSL path",
        );
        assert_approx_eq_slice(
            &output_right,
            &silent_input,
            TOLERANCE,
            "1-sample delay LSL path (output_right silent)",
        );
    }

    #[test]
    fn test_simple_filter_ir_single_path() {
        let mut engine = ConvolutionEngine::default();
        let filter_ir = vec![0.5, 0.5];
        engine.set_ir(ConvolutionPath::LSL, filter_ir.clone());
        engine.set_ir(ConvolutionPath::LSR, vec![0.0]);
        engine.set_ir(ConvolutionPath::RSL, vec![0.0]);
        engine.set_ir(ConvolutionPath::RSR, vec![0.0]);

        let input_signal: Vec<f32> = vec![1.0, 2.0, 3.0, 4.0, 0.0];
        let expected_output: Vec<f32> = vec![0.5, 1.5, 2.5, 3.5, 2.0];

        let silent_input: Vec<f32> = vec![0.0; input_signal.len()];
        let mut output_left = vec![0.0; input_signal.len()];
        let mut output_right = vec![0.0; input_signal.len()];

        engine.process_block(
            &input_signal,
            &silent_input,
            &mut output_left,
            &mut output_right,
        );
        assert_approx_eq_slice(
            &output_left,
            &expected_output,
            TOLERANCE,
            "Simple filter LSL path",
        );
    }

    #[test]
    fn test_stereo_interaction_default_passthrough() {
        let mut engine = ConvolutionEngine::default();

        let input_l: Vec<f32> = vec![1.0, 2.0, 3.0];
        let input_r: Vec<f32> = vec![0.1, 0.2, 0.3];

        let mut output_l = vec![0.0; input_l.len()];
        let mut output_r = vec![0.0; input_r.len()];

        engine.process_block(&input_l, &input_r, &mut output_l, &mut output_r);

        assert_approx_eq_slice(
            &output_l,
            &input_l,
            TOLERANCE,
            "Stereo default passthrough output_l",
        );
        assert_approx_eq_slice(
            &output_r,
            &input_r,
            TOLERANCE,
            "Stereo default passthrough output_r",
        );
    }

    #[test]
    fn test_stereo_interaction_full_mix() {
        let mut engine = ConvolutionEngine::default();
        let ir_val = vec![1.0];
        engine.set_ir(ConvolutionPath::LSL, ir_val.clone());
        engine.set_ir(ConvolutionPath::LSR, ir_val.clone());
        engine.set_ir(ConvolutionPath::RSL, ir_val.clone());
        engine.set_ir(ConvolutionPath::RSR, ir_val.clone());

        let input_l: Vec<f32> = vec![1.0, 2.0, 3.0];
        let input_r: Vec<f32> = vec![0.1, 0.2, 0.3];

        let mut output_l_buf = vec![0.0; input_l.len()];
        let mut output_r_buf = vec![0.0; input_r.len()];

        engine.process_block(&input_l, &input_r, &mut output_l_buf, &mut output_r_buf);

        let expected_sum_output: Vec<f32> = input_l
            .iter()
            .zip(input_r.iter())
            .map(|(l, r)| l + r)
            .collect();

        assert_approx_eq_slice(
            &output_l_buf,
            &expected_sum_output,
            TOLERANCE,
            "Stereo full mix output_l",
        );
        assert_approx_eq_slice(
            &output_r_buf,
            &expected_sum_output,
            TOLERANCE,
            "Stereo full mix output_r",
        );
    }

    #[test]
    fn test_stereo_interaction_specific_paths_with_delay() {
        let mut engine = ConvolutionEngine::default();
        let identity_ir = vec![1.0];
        let delay_ir_short = vec![0.0, 1.0];

        engine.set_ir(ConvolutionPath::LSL, identity_ir.clone());
        engine.set_ir(ConvolutionPath::LSR, vec![0.0]);
        engine.set_ir(ConvolutionPath::RSL, vec![0.0]);
        engine.set_ir(ConvolutionPath::RSR, delay_ir_short.clone());

        let input_l: Vec<f32> = vec![1.0, 2.0, 3.0, 0.0];
        let input_r: Vec<f32> = vec![5.0, 6.0, 7.0, 0.0];

        let mut output_l = vec![0.0; input_l.len()];
        let mut output_r = vec![0.0; input_r.len()];

        engine.process_block(&input_l, &input_r, &mut output_l, &mut output_r);

        let expected_output_l = input_l.clone();
        let expected_output_r: Vec<f32> = vec![0.0, 5.0, 6.0, 7.0];

        assert_approx_eq_slice(
            &output_l,
            &expected_output_l,
            TOLERANCE,
            "Specific paths: output_l",
        );
        assert_approx_eq_slice(
            &output_r,
            &expected_output_r,
            TOLERANCE,
            "Specific paths: output_r (with delay)",
        );
    }

    #[test]
    fn test_process_multiple_blocks_maintains_delay_state() {
        let mut engine = ConvolutionEngine::default();
        let delay_ir = vec![0.0, 0.0, 1.0];
        engine.set_ir(ConvolutionPath::LSL, delay_ir.clone());
        engine.set_ir(ConvolutionPath::LSR, vec![0.0]);
        engine.set_ir(ConvolutionPath::RSL, vec![0.0]);
        engine.set_ir(ConvolutionPath::RSR, vec![0.0]);

        let silent_input_block = vec![0.0; 2];

        let input_block1: Vec<f32> = vec![1.0, 2.0];
        let mut output_block1_l = vec![0.0; input_block1.len()];
        let mut output_block1_r = vec![0.0; input_block1.len()];
        engine.process_block(
            &input_block1,
            &silent_input_block,
            &mut output_block1_l,
            &mut output_block1_r,
        );
        assert_approx_eq_slice(
            &output_block1_l,
            &[0.0, 0.0],
            TOLERANCE,
            "Multi-block delay: block 1 output",
        );

        let input_block2: Vec<f32> = vec![3.0, 4.0];
        let mut output_block2_l = vec![0.0; input_block2.len()];
        let mut output_block2_r = vec![0.0; input_block2.len()];
        engine.process_block(
            &input_block2,
            &silent_input_block,
            &mut output_block2_l,
            &mut output_block2_r,
        );
        assert_approx_eq_slice(
            &output_block2_l,
            &[1.0, 2.0],
            TOLERANCE,
            "Multi-block delay: block 2 output",
        );

        let input_block3: Vec<f32> = vec![0.0, 0.0];
        let mut output_block3_l = vec![0.0; input_block3.len()];
        let mut output_block3_r = vec![0.0; input_block3.len()];
        engine.process_block(
            &input_block3,
            &silent_input_block,
            &mut output_block3_l,
            &mut output_block3_r,
        );
        assert_approx_eq_slice(
            &output_block3_l,
            &[3.0, 4.0],
            TOLERANCE,
            "Multi-block delay: block 3 output (flush)",
        );
    }

    #[test]
    fn test_empty_ir_results_in_silence_for_that_path() {
        let mut engine = ConvolutionEngine::default(); // Starts with LSL=1, LSR=0, RSL=0, RSR=1
        engine.set_ir(ConvolutionPath::LSL, vec![]); // Empty IR for LSL
                                                     // RSR is still vec![1.0] from default

        let input_signal: Vec<f32> = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let mut output_left = vec![0.0; input_signal.len()];
        let mut output_right = vec![0.0; input_signal.len()];

        engine.process_block(
            &input_signal,
            &input_signal,
            &mut output_left,
            &mut output_right,
        );

        let expected_output_left = vec![0.0; input_signal.len()]; // LSL is empty, RSL is [0.0]
        let expected_output_right = input_signal.clone(); // LSR is [0.0], RSR is [1.0] (passthrough for input_right)

        assert_approx_eq_slice(
            &output_left,
            &expected_output_left,
            TOLERANCE,
            "Empty LSL IR should mute left output contribution from left input",
        );
        assert_approx_eq_slice(
            &output_right,
            &expected_output_right,
            TOLERANCE,
            "Empty LSL IR should not affect RSR path on right output",
        );
    }
}
