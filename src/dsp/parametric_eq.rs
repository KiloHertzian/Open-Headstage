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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Enum, EnumIter, Serialize, Deserialize, Default, Display)]
#[allow(dead_code)]
pub enum FilterType {
    #[default]
    Peak,
    LowShelf,
    HighShelf,
    LowPass,
    HighPass,
    BandPass,
    Notch,
    AllPass,
}

#[derive(Debug, Clone, Copy)]
pub struct BandConfig {
    pub filter_type: FilterType,
    pub center_freq: f32,
    pub q: f32,
    pub gain_db: f32,
    pub enabled: bool,
}

pub struct BiquadFilter {
    filter: DirectForm2Transposed<f32>,
    coeffs: Coefficients<f32>,
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

    pub fn reset_state(&mut self) {
        self.filter.reset_state();
    }

    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }

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

    #[inline]
    pub fn process_sample(&mut self, input_sample: f32) -> f32 {
        if !self.enabled {
            return input_sample;
        }
        self.filter.run(input_sample)
    }
}

pub struct StereoParametricEQ {
    bands_left: Vec<BiquadFilter>,
    bands_right: Vec<BiquadFilter>,
    num_bands: usize,
}

impl StereoParametricEQ {
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

    pub fn reset_all_bands_state(&mut self) {
        for band in self.bands_left.iter_mut() {
            band.reset_state();
        }
        for band in self.bands_right.iter_mut() {
            band.reset_state();
        }
    }

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
