// src/dsp/convolution.rs

use rustfft::num_complex::Complex;
use rustfft::{Fft, FftPlanner}; // FftDirection
use std::sync::Arc;

const FFT_SIZE: usize = 2048; // Example FFT size, should be configurable or dynamic

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
pub struct ConvolutionPathData {
    ir_fft: Vec<Complex<f32>>,
    overlap_buffer: Vec<f32>,
    ir_len: usize, // Store original IR length for overlap calculation
}

impl ConvolutionPathData {
    // Helper to initialize with a default IR (e.g. passthrough or mute)
    fn default_new(ir: &[f32], fft_size: usize, forward_fft: &Arc<dyn Fft<f32>>) -> Self {
        let ir_len = ir.len();
        let mut padded_ir = ir.to_vec();
        padded_ir.resize(fft_size, 0.0);

        let mut ir_buffer_fft = padded_ir.iter().map(|&x| Complex::new(x, 0.0)).collect::<Vec<_>>();
        forward_fft.process(&mut ir_buffer_fft);

        Self {
            ir_fft: ir_buffer_fft,
            overlap_buffer: vec![0.0; fft_size], // Overlap for Overlap-Add (N_fft - block_size, but simplified for now)
            ir_len,
        }
    }
}

pub struct ConvolutionEngine {
    path_lsl: ConvolutionPathData,
    path_lsr: ConvolutionPathData,
    path_rsl: ConvolutionPathData,
    path_rsr: ConvolutionPathData,

    forward_fft: Arc<dyn Fft<f32>>,
    inverse_fft: Arc<dyn Fft<f32>>,
    // fft_planner: FftPlanner<f32>, // Keep planner for potential re-planning if config changes. Removed for Debug derive.
                                  // Re-introduce if re-planning logic is added, with manual Debug impl.

    // Temporary storage for FFT processing of input blocks
    input_fft_buffer: Vec<Complex<f32>>,
    // Temporary storage for the result of IFFT
    ifft_buffer: Vec<Complex<f32>>,
}

// Manual Debug implementation to exclude non-Debug fields (fft_planner, forward_fft, inverse_fft)
impl std::fmt::Debug for ConvolutionEngine {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ConvolutionEngine")
            .field("path_lsl", &self.path_lsl)
            .field("path_lsr", &self.path_lsr)
            .field("path_rsl", &self.path_rsl)
            .field("path_rsr", &self.path_rsr)
            // .field("forward_fft", &"Omitted Arc<dyn Fft<f32>>") // Or skip entirely
            // .field("inverse_fft", &"Omitted Arc<dyn Fft<f32>>")
            // .field("fft_planner", &"Omitted FftPlanner<f32>")
            .field("input_fft_buffer", &self.input_fft_buffer)
            .field("ifft_buffer", &self.ifft_buffer)
            .finish()
    }
}


impl ConvolutionEngine {
    /// Creates a new `ConvolutionEngine` with default IRs.
    /// Default IRs are passthrough for LSL and RSR, mute for LSR and RSL.
    pub fn new() -> Self {
        let mut planner = FftPlanner::<f32>::new();
        let forward_fft = planner.plan_fft_forward(FFT_SIZE);
        let inverse_fft = planner.plan_fft_inverse(FFT_SIZE);

        let ir_passthrough = vec![1.0];
        let ir_mute = vec![0.0];

        Self {
            path_lsl: ConvolutionPathData::default_new(&ir_passthrough, FFT_SIZE, &forward_fft),
            path_lsr: ConvolutionPathData::default_new(&ir_mute, FFT_SIZE, &forward_fft),
            path_rsl: ConvolutionPathData::default_new(&ir_mute, FFT_SIZE, &forward_fft),
            path_rsr: ConvolutionPathData::default_new(&ir_passthrough, FFT_SIZE, &forward_fft),
            forward_fft,
            inverse_fft,
            // fft_planner: planner, // Store if needed for re-planning
            input_fft_buffer: vec![Complex::new(0.0,0.0); FFT_SIZE],
            ifft_buffer: vec![Complex::new(0.0,0.0); FFT_SIZE],
        }
    }

    /// Sets the impulse response (IR) for a specific convolution path.
    /// This involves padding the IR, performing FFT, and storing the spectrum.
    pub fn set_ir(&mut self, path: ConvolutionPath, ir_data: Vec<f32>) {
        // Ensure IR is not excessively long for the chosen FFT_SIZE.
        // A more robust implementation might re-plan FFT if IR + block_size > FFT_SIZE.
        // For now, let's assume ir_data.len() is reasonable.
        // if ir_data.len() > FFT_SIZE {
        //     // Handle error: IR too long for current FFT_SIZE
        //     // Or, truncate, or re-plan. For now, let's assume it fits.
        //     nih_plug::nih_log!("Warning: IR length {} exceeds FFT_SIZE {}", ir_data.len(), FFT_SIZE);
        // }

        let target_path_data = match path {
            ConvolutionPath::LSL => &mut self.path_lsl,
            ConvolutionPath::LSR => &mut self.path_lsr,
            ConvolutionPath::RSL => &mut self.path_rsl,
            ConvolutionPath::RSR => &mut self.path_rsr,
        };

        target_path_data.ir_len = ir_data.len();

        let mut padded_ir = ir_data;
        padded_ir.resize(FFT_SIZE, 0.0);

        // Convert to complex for FFT
        let mut complex_ir_buffer: Vec<Complex<f32>> = padded_ir.iter().map(|&x| Complex::new(x, 0.0)).collect();

        // Perform forward FFT
        self.forward_fft.process(&mut complex_ir_buffer);
        target_path_data.ir_fft = complex_ir_buffer;

        // Initialize/reset overlap buffer. Its length depends on the Overlap-Add strategy.
        // For Overlap-Add, overlap length is FFT_SIZE - block_size.
        // Initialize overlap_buffer to all zeros. It's correctly sized in default_new and here.
        target_path_data.overlap_buffer.clear();
        target_path_data.overlap_buffer.resize(FFT_SIZE, 0.0); // Resize and fill with 0.0
    }

    fn convolve_path_fft(
        input_signal: &[f32],
        path_data: &mut ConvolutionPathData, // Contains IR_fft and overlap_buffer
        forward_fft: &Arc<dyn Fft<f32>>,
        inverse_fft: &Arc<dyn Fft<f32>>,
        input_fft_buffer: &mut Vec<Complex<f32>>,
        ifft_buffer: &mut Vec<Complex<f32>>,
        output_buffer: &mut [f32],
        block_size: usize,
    ) {
        // Basic Overlap-Add Implementation Sketch:

        // 1. Pad input_signal to FFT_SIZE, store in a temporary buffer (e.g. first part of input_fft_buffer)
        // For now, assume input_fft_buffer is pre-sized to FFT_SIZE and cleared/filled appropriately.
        for i in 0..block_size {
            input_fft_buffer[i] = Complex::new(input_signal[i], 0.0);
        }
        for i in block_size..FFT_SIZE {
            input_fft_buffer[i] = Complex::new(0.0, 0.0);
        }

        // 2. Perform forward FFT on the padded input block
        forward_fft.process(input_fft_buffer);

        // 3. Complex multiplication: input_fft_buffer * path_data.ir_fft
        // Store result in ifft_buffer (or reuse input_fft_buffer if careful)
        for i in 0..input_fft_buffer.len() { // Should be FFT_SIZE or FFT_SIZE/2+1 if using rfft/irfft
            ifft_buffer[i] = input_fft_buffer[i] * path_data.ir_fft[i];
        }

        // 4. Perform inverse FFT on the result
        inverse_fft.process(ifft_buffer);

        // 5. Add overlap from previous block & store new overlap (Overlap-Add)
        // Scale output of IFFT by 1/FFT_SIZE
        let scale = 1.0 / FFT_SIZE as f32;
        for i in 0..block_size {
            output_buffer[i] = (ifft_buffer[i].re * scale) + path_data.overlap_buffer[i];
        }

        // Save the tail for the next block's overlap
        // The length of this tail is FFT_SIZE - block_size
        for i in 0..(FFT_SIZE - block_size) {
            path_data.overlap_buffer[i] = ifft_buffer[i + block_size].re * scale;
        }
        // Zero out the rest of the overlap buffer if it's larger
        for i in (FFT_SIZE - block_size)..path_data.overlap_buffer.len() {
             path_data.overlap_buffer[i] = 0.0;
        }


        // This is a simplified version. Correct Overlap-Add requires careful handling
        // of buffer lengths and indices.
        // For example, if ir_len is small, overlap_buffer might be ir_len - 1.
        // If block_size + ir_len - 1 <= FFT_SIZE, then the output of IFFT is block_size + ir_len - 1 long.
        // The first block_size samples are output, the next ir_len - 1 are saved for overlap.

        // For now, we're using a fixed FFT_SIZE and simplified overlap handling.
        // output_buffer is assumed to be block_size. The actual convolved output for one block is block_size samples.
    }

    // This method is no longer used.
    // fn convolve_path_direct(
    //     input_signal: &[f32],
    //     ir: &[f32],
    //     delay_line: &mut [f32],
    //     output_buffer: &mut [f32],
    // ) {
    //     if ir.is_empty() {
    //         for val in output_buffer.iter_mut() {
    //             *val = 0.0;
    //         }
    //         return;
    //     }

    //     let ir_len = ir.len();
    //     let delay_len = delay_line.len();

    //     assert_eq!(
    //         input_signal.len(),
    //         output_buffer.len(),
    //         "Input and output buffer lengths must match."
    //     );
    //     assert_eq!(
    //         delay_len,
    //         ir_len.saturating_sub(1),
    //         "Delay line length mismatch for IR."
    //     );

    //     for i in 0..input_signal.len() {
    //         let mut accumulator = 0.0;
    //         let current_input_sample = input_signal[i];

    //         accumulator += current_input_sample * ir[0];

    //         for k in 1..ir_len {
    //             if k - 1 < delay_len {
    //                 // Should always be true if assert_eq passed
    //                 accumulator += delay_line[k - 1] * ir[k];
    //             }
    //         }
    //         output_buffer[i] = accumulator;

    //         // Refined delay line update
    //         if delay_len > 0 {
    //             delay_line.rotate_right(1); // Shift all elements one position to the right
    //             delay_line[0] = current_input_sample; // Insert current sample at the beginning
    //         }
    //     }
    // }

    pub fn process_block(
        &mut self,
        input_left: &[f32],
        input_right: &[f32],
        output_left: &mut [f32],
        output_right: &mut [f32],
    ) {
        let block_size = input_left.len();
        // Assertions for buffer lengths (already present, good)
        assert_eq!(input_right.len(), block_size, "Input right channel length mismatch.");
        // ... other assertions ...

        // Ensure internal FFT buffers are correctly sized (though fixed for now)
        // These should ideally be allocated once or when FFT_SIZE changes.
        // For this example, assuming they are managed in new() or when FFT_SIZE changes.
        // self.input_fft_buffer.resize(FFT_SIZE, Complex::new(0.0,0.0));
        // self.ifft_buffer.resize(FFT_SIZE, Complex::new(0.0,0.0));


        let mut lsl_out_block = vec![0.0; block_size];
        let mut lsr_out_block = vec![0.0; block_size];
        let mut rsl_out_block = vec![0.0; block_size];
        let mut rsr_out_block = vec![0.0; block_size];

        // LSL Path
        Self::convolve_path_fft(
            input_left,
            &mut self.path_lsl,
            &self.forward_fft,
            &self.inverse_fft,
            &mut self.input_fft_buffer,
            &mut self.ifft_buffer,
            &mut lsl_out_block,
            block_size,
        );

        // LSR Path
        Self::convolve_path_fft(
            input_left,
            &mut self.path_lsr,
            &self.forward_fft,
            &self.inverse_fft,
            &mut self.input_fft_buffer, // Re-use buffer
            &mut self.ifft_buffer,    // Re-use buffer
            &mut lsr_out_block,
            block_size,
        );

        // RSL Path
        Self::convolve_path_fft(
            input_right, // Process right input
            &mut self.path_rsl,
            &self.forward_fft,
            &self.inverse_fft,
            &mut self.input_fft_buffer, // Re-use buffer
            &mut self.ifft_buffer,    // Re-use buffer
            &mut rsl_out_block,
            block_size,
        );

        // RSR Path
        Self::convolve_path_fft(
            input_right, // Process right input
            &mut self.path_rsr,
            &self.forward_fft,
            &self.inverse_fft,
            &mut self.input_fft_buffer, // Re-use buffer
            &mut self.ifft_buffer,    // Re-use buffer
            &mut rsr_out_block,
            block_size,
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
    // use rustfft::num_complex::Complex; // No longer needed here after test changes

    const TOLERANCE: f32 = 1e-5; // FFT can have slightly higher error

    fn assert_approx_eq_slice(a: &[f32], b: &[f32], tolerance: f32, message: &str) {
        assert_eq!(a.len(), b.len(), "Slices have different lengths: {}", message);
        for (i, (val_a, val_b)) in a.iter().zip(b.iter()).enumerate() {
            assert!(
                (val_a - val_b).abs() < tolerance,
                "Mismatch at index {} for '{}': {} (expected) != {} (actual) (tolerance {})",
                i, message, val_a, val_b, tolerance
            );
        }
    }

    // Helper to create an IR for testing (e.g. identity, delay)
    fn make_ir(samples: &[f32]) -> Vec<f32> {
        samples.to_vec()
    }

    // Helper to get a path's IR FFT for advanced checks (might not be needed for typical tests)
    // fn get_ir_fft(engine: &ConvolutionEngine, path: ConvolutionPath) -> &Vec<Complex<f32>> {
    //     match path {
    //         ConvolutionPath::LSL => &engine.path_lsl.ir_fft,
    //         ConvolutionPath::LSR => &engine.path_lsr.ir_fft,
    //         ConvolutionPath::RSL => &engine.path_rsl.ir_fft,
    //         ConvolutionPath::RSR => &engine.path_rsr.ir_fft,
    //     }
    // }

    #[test]
    fn test_default_trait_produces_valid_engine() {
        let mut engine_default = ConvolutionEngine::default();
        // Test if it can process a block without panicking
        let input_l = vec![0.0; 128];
        let input_r = vec![0.0; 128];
        let mut output_l = vec![0.0; 128];
        let mut output_r = vec![0.0; 128];
        engine_default.process_block(&input_l, &input_r, &mut output_l, &mut output_r);
        // Further checks could verify passthrough behavior of default.
    }

    #[test]
    fn test_new_engine_is_passthrough() {
        let mut engine = ConvolutionEngine::new();
        let input_l: Vec<f32> = (1..=64).map(|x| x as f32 * 0.1).collect();
        let input_r: Vec<f32> = (1..=64).map(|x| x as f32 * -0.1).collect();
        let mut output_l = vec![0.0; input_l.len()];
        let mut output_r = vec![0.0; input_r.len()];

        engine.process_block(&input_l, &input_r, &mut output_l, &mut output_r);

        // Default IR for LSL and RSR is [1.0], LSR/RSL is [0.0]
        // This should result in passthrough for LSL and RSR.
        // Due to FFT scaling and potential windowing effects (though none here),
        // perfect bit-for-bit passthrough might be tricky with FFT convolution.
        // Small tolerance is expected.
        assert_approx_eq_slice(&output_l, &input_l, TOLERANCE, "Default engine LSL passthrough");
        assert_approx_eq_slice(&output_r, &input_r, TOLERANCE, "Default engine RSR passthrough");
    }

    #[test]
    fn test_set_ir_updates_path_properties() {
        let mut engine = ConvolutionEngine::new();
        let test_ir_samples = vec![0.1, 0.2, 0.3];
        let original_ir_len = test_ir_samples.len();
        engine.set_ir(ConvolutionPath::LSL, test_ir_samples.clone());

        assert_eq!(engine.path_lsl.ir_len, original_ir_len, "LSL ir_len after set_ir");
        assert!(!engine.path_lsl.ir_fft.is_empty(), "LSL ir_fft should be populated");
        // IR FFT length should be FFT_SIZE for full complex FFT, or FFT_SIZE/2+1 for RFFT.
        // Current impl uses full complex FFT via `plan_fft_forward`.
        assert_eq!(engine.path_lsl.ir_fft.len(), FFT_SIZE, "LSL ir_fft length");

        // Check if overlap buffer is initialized (e.g., to zeros and correct length)
        // It's set to FFT_SIZE in set_ir.
        assert_eq!(engine.path_lsl.overlap_buffer.len(), FFT_SIZE, "LSL overlap_buffer length");
        assert!(engine.path_lsl.overlap_buffer.iter().all(|&x| x == 0.0), "LSL overlap_buffer initialized to zeros");

        // Test with an empty IR - should effectively mute the path.
        // An empty IR would mean ir_len = 0. Its FFT would be all zeros.
        engine.set_ir(ConvolutionPath::LSR, vec![]);
        assert_eq!(engine.path_lsr.ir_len, 0, "LSR ir_len for empty IR");
        assert!(engine.path_lsr.ir_fft.iter().all(|c| c.re == 0.0 && c.im == 0.0), "LSR ir_fft for empty IR should be zeros");
    }


    #[test]
    fn test_identity_ir_single_path_passthrough_fft() {
        let mut engine = ConvolutionEngine::new(); // Uses FFT_SIZE
        let block_size = 128; // Typical block size

        // Set LSL to identity, others to mute
        engine.set_ir(ConvolutionPath::LSL, make_ir(&[1.0]));
        engine.set_ir(ConvolutionPath::LSR, make_ir(&[0.0]));
        engine.set_ir(ConvolutionPath::RSL, make_ir(&[0.0]));
        engine.set_ir(ConvolutionPath::RSR, make_ir(&[0.0]));

        let input_signal: Vec<f32> = (1..=block_size).map(|x| x as f32 * 0.1).collect();
        let silent_input: Vec<f32> = vec![0.0; block_size];
        let mut output_left = vec![0.0; block_size];
        let mut output_right = vec![0.0; block_size];

        // Process one block
        engine.process_block(&input_signal, &silent_input, &mut output_left, &mut output_right);
        assert_approx_eq_slice(&output_left, &input_signal, TOLERANCE, "Identity LSL FFT block 1");
        assert_approx_eq_slice(&output_right, &silent_input, TOLERANCE, "Mute paths output_right FFT block 1");

        // Process another block to check if state (overlap) is handled correctly
        let input_signal2: Vec<f32> = (block_size+1..=2*block_size).map(|x| x as f32 * 0.1).collect();
        let mut output_left2 = vec![0.0; block_size];
        let mut output_right2 = vec![0.0; block_size];
        engine.process_block(&input_signal2, &silent_input, &mut output_left2, &mut output_right2);
        assert_approx_eq_slice(&output_left2, &input_signal2, TOLERANCE, "Identity LSL FFT block 2");
    }

    #[test]
    fn test_delay_ir_single_path_fft() {
        let mut engine = ConvolutionEngine::new();
        let block_size = 5; // Small block size for easier manual tracing
        let delay_samples = 1;
        let ir_delay = {
            let mut v = vec![0.0; delay_samples + 1];
            v[delay_samples] = 1.0; // IR for a delay of 'delay_samples'
            make_ir(&v)
        };

        engine.set_ir(ConvolutionPath::LSL, ir_delay);
        engine.set_ir(ConvolutionPath::LSR, make_ir(&[0.0]));
        engine.set_ir(ConvolutionPath::RSL, make_ir(&[0.0]));
        engine.set_ir(ConvolutionPath::RSR, make_ir(&[0.0]));

        let input_block1 = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let expected_block1_out = vec![0.0, 1.0, 2.0, 3.0, 4.0]; // Delayed by 1 sample

        let silent_input: Vec<f32> = vec![0.0; block_size];
        let mut output_l1 = vec![0.0; block_size];
        let mut output_r1 = vec![0.0; block_size];

        engine.process_block(&input_block1, &silent_input, &mut output_l1, &mut output_r1);
        // The first sample of output is from overlap_buffer, initially zero.
        // The first input sample (1.0) will appear as the second output sample if delay=1.
        assert_approx_eq_slice(&output_l1, &expected_block1_out, TOLERANCE, "1-sample delay LSL FFT, block 1");

        let input_block2 = vec![6.0, 7.0, 8.0, 9.0, 10.0];
        let expected_block2_out = vec![5.0, 6.0, 7.0, 8.0, 9.0];
        let mut output_l2 = vec![0.0; block_size];
        let mut output_r2 = vec![0.0; block_size];
        engine.process_block(&input_block2, &silent_input, &mut output_l2, &mut output_r2);
        assert_approx_eq_slice(&output_l2, &expected_block2_out, TOLERANCE, "1-sample delay LSL FFT, block 2");
    }

    #[test]
    fn test_stereo_interaction_default_passthrough_fft() {
        let mut engine = ConvolutionEngine::new(); // Default is passthrough LSL/RSR
        let block_size = 128;
        let input_l: Vec<f32> = (1..=block_size).map(|x| x as f32 * 0.1).collect();
        let input_r: Vec<f32> = (1..=block_size).map(|x| x as f32 * -0.2).collect();
        let mut output_l = vec![0.0; block_size];
        let mut output_r = vec![0.0; block_size];

        engine.process_block(&input_l, &input_r, &mut output_l, &mut output_r);
        assert_approx_eq_slice(&output_l, &input_l, TOLERANCE, "Stereo passthrough FFT output_l");
        assert_approx_eq_slice(&output_r, &input_r, TOLERANCE, "Stereo passthrough FFT output_r");
    }

    #[test]
    fn test_empty_ir_mutes_path_fft() {
        let mut engine = ConvolutionEngine::new();
        let block_size = 64;
        engine.set_ir(ConvolutionPath::LSL, make_ir(&[])); // Empty IR for LSL
        engine.set_ir(ConvolutionPath::RSR, make_ir(&[1.0])); // RSR passthrough

        let input_signal: Vec<f32> = (1..=block_size).map(|x| x as f32).collect();
        let mut output_l = vec![0.0; block_size];
        let mut output_r = vec![0.0; block_size];

        engine.process_block(&input_signal, &input_signal, &mut output_l, &mut output_r);

        let expected_output_l = vec![0.0; block_size]; // LSL is empty, RSL is default mute ([0.0])
        // RSR is passthrough, LSR is default mute ([0.0])
        let expected_output_r = input_signal.clone();

        assert_approx_eq_slice(&output_l, &expected_output_l, TOLERANCE, "Empty LSL IR mutes L-channel output from L-input");
        assert_approx_eq_slice(&output_r, &expected_output_r, TOLERANCE, "Empty LSL IR does not affect RSR path");
    }

    // More tests to consider:
    // - IR longer than block_size.
    // - IR much shorter than block_size.
    // - Multiple blocks to verify correct overlap handling over time (already in delay test).
    // - Specific filter shapes (e.g., simple averaging filter).
    // - Test with block_size that is not a power of two, if supported (FFT_SIZE is fixed power of two).
    // - Test with various FFT_SIZE values if it becomes configurable.
}
