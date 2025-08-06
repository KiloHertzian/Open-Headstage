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

use nih_plug::prelude::nih_log;
use std::error::Error;
use std::fs;
use std::path::Path;
use std::str::FromStr;

use crate::dsp::parametric_eq::FilterType;

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, Default)]
pub struct BandSetting {
    pub enabled: bool,
    pub filter_type: FilterType,
    pub frequency: f32,
    pub q: f32,
    pub gain: f32,
}

#[derive(Debug, Clone, Default)]
pub struct AutoEqProfile {
    pub preamp: f32,
    pub bands: Vec<BandSetting>,
}

fn parse_filter_line(line: &str) -> Result<BandSetting, Box<dyn Error>> {
    let parts: Vec<&str> = line.split_whitespace().collect();
    // Example: Filter 1: ON PK Fc 8983 Hz Gain 4.4 dB Q 2.63
    if parts.len() < 11 || parts[0] != "Filter" || parts[2] != "ON" {
        return Err("Invalid filter line format".into());
    }

    let filter_type_str = parts[3];
    let fc_str = parts[5];
    let gain_str = parts[8];
    let q_str = parts[11];

    let filter_type = match filter_type_str {
        "PK" => FilterType::Peak,
        "LSC" => FilterType::LowShelf,
        "HSC" => FilterType::HighShelf,
        _ => return Err(format!("Unknown filter type: {}", filter_type_str).into()),
    };

    Ok(BandSetting {
        enabled: true,
        filter_type,
        frequency: f32::from_str(fc_str)?,
        q: f32::from_str(q_str)?,
        gain: f32::from_str(gain_str)?,
    })
}

pub fn parse_autoeq_file(path: &Path) -> Result<AutoEqProfile, Box<dyn Error>> {
    let content = fs::read_to_string(path)?;
    let mut profile = AutoEqProfile::default();

    for line in content.lines() {
        if line.starts_with("Preamp:") {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 2 {
                profile.preamp = f32::from_str(parts[1])?;
            }
        } else if line.starts_with("Filter") {
            match parse_filter_line(line) {
                Ok(band) => profile.bands.push(band),
                Err(e) => nih_log!("Skipping invalid filter line: {}", e),
            }
        }
    }

    Ok(profile)
}
