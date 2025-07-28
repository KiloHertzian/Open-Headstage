// Copyright 2025 SignalVerse
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

// src/dsp/convolution.rs

use rustfft::num_complex::Complex;
use rustfft::{Fft, FftPlanner};
use std::sync::Arc;

// Configuration for partitioned convolution
const BLOCK_SIZE: usize = 512; // Internal processing block size
const FFT_SIZE: usize = BLOCK_SIZE * 2; // FFT size, typically 2 * block_size for 50% overlap-add

/// Enum to identify one of the four convolution paths in a binaural setup.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(dead_code)]
pub enum ConvolutionPath {
    Lsl,
    Lsr,
    Rsl,
    Rsr,
}

/// Holds the data for a single convolution path, adapted for partitioned convolution.
#[derive(Debug, Clone)]
struct ConvolutionPathData {
    ir_fft_partitions: Vec<Vec<Complex<f32>>>,
    input_fft_history: Vec<Vec<Complex<f32>>>,
    history_index: usize,
    overlap_buffer: Vec<f32>,
}

impl ConvolutionPathData {
    fn new(fft_plan: &Arc<dyn Fft<f32>>) -> Self {
        // Default to a single partition representing silence (or passthrough if IR is [1.0])
        let default_ir = vec![0.0f32; BLOCK_SIZE];
        // default_ir[0] = 1.0; // for passthrough

        let mut padded_ir = default_ir;
        padded_ir.resize(FFT_SIZE, 0.0);
        let mut ir_fft = padded_ir
            .into_iter()
            .map(|s| Complex::new(s, 0.0))
            .collect::<Vec<_>>();
        fft_plan.process(&mut ir_fft);

        Self {
            ir_fft_partitions: vec![ir_fft],
            input_fft_history: vec![vec![Complex::new(0.0, 0.0); FFT_SIZE]],
            history_index: 0,
            overlap_buffer: vec![0.0; BLOCK_SIZE],
        }
    }
}

/// Manages four convolution paths for binaural processing using partitioned convolution.
#[derive(Clone)]
pub struct ConvolutionEngine {
    paths: [ConvolutionPathData; 4],

    forward_fft: Arc<dyn Fft<f32>>,
    inverse_fft: Arc<dyn Fft<f32>>,

    // Buffers for handling variable host block sizes
    input_buffer_l: Vec<f32>,
    input_buffer_r: Vec<f32>,
    output_buffer_l: Vec<f32>,
    output_buffer_r: Vec<f32>,

    // Temporary buffers for FFT processing
    input_fft_buffer: Vec<Complex<f32>>,
    conv_accumulator: Vec<Complex<f32>>,
}

impl ConvolutionEngine {
    pub fn new() -> Self {
        let mut planner = FftPlanner::<f32>::new();
        let forward_fft = planner.plan_fft_forward(FFT_SIZE);
        let inverse_fft = planner.plan_fft_inverse(FFT_SIZE);

        Self {
            paths: [
                ConvolutionPathData::new(&forward_fft), // LSL
                ConvolutionPathData::new(&forward_fft), // LSR
                ConvolutionPathData::new(&forward_fft), // RSL
                ConvolutionPathData::new(&forward_fft), // RSR
            ],
            forward_fft,
            inverse_fft,
            input_buffer_l: Vec::with_capacity(BLOCK_SIZE * 2),
            input_buffer_r: Vec::with_capacity(BLOCK_SIZE * 2),
            output_buffer_l: Vec::with_capacity(BLOCK_SIZE * 2),
            output_buffer_r: Vec::with_capacity(BLOCK_SIZE * 2),
            input_fft_buffer: vec![Complex::new(0.0, 0.0); FFT_SIZE],
            conv_accumulator: vec![Complex::new(0.0, 0.0); FFT_SIZE],
        }
    }

    #[allow(dead_code)]
    pub fn set_ir(&mut self, path: ConvolutionPath, ir_data: &[f32]) {
        let path_data = &mut self.paths[path as usize];

        if ir_data.is_empty() {
            // Handle empty IR (mute)
            let mut silent_part = vec![Complex::new(0.0, 0.0); FFT_SIZE];
            self.forward_fft.process(&mut silent_part);
            path_data.ir_fft_partitions = vec![silent_part];
        } else {
            path_data.ir_fft_partitions = ir_data
                .chunks(BLOCK_SIZE)
                .map(|ir_chunk| {
                    let mut padded_chunk = ir_chunk.to_vec();
                    padded_chunk.resize(FFT_SIZE, 0.0);
                    let mut complex_chunk = padded_chunk
                        .into_iter()
                        .map(|s| Complex::new(s, 0.0))
                        .collect::<Vec<_>>();
                    self.forward_fft.process(&mut complex_chunk);
                    complex_chunk
                })
                .collect();
        }

        let num_partitions = path_data.ir_fft_partitions.len();
        path_data.input_fft_history = vec![vec![Complex::new(0.0, 0.0); FFT_SIZE]; num_partitions];
        path_data.history_index = 0;
        path_data.overlap_buffer.iter_mut().for_each(|s| *s = 0.0);
    }

    pub fn process_block(
        &mut self,
        input_left: &[f32],
        input_right: &[f32],
        output_left: &mut [f32],
        output_right: &mut [f32],
    ) {
        let num_samples = input_left.len();
        self.input_buffer_l.extend_from_slice(input_left);
        self.input_buffer_r.extend_from_slice(input_right);

        while self.input_buffer_l.len() >= BLOCK_SIZE {
            let input_chunk_l = self.input_buffer_l.drain(..BLOCK_SIZE).collect::<Vec<_>>();
            let input_chunk_r = self.input_buffer_r.drain(..BLOCK_SIZE).collect::<Vec<_>>();

            let (processed_l, processed_r) =
                self.process_internal_block(&input_chunk_l, &input_chunk_r);

            self.output_buffer_l.extend_from_slice(&processed_l);
            self.output_buffer_r.extend_from_slice(&processed_r);
        }

        if self.output_buffer_l.len() >= num_samples {
            output_left.copy_from_slice(
                &self
                    .output_buffer_l
                    .drain(..num_samples)
                    .collect::<Vec<_>>(),
            );
            output_right.copy_from_slice(
                &self
                    .output_buffer_r
                    .drain(..num_samples)
                    .collect::<Vec<_>>(),
            );
        } else {
            // This case should ideally not be hit if input/output lengths match,
            // but as a fallback, output silence to prevent weird audio artifacts.
            output_left.fill(0.0);
            output_right.fill(0.0);
        }
    }

    fn process_internal_block(&mut self, input_l: &[f32], input_r: &[f32]) -> (Vec<f32>, Vec<f32>) {
        let mut out_l = vec![0.0; BLOCK_SIZE];
        let mut out_r = vec![0.0; BLOCK_SIZE];

        let (lsl, lsr, rsl, rsr) = {
            let (paths_l, paths_r) = self.paths.split_at_mut(2);
            let (path_lsl, path_lsr) = paths_l.split_at_mut(1);
            let (path_rsl, path_rsr) = paths_r.split_at_mut(1);

            let lsl = convolve_path_partitioned(
                input_l,
                &mut path_lsl[0],
                &self.forward_fft,
                &self.inverse_fft,
                &mut self.input_fft_buffer,
                &mut self.conv_accumulator,
            );
            let lsr = convolve_path_partitioned(
                input_l,
                &mut path_lsr[0],
                &self.forward_fft,
                &self.inverse_fft,
                &mut self.input_fft_buffer,
                &mut self.conv_accumulator,
            );
            let rsl = convolve_path_partitioned(
                input_r,
                &mut path_rsl[0],
                &self.forward_fft,
                &self.inverse_fft,
                &mut self.input_fft_buffer,
                &mut self.conv_accumulator,
            );
            let rsr = convolve_path_partitioned(
                input_r,
                &mut path_rsr[0],
                &self.forward_fft,
                &self.inverse_fft,
                &mut self.input_fft_buffer,
                &mut self.conv_accumulator,
            );
            (lsl, lsr, rsl, rsr)
        };

        for i in 0..BLOCK_SIZE {
            out_l[i] = lsl[i] + rsl[i];
            out_r[i] = lsr[i] + rsr[i];
        }
        (out_l, out_r)
    }
}

fn convolve_path_partitioned(
    input_signal: &[f32],
    path_data: &mut ConvolutionPathData,
    forward_fft: &Arc<dyn Fft<f32>>,
    inverse_fft: &Arc<dyn Fft<f32>>,
    input_fft_buffer: &mut [Complex<f32>],
    conv_accumulator: &mut [Complex<f32>],
) -> Vec<f32> {
    // 1. FFT the current input block
    for (i, sample) in input_signal.iter().enumerate() {
        input_fft_buffer[i] = Complex::new(*sample, 0.0);
    }
    for val in input_fft_buffer
        .iter_mut()
        .skip(BLOCK_SIZE)
        .take(FFT_SIZE - BLOCK_SIZE)
    {
        *val = Complex::new(0.0, 0.0);
    }
    forward_fft.process(input_fft_buffer);

    // 2. Store in history and advance index
    path_data.input_fft_history[path_data.history_index] = input_fft_buffer.to_vec();

    // 3. Perform convolution with all partitions
    conv_accumulator
        .iter_mut()
        .for_each(|c| *c = Complex::new(0.0, 0.0));
    let num_partitions = path_data.ir_fft_partitions.len();
    for i in 0..num_partitions {
        let history_idx = (path_data.history_index + num_partitions - i) % num_partitions;
        let input_fft = &path_data.input_fft_history[history_idx];
        let ir_fft = &path_data.ir_fft_partitions[i];

        for j in 0..FFT_SIZE {
            conv_accumulator[j] += input_fft[j] * ir_fft[j];
        }
    }

    // 4. Inverse FFT
    inverse_fft.process(conv_accumulator);

    // 5. Overlap-add
    let mut output = vec![0.0; BLOCK_SIZE];
    let scale = 1.0 / FFT_SIZE as f32;
    for i in 0..BLOCK_SIZE {
        output[i] = conv_accumulator[i].re * scale + path_data.overlap_buffer[i];
        path_data.overlap_buffer[i] = conv_accumulator[i + BLOCK_SIZE].re * scale;
    }

    path_data.history_index = (path_data.history_index + 1) % num_partitions;

    output
}

impl Default for ConvolutionEngine {
    fn default() -> Self {
        Self::new()
    }
}

// --- Unit Tests ---
#[cfg(test)]
mod tests {
    use super::*;
    const TOLERANCE: f32 = 1e-3;

    fn assert_approx_eq_slice(a: &[f32], b: &[f32], tolerance: f32, msg: &str) {
        assert_eq!(a.len(), b.len(), "Slice length mismatch in '{}'", msg);
        for (i, (x, y)) in a.iter().zip(b.iter()).enumerate() {
            assert!(
                (x - y).abs() < tolerance,
                "Mismatch at index {} in '{}': {} vs {}",
                i,
                msg,
                x,
                y
            );
        }
    }

    #[test]
    fn test_identity_ir_passthrough() {
        let mut engine = ConvolutionEngine::new();
        engine.set_ir(ConvolutionPath::Lsl, &[1.0]);
        engine.set_ir(ConvolutionPath::Lsr, &[0.0]);
        engine.set_ir(ConvolutionPath::Rsl, &[0.0]);
        engine.set_ir(ConvolutionPath::Rsr, &[1.0]);

        let input_l: Vec<f32> = (0..BLOCK_SIZE).map(|i| (i as f32 * 0.1).sin()).collect();
        let input_r: Vec<f32> = (0..BLOCK_SIZE).map(|i| (i as f32 * -0.1).sin()).collect();
        let mut output_l = vec![0.0; BLOCK_SIZE];
        let mut output_r = vec![0.0; BLOCK_SIZE];

        engine.process_block(&input_l, &input_r, &mut output_l, &mut output_r);

        // The first block will have a delay due to processing, so we check the second block
        engine.process_block(&input_l, &input_r, &mut output_l, &mut output_r);

        assert_approx_eq_slice(
            &output_l,
            &input_l,
            TOLERANCE,
            "Identity passthrough L channel",
        );
        assert_approx_eq_slice(
            &output_r,
            &input_r,
            TOLERANCE,
            "Identity passthrough R channel",
        );
    }

    #[test]
    fn test_delay_ir() {
        let mut engine = ConvolutionEngine::new();
        let delay_samples = 5;
        let mut ir = vec![0.0; delay_samples + 1];
        ir[delay_samples] = 1.0;

        engine.set_ir(ConvolutionPath::Lsl, &ir);
        engine.set_ir(ConvolutionPath::Lsr, &[0.0]);
        engine.set_ir(ConvolutionPath::Rsl, &[0.0]);
        engine.set_ir(ConvolutionPath::Rsr, &[0.0]);

        let mut input_l = vec![0.0; BLOCK_SIZE * 2];
        for (i, sample) in input_l.iter_mut().enumerate() {
            *sample = i as f32;
        }
        let input_r = vec![0.0; BLOCK_SIZE * 2];
        let mut output_l = vec![0.0; BLOCK_SIZE * 2];
        let mut output_r = vec![0.0; BLOCK_SIZE * 2];

        engine.process_block(&input_l, &input_r, &mut output_l, &mut output_r);

        let mut expected_output = vec![0.0; BLOCK_SIZE * 2];
        let end_index = BLOCK_SIZE * 2;
        expected_output[delay_samples..end_index]
            .copy_from_slice(&input_l[..(end_index - delay_samples)]);

        // We check the output after enough samples have been processed to overcome initial latency
        assert_approx_eq_slice(
            &output_l[delay_samples..],
            &expected_output[delay_samples..],
            TOLERANCE,
            "Delayed signal",
        );
    }

    #[test]
    fn test_long_ir_partitioning() {
        let mut engine = ConvolutionEngine::new();
        // Create an IR longer than one block
        let ir_len = BLOCK_SIZE + BLOCK_SIZE / 2;
        let mut ir = vec![0.0; ir_len];
        ir[0] = 1.0; // Passthrough part
        ir[ir_len - 1] = 0.5; // Echo part

        engine.set_ir(ConvolutionPath::Lsl, &ir);
        assert_eq!(
            engine.paths[0].ir_fft_partitions.len(),
            2,
            "IR should be split into 2 partitions"
        );

        let mut input_l = vec![0.0; BLOCK_SIZE * 3];
        input_l[0] = 1.0; // Impulse
        let input_r = vec![0.0; BLOCK_SIZE * 3];
        let mut output_l = vec![0.0; BLOCK_SIZE * 3];
        let mut output_r = vec![0.0; BLOCK_SIZE * 3];

        engine.process_block(&input_l, &input_r, &mut output_l, &mut output_r);

        // After processing enough blocks, we expect to see the impulse response in the output
        let mut expected_output = vec![0.0; BLOCK_SIZE * 3];
        expected_output[0] = 1.0;
        expected_output[ir_len - 1] = 0.5;

        // Check the relevant part of the output
        assert_approx_eq_slice(
            &output_l[0..ir_len],
            &expected_output[0..ir_len],
            TOLERANCE,
            "Long IR convolution",
        );
    }
}
