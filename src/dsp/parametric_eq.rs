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

// src/dsp/parametric_eq.rs

use biquad::{Biquad, Coefficients, DirectForm2Transposed, ToHertz, Type};
use nih_plug::prelude::Enum;
use num_complex::Complex;
use serde::{Deserialize, Serialize};
use strum::{Display, EnumIter};

/// Defines the types of filters available for each EQ band.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Enum, EnumIter, Serialize, Deserialize, Default, Display,
)]
#[allow(dead_code)]
pub enum FilterType {
    /// A peaking filter that boosts or cuts a frequency range.
    #[default]
    Peak,
    /// A low-shelf filter that boosts or cuts frequencies below the cutoff.
    LowShelf,
    /// A high-shelf filter that boosts or cuts frequencies above the cutoff.
    HighShelf,
    /// A low-pass filter that passes frequencies below the cutoff.
    LowPass,
    /// A high-pass filter that passes frequencies above the cutoff.
    HighPass,
    /// A band-pass filter that passes frequencies within a certain range.
    BandPass,
    /// A notch filter that cuts a narrow frequency band.
    Notch,
    /// An all-pass filter that changes the phase relationship of frequencies.
    AllPass,
}

/// Configuration for a single band of the parametric EQ.
#[derive(Debug, Clone, Copy)]
pub struct BandConfig {
    /// The type of filter for the band.
    pub filter_type: FilterType,
    /// The center or cutoff frequency of the filter in Hz.
    pub center_freq: f32,
    /// The Q factor of the filter, which determines its bandwidth.
    pub q: f32,
    /// The gain of the filter in decibels (dB).
    pub gain_db: f32,
    /// Whether the band is currently enabled.
    pub enabled: bool,
}

/// A single biquad filter, representing one band of the parametric EQ.
pub struct BiquadFilter {
    /// The biquad filter implementation.
    filter: DirectForm2Transposed<f32>,
    /// The coefficients of the biquad filter.
    coeffs: Coefficients<f32>,
    /// Whether the filter is currently enabled.
    pub enabled: bool,
}

impl Clone for BiquadFilter {
    fn clone(&self) -> Self {
        Self {
            filter: self.filter,
            coeffs: self.coeffs,
            enabled: self.enabled,
        }
    }
}

impl BiquadFilter {
    /// Creates a new `BiquadFilter`.
    ///
    /// # Arguments
    ///
    /// * `initial_sample_rate` - The initial sample rate for the filter.
    ///
    /// # Returns
    ///
    /// A new `BiquadFilter` instance, initialized as a disabled peaking EQ.
    pub fn new(initial_sample_rate: f32) -> Self {
        let coeffs = Coefficients::<f32>::from_params(
            Type::PeakingEQ(0.0),
            initial_sample_rate.hz(),
            20.0.hz(),
            0.707,
        )
        .unwrap();
        Self {
            filter: DirectForm2Transposed::<f32>::new(coeffs),
            coeffs,
            enabled: false,
        }
    }

    /// Resets the internal state of the filter.
    pub fn reset_state(&mut self) {
        self.filter.reset_state();
    }

    /// Enables or disables the filter.
    ///
    /// # Arguments
    ///
    /// * `enabled` - A boolean indicating whether the filter should be enabled.
    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }

    /// Updates the filter's coefficients based on new parameters.
    ///
    /// # Arguments
    ///
    /// * `filter_type` - The new type of the filter.
    /// * `sample_rate` - The current sample rate.
    /// * `center_freq` - The new center frequency.
    /// * `q` - The new Q factor.
    /// * `gain_db` - The new gain in dB.
    pub fn update_coeffs(
        &mut self,
        filter_type: FilterType,
        sample_rate: f32,
        center_freq: f32,
        q: f32,
        gain_db: f32,
    ) {
        let filter_type_biquad = match filter_type {
            FilterType::Peak => Type::PeakingEQ(gain_db),
            FilterType::LowShelf => Type::LowShelf(gain_db),
            FilterType::HighShelf => Type::HighShelf(gain_db),
            FilterType::LowPass => Type::LowPass,
            FilterType::HighPass => Type::HighPass,
            FilterType::BandPass => Type::BandPass,
            FilterType::Notch => Type::Notch,
            FilterType::AllPass => Type::AllPass,
        };

        let coeffs = Coefficients::<f32>::from_params(
            filter_type_biquad,
            sample_rate.hz(),
            center_freq.hz(),
            q,
        )
        .unwrap();
        self.filter.update_coefficients(coeffs);
        self.coeffs = coeffs;
    }

    /// Processes a single audio sample.
    ///
    /// If the filter is disabled, it returns the input sample unchanged.
    ///
    /// # Arguments
    ///
    /// * `input_sample` - The input audio sample.
    ///
    /// # Returns
    ///
    /// The processed audio sample.
    #[inline]
    pub fn process_sample(&mut self, input_sample: f32) -> f32 {
        if !self.enabled {
            return input_sample;
        }
        self.filter.run(input_sample)
    }
}

/// A stereo parametric equalizer with a configurable number of bands.
pub struct StereoParametricEQ {
    /// The biquad filters for the left channel.
    bands_left: Vec<BiquadFilter>,
    /// The biquad filters for the right channel.
    bands_right: Vec<BiquadFilter>,
    /// The number of bands in the EQ.
    num_bands: usize,
}

impl StereoParametricEQ {
    /// Creates a new `StereoParametricEQ`.
    ///
    /// # Arguments
    ///
    /// * `num_bands` - The number of EQ bands to create.
    /// * `initial_sample_rate` - The initial sample rate for the EQ.
    ///
    /// # Returns
    ///
    /// A new `StereoParametricEQ` instance.
    pub fn new(num_bands: usize, initial_sample_rate: f32) -> Self {
        Self {
            bands_left: (0..num_bands)
                .map(|_| BiquadFilter::new(initial_sample_rate))
                .collect(),
            bands_right: (0..num_bands)
                .map(|_| BiquadFilter::new(initial_sample_rate))
                .collect(),
            num_bands,
        }
    }

    /// Updates the coefficients for a specific band in both channels.
    ///
    /// # Arguments
    ///
    /// * `band_idx` - The index of the band to update.
    /// * `sample_rate` - The current sample rate.
    /// * `config` - The new configuration for the band.
    pub fn update_band_coeffs(&mut self, band_idx: usize, sample_rate: f32, config: &BandConfig) {
        if band_idx < self.num_bands {
            self.bands_left[band_idx].update_coeffs(
                config.filter_type,
                sample_rate,
                config.center_freq,
                config.q,
                config.gain_db,
            );
            self.bands_left[band_idx].set_enabled(config.enabled);

            self.bands_right[band_idx].update_coeffs(
                config.filter_type,
                sample_rate,
                config.center_freq,
                config.q,
                config.gain_db,
            );
            self.bands_right[band_idx].set_enabled(config.enabled);
        }
    }

    /// Processes a block of stereo audio data.
    ///
    /// The processing is done in place on the provided buffers.
    ///
    /// # Arguments
    ///
    /// * `input_left` - The mutable buffer for the left channel.
    /// * `input_right` - The mutable buffer for the right channel.
    pub fn process_block(&mut self, input_left: &mut [f32], input_right: &mut [f32]) {
        for i in 0..input_left.len() {
            let mut sample_l = input_left[i];
            let mut sample_r = input_right[i];

            for j in 0..self.num_bands {
                sample_l = self.bands_left[j].process_sample(sample_l);
                sample_r = self.bands_right[j].process_sample(sample_r);
            }

            input_left[i] = sample_l;
            input_right[i] = sample_r;
        }
    }

    /// Resets the internal state of all filter bands for both channels.
    pub fn reset_all_bands_state(&mut self) {
        for band in self.bands_left.iter_mut() {
            band.reset_state();
        }
        for band in self.bands_right.iter_mut() {
            band.reset_state();
        }
    }

    /// Calculates the combined frequency response of all enabled EQ bands.
    ///
    /// # Arguments
    ///
    /// * `sample_rate` - The sample rate to use for the calculation.
    /// * `frequencies` - A slice of frequencies at which to calculate the response.
    ///
    /// # Returns
    ///
    /// A `Vec<f32>` containing the magnitude response at each frequency.
    #[allow(dead_code)]
    pub fn calculate_frequency_response(&self, sample_rate: f32, frequencies: &[f32]) -> Vec<f32> {
        let mut response = vec![1.0; frequencies.len()];
        for (i, &freq) in frequencies.iter().enumerate() {
            let mut band_response = Complex::new(1.0, 0.0);
            for band in &self.bands_left {
                if band.enabled {
                    let coeffs = band.coeffs;
                    let omega = 2.0 * std::f32::consts::PI * freq / sample_rate;
                    let z = Complex::from_polar(1.0, -omega);
                    let numerator = coeffs.b0 + coeffs.b1 * z.powi(-1) + coeffs.b2 * z.powi(-2);
                    let denominator = 1.0 + coeffs.a1 * z.powi(-1) + coeffs.a2 * z.powi(-2);
                    let r = numerator / denominator;
                    band_response *= r;
                }
            }
            response[i] = band_response.norm();
        }
        response
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE_RATE: f32 = 48000.0;

    #[test]
    fn test_biquad_filter_passthrough_when_disabled() {
        let mut filter = BiquadFilter::new(SAMPLE_RATE);
        filter.set_enabled(false);
        let input = 0.5;
        let output = filter.process_sample(input);
        assert_eq!(input, output, "Filter should be passthrough when disabled");
    }

    #[test]
    fn test_biquad_filter_processes_when_enabled() {
        let mut filter = BiquadFilter::new(SAMPLE_RATE);
        filter.update_coeffs(FilterType::LowPass, SAMPLE_RATE, 1000.0, 0.707, 0.0);
        filter.set_enabled(true);
        let input = 0.5;
        let output = filter.process_sample(input);
        assert_ne!(
            input, output,
            "Filter should process the sample when enabled"
        );
    }
}
