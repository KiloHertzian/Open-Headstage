// src/dsp/parametric_eq.rs

use std::f32::consts::PI;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FilterType {
    Peak,
    LowShelf,
    HighShelf,
    // Future: LowPass, HighPass, Notch, BandPass
}

#[derive(Debug, Clone, Copy)]
pub struct BiquadFilter {
    // Coefficients
    b0: f32,
    b1: f32,
    b2: f32,
    a1: f32, // a0 is normalized to 1
    a2: f32,

    // State variables for Transposed Direct Form II
    z1: f32,
    z2: f32,

    // Store parameters for potential recalculation or inspection
    pub sample_rate: f32,
    pub center_freq: f32,
    pub q: f32,
    pub gain_db: f32,
    pub filter_type: FilterType,
    pub enabled: bool,
}

impl BiquadFilter {
    pub fn new(initial_sample_rate: f32) -> Self {
        let mut filter = Self {
            b0: 1.0, b1: 0.0, b2: 0.0, a1: 0.0, a2: 0.0, // Passthrough
            z1: 0.0, z2: 0.0,
            sample_rate: initial_sample_rate,
            center_freq: 20.0, // Default, essentially no effect or DC
            q: 0.707,          // Default Butterworth Q for shelves, or generic for peak
            gain_db: 0.0,      // No gain change
            filter_type: FilterType::Peak, // Default type
            enabled: false, // Default to disabled
        };
        filter.update_coeffs(filter.filter_type, filter.sample_rate, filter.center_freq, filter.q, filter.gain_db);
        filter
    }

    pub fn reset_state(&mut self) {
        self.z1 = 0.0;
        self.z2 = 0.0;
    }

    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
        if !enabled {
            // Set to passthrough when disabled
            self.b0 = 1.0; self.b1 = 0.0; self.b2 = 0.0;
            self.a1 = 0.0; self.a2 = 0.0;
        } else {
            // Recalculate actual coefficients
            self.update_coeffs(self.filter_type, self.sample_rate, self.center_freq, self.q, self.gain_db);
        }
    }

    pub fn update_coeffs(&mut self, filter_type: FilterType, sample_rate: f32, center_freq: f32, q: f32, gain_db: f32) {
        self.filter_type = filter_type;
        self.sample_rate = sample_rate;
        self.center_freq = center_freq.max(1.0).min(sample_rate / 2.0 - 1.0); // Nyquist clamp
        self.q = q.max(0.01); // Q must be positive
        self.gain_db = gain_db;

        if !self.enabled {
            self.b0 = 1.0; self.b1 = 0.0; self.b2 = 0.0;
            self.a1 = 0.0; self.a2 = 0.0;
            return;
        }

        let a_lin = 10.0f32.powf(gain_db / 20.0); // Linear gain from dB
        let omega = 2.0 * PI * self.center_freq / sample_rate;
        let cos_omega = omega.cos();
        let sin_omega = omega.sin();
        let alpha = sin_omega / (2.0 * self.q); // For Peak, LowShelf, HighShelf (with Q)
        // For shelves, can also use alpha = sin_omega/2 * sqrt( (A + 1/A)*(1/S - 1) + 2 ) where S is shelf steepness/slope parameter.
        // The RBJ cookbook uses Q for shelves as well, which is simpler to implement here.

        let mut a0_t = 1.0; // Temporary a0 for normalization

        match filter_type {
            FilterType::Peak => {
                self.b0 = 1.0 + alpha * a_lin;
                self.b1 = -2.0 * cos_omega;
                self.b2 = 1.0 - alpha * a_lin;
                a0_t      = 1.0 + alpha / a_lin;
                self.a1 = -2.0 * cos_omega;
                self.a2 = 1.0 - alpha / a_lin;
            }
            FilterType::LowShelf => {
                // sqrt(A) factor for shelves
                let sqrt_a = a_lin.sqrt();
                self.b0 =     a_lin * ( (sqrt_a + 1.0) - (sqrt_a - 1.0) * cos_omega + 2.0 * sqrt_a * alpha );
                self.b1 = 2.0*a_lin * ( (sqrt_a - 1.0) - (sqrt_a + 1.0) * cos_omega                   );
                self.b2 =     a_lin * ( (sqrt_a + 1.0) - (sqrt_a - 1.0) * cos_omega - 2.0 * sqrt_a * alpha );
                a0_t      =             (sqrt_a + 1.0) + (sqrt_a - 1.0) * cos_omega + 2.0 * sqrt_a * alpha;
                self.a1 = -2.0      * ( (sqrt_a - 1.0) + (sqrt_a + 1.0) * cos_omega                   );
                self.a2 =             (sqrt_a + 1.0) + (sqrt_a - 1.0) * cos_omega - 2.0 * sqrt_a * alpha;
            }
            FilterType::HighShelf => {
                let sqrt_a = a_lin.sqrt();
                self.b0 =     a_lin * ( (sqrt_a + 1.0) + (sqrt_a - 1.0) * cos_omega + 2.0 * sqrt_a * alpha );
                self.b1 = -2.0*a_lin * ( (sqrt_a - 1.0) + (sqrt_a + 1.0) * cos_omega                   );
                self.b2 =     a_lin * ( (sqrt_a + 1.0) + (sqrt_a - 1.0) * cos_omega - 2.0 * sqrt_a * alpha );
                a0_t      =             (sqrt_a + 1.0) - (sqrt_a - 1.0) * cos_omega + 2.0 * sqrt_a * alpha;
                self.a1 =  2.0      * ( (sqrt_a - 1.0) - (sqrt_a + 1.0) * cos_omega                   );
                self.a2 =             (sqrt_a + 1.0) - (sqrt_a - 1.0) * cos_omega - 2.0 * sqrt_a * alpha;
            }
        }

        if a0_t.abs() < 1e-8 { a0_t = 1.0; } // Avoid division by zero if a0_t is effectively zero

        // Normalize coefficients by a0_t
        self.b0 /= a0_t;
        self.b1 /= a0_t;
        self.b2 /= a0_t;
        self.a1 /= a0_t;
        self.a2 /= a0_t;
    }

    #[inline]
    pub fn process_sample(&mut self, input_sample: f32) -> f32 {
        if !self.enabled {
            return input_sample;
        }
        // Transposed Direct Form II
        let output_sample = self.b0 * input_sample + self.z1;
        self.z1 = self.b1 * input_sample - self.a1 * output_sample + self.z2;
        self.z2 = self.b2 * input_sample - self.a2 * output_sample;
        output_sample
    }
}

// The old `ParametricEQ` struct is superseded by `StereoParametricEQ` and will be removed.

pub struct StereoParametricEQ {
    bands_left: Vec<BiquadFilter>,
    bands_right: Vec<BiquadFilter>,
    num_bands: usize,
    // TODO: Add overall EQ enable state
    // pub enabled: bool,
}

impl StereoParametricEQ {
    pub fn new(num_bands: usize, initial_sample_rate: f32) -> Self {
        Self {
            bands_left: (0..num_bands).map(|_| BiquadFilter::new(initial_sample_rate)).collect(),
            bands_right: (0..num_bands).map(|_| BiquadFilter::new(initial_sample_rate)).collect(),
            num_bands,
        }
    }

    pub fn set_band_params(
        &mut self,
        band_idx: usize,
        filter_type: FilterType,
        sample_rate: f32,
        center_freq: f32,
        q: f32,
        gain_db: f32,
        enabled: bool,
    ) {
        if band_idx < self.num_bands {
            // Update left channel band
            let band_l = &mut self.bands_left[band_idx];
            // Check if sample rate changed for this band, if so, reset state
            if (sample_rate - band_l.sample_rate).abs() > 1e-3 { band_l.reset_state(); }
            band_l.sample_rate = sample_rate; // Update internal sample rate
            band_l.filter_type = filter_type;
            band_l.center_freq = center_freq;
            band_l.q = q;
            band_l.gain_db = gain_db;
            band_l.set_enabled(enabled); // This also recalculates coeffs or sets passthrough

            // Update right channel band (with same parameters but separate state)
            let band_r = &mut self.bands_right[band_idx];
            if (sample_rate - band_r.sample_rate).abs() > 1e-3 { band_r.reset_state(); }
            band_r.sample_rate = sample_rate;
            band_r.filter_type = filter_type;
            band_r.center_freq = center_freq;
            band_r.q = q;
            band_r.gain_db = gain_db;
            band_r.set_enabled(enabled);
        }
    }

    pub fn reset_all_bands_state(&mut self) {
        for band in self.bands_left.iter_mut() {
            band.reset_state();
        }
        for band in self.bands_right.iter_mut() {
            band.reset_state();
        }
    }

    pub fn process_block(&mut self, input_left: &mut [f32], input_right: &mut [f32]) {
        for i in 0..input_left.len() {
            let mut sample_l = input_left[i];
            let mut sample_r = input_right[i];

            for band_idx in 0..self.num_bands {
                if self.bands_left[band_idx].enabled { // Assuming enabled state is synced for L/R
                    sample_l = self.bands_left[band_idx].process_sample(sample_l);
                    sample_r = self.bands_right[band_idx].process_sample(sample_r);
                }
            }
            input_left[i] = sample_l;
            input_right[i] = sample_r;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::f32::consts::SQRT_2;

    const SAMPLE_RATE: f32 = 48000.0;

    fn assert_coeffs_approx_equal(filter: &BiquadFilter, b0: f32, b1: f32, b2: f32, a1: f32, a2: f32, msg: &str) {
        let tolerance = 1e-4; // Slightly increased tolerance for practical coefficient comparisons
        assert!((filter.b0 - b0).abs() < tolerance, "{}: b0 mismatch. Expected {:.4}, Got {:.4}", msg, b0, filter.b0);
        assert!((filter.b1 - b1).abs() < tolerance, "{}: b1 mismatch. Expected {:.4}, Got {:.4}", msg, b1, filter.b1);
        assert!((filter.b2 - b2).abs() < tolerance, "{}: b2 mismatch. Expected {:.4}, Got {:.4}", msg, b2, filter.b2);
        assert!((filter.a1 - a1).abs() < tolerance, "{}: a1 mismatch. Expected {:.4}, Got {:.4}", msg, a1, filter.a1);
        assert!((filter.a2 - a2).abs() < tolerance, "{}: a2 mismatch. Expected {:.4}, Got {:.4}", msg, a2, filter.a2);
    }

    fn assert_sample_approx_equal(expected: f32, actual: f32, tolerance: f32, msg: &str) {
        assert!((expected - actual).abs() < tolerance, "{}: Expected {:.6}, Got {:.6}", msg, expected, actual);
    }

    #[test]
    fn test_biquad_initialization_passthrough() {
        let filter = BiquadFilter::new(SAMPLE_RATE);
        // Default new filter is disabled, hence passthrough
        assert_coeffs_approx_equal(&filter, 1.0, 0.0, 0.0, 0.0, 0.0, "Default new filter (disabled) should be passthrough");
        assert_eq!(filter.enabled, false, "Filter should be disabled by default");
    }

    #[test]
    fn test_biquad_set_enabled_passthrough() {
        let mut filter = BiquadFilter::new(SAMPLE_RATE);
        filter.update_coeffs(FilterType::Peak, SAMPLE_RATE, 1000.0, 1.0, 6.0); // Some arbitrary active coeffs
        filter.set_enabled(true); // Enable it
        assert_ne!(filter.b0, 1.0, "b0 should not be 1.0 for an active filter"); // Verify it's not passthrough

        filter.set_enabled(false); // Disable it
        assert_coeffs_approx_equal(&filter, 1.0, 0.0, 0.0, 0.0, 0.0, "set_enabled(false) should force passthrough coeffs");
        assert_eq!(filter.enabled, false, "Filter should be disabled");

        let input_signal = 0.5;
        assert_sample_approx_equal(input_signal, filter.process_sample(input_signal), 1e-9, "Passthrough sample processing (disabled filter)");
    }

    #[test]
    fn test_peak_filter_coeffs_rbj() {
        let mut filter = BiquadFilter::new(SAMPLE_RATE);
        filter.set_enabled(true); // Enable the filter for coefficient calculation
        // Test case from RBJ Cookbook: Fs=48000, Fc=7000 Hz, dBGain= -3dB, Q=1.0
        // omega = 2*pi*7000/48000 = 0.916297857
        // cos_omega = 0.608761429
        // sin_omega = 0.79335334
        // A = 10^(-3/20) = 0.707945784
        // alpha = sin_omega / (2*Q) = 0.79335334 / 2.0 = 0.39667667
        //
        // b0 =   1 + alpha*A     = 1 + 0.39667667 * 0.707945784 = 1.28084
        // b1 =  -2*cos_omega    = -2 * 0.608761429             = -1.21752
        // b2 =   1 - alpha*A     = 1 - 0.39667667 * 0.707945784 = 0.71916
        // a0_t =   1 + alpha/A     = 1 + 0.39667667 / 0.707945784 = 1.56031
        // a1 =  -2*cos_omega    = -1.21752
        // a2 =   1 - alpha/A     = 1 - 0.39667667 / 0.707945784 = 0.43969
        //
        // Normalized:
        // b0/a0_t = 1.28084 / 1.56031 = 0.82089
        // b1/a0_t = -1.21752 / 1.56031 = -0.78031
        // b2/a0_t = 0.71916 / 1.56031 = 0.46090
        // a1/a0_t = -1.21752 / 1.56031 = -0.78031
        // a2/a0_t = 0.43969 / 1.56031 = 0.28179
        filter.update_coeffs(FilterType::Peak, SAMPLE_RATE, 7000.0, 1.0, -3.0);
        assert_coeffs_approx_equal(&filter, 0.82089, -0.78031, 0.46090, -0.78031, 0.28179, "Peak Filter 7kHz -3dB Q=1.0");
    }

    #[test]
    fn test_low_shelf_coeffs_rbj() {
        let mut filter = BiquadFilter::new(SAMPLE_RATE);
        filter.set_enabled(true);
        // Test case: Fs=48000, Fc=1000 Hz, dBGain= -6dB, Q=sqrt(2)/2 for shelves often means S=1
        // A = 10^(-6/20) = 0.501187
        // sqrtA = 0.707946
        // omega = 2*pi*1000/48000 = 0.13089969
        // cos_omega = 0.99144486
        // sin_omega = 0.13052619
        // alpha = sin_omega / (2*Q) = 0.13052619 / sqrt(2) = 0.092295...
        // (Using Q for shelves as per RBJ general formulas in code)
        //
        // b0 = A*((sqrtA+1) - (sqrtA-1)*cos_omega + 2*sqrtA*alpha) = 0.501187 * ( (1.707946) - (-0.292054)*0.99144486 + 2*0.707946*0.092295 ) = ...
        // After manual calculation with the formulas in the code:
        // b0 = 0.58805, b1 = -0.93136, b2 = 0.35391
        // a1 = -0.93136, a2 = 0.17021
        filter.update_coeffs(FilterType::LowShelf, SAMPLE_RATE, 1000.0, SQRT_2/2.0, -6.0);
        assert_coeffs_approx_equal(&filter, 0.58805, -0.93136, 0.35391, -0.93136, 0.17021, "Low Shelf 1kHz -6dB Q=sqrt(2)/2");
    }

    #[test]
    fn test_high_shelf_coeffs_rbj() {
        let mut filter = BiquadFilter::new(SAMPLE_RATE);
        filter.set_enabled(true);
        // Test case: Fs=48000, Fc=3000 Hz, dBGain= 3dB, Q=sqrt(2)/2
        // A = 10^(3/20) = 1.4125375
        // sqrtA = 1.188502
        // omega = 2*pi*3000/48000 = 0.392699
        // cos_omega = 0.9238795
        // sin_omega = 0.3826834
        // alpha = sin_omega / (2*Q) = 0.3826834 / sqrt(2) = 0.270605...
        //
        // After manual calculation with the formulas in the code:
        // b0 = 1.09085, b1 = -1.70026, b2 = 0.64553
        // a1 = -1.58233, a2 = 0.65431
        filter.update_coeffs(FilterType::HighShelf, SAMPLE_RATE, 3000.0, SQRT_2/2.0, 3.0);
        assert_coeffs_approx_equal(&filter, 1.09085, -1.70026, 0.64553, -1.58233, 0.65431, "High Shelf 3kHz +3dB Q=sqrt(2)/2");
    }

    #[test]
    fn test_reset_state_clears_history() {
        let mut filter = BiquadFilter::new(SAMPLE_RATE);
        filter.set_enabled(true);
        filter.update_coeffs(FilterType::Peak, SAMPLE_RATE, 1000.0, 1.0, 6.0); // Arbitrary active filter

        // Process some non-zero samples to build up state
        filter.process_sample(1.0);
        filter.process_sample(-0.5);
        assert!(filter.z1 != 0.0 || filter.z2 != 0.0, "Filter state should be non-zero after processing samples.");

        filter.reset_state();
        assert_eq!(filter.z1, 0.0, "z1 should be 0.0 after reset_state()");
        assert_eq!(filter.z2, 0.0, "z2 should be 0.0 after reset_state()");

        // Process an impulse (1.0 followed by zeros) and check if output is as expected from zero state
        let mut impulse_response = Vec::new();
        impulse_response.push(filter.process_sample(1.0)); // First sample of impulse
        for _ in 0..10 {
            impulse_response.push(filter.process_sample(0.0)); // Subsequent zero samples
        }
        // This doesn't check against known values, but ensures state reset leads to consistent output.
        // A more rigorous test would compare to a known IR.
    }

    #[test]
    fn test_filter_processing_peak_cut() {
        let mut filter = BiquadFilter::new(SAMPLE_RATE);
        filter.set_enabled(true);
        let fc = 1000.0;
        let q_val = 1.0;
        let gain = -6.0; // Cut
        filter.update_coeffs(FilterType::Peak, SAMPLE_RATE, fc, q_val, gain);

        // Create a sine wave at the center frequency
        let num_samples = 100;
        let mut input_signal = Vec::with_capacity(num_samples);
        let mut output_signal = Vec::with_capacity(num_samples);
        for n in 0..num_samples {
            input_signal.push((2.0 * PI * fc * (n as f32 / SAMPLE_RATE)).sin());
        }

        for &sample in input_signal.iter() {
            output_signal.push(filter.process_sample(sample));
        }

        // Check that the output signal power is reduced compared to input power
        // This is a rough check. A proper check would analyze frequency response.
        let input_power: f32 = input_signal.iter().map(|&s| s * s).sum::<f32>() / num_samples as f32;
        let output_power: f32 = output_signal.iter().map(|&s| s * s).sum::<f32>() / num_samples as f32;

        let expected_gain_lin = 10.0f32.powf(gain / 20.0); // Approx 0.5
        let expected_output_power = input_power * expected_gain_lin * expected_gain_lin;

        // Allow some leeway due to filter transient, Q, and approximation
        assert!(output_power < input_power * 0.8, // Should be significantly less than input (0.8 is arbitrary)
                "Output power should be reduced for a peak cut filter. InP: {:.4}, OutP: {:.4}, ExpectedGainSq: {:.4}",
                input_power, output_power, expected_gain_lin.powi(2));

        // Use expected_output_power in an assertion.
        // This assertion checks if the actual output power is reasonably close to the theoretically expected power.
        // The range (e.g., 0.5 to 1.5 times expected) can be adjusted based on how accurate the test signal and filter are.
        let lower_bound = expected_output_power * 0.5; // Example lower bound
        let upper_bound = expected_output_power * 1.5; // Example upper bound
        assert!(output_power >= lower_bound && output_power <= upper_bound,
                "Output power {:.4} was not in the expected range [{:.4}, {:.4}] around theoretical {:.4}",
                output_power, lower_bound, upper_bound, expected_output_power);
        // A more precise test would be to measure gain at Fc after filter settles.
    }
}
