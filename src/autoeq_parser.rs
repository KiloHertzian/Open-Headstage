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

use serde::Deserialize;
use std::error::Error;
use std::fs::File;
use std::path::Path;

use crate::dsp::parametric_eq::FilterType;

#[derive(Debug, Deserialize, Clone)]
pub struct ParsedEqBand {
    #[serde(rename = "Filter-Type")]
    pub filter_type_str: String,
    #[serde(rename = "Fc")]
    pub frequency: f32,
    #[serde(rename = "Q")]
    pub q: f32,
    #[serde(rename = "Gain")]
    pub gain: f32,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, Default)]
pub struct BandSetting {
    pub enabled: bool,
    pub filter_type: FilterType,
    pub frequency: f32,
    pub q: f32,
    pub gain: f32,
}

fn map_filter_type(autoeq_type: &str) -> Result<FilterType, String> {
    match autoeq_type {
        "PK" => Ok(FilterType::Peak),
        "LS" => Ok(FilterType::LowShelf),
        "HS" => Ok(FilterType::HighShelf),
        _ => Err(format!("Unsupported filter type: {}", autoeq_type)),
    }
}

pub fn parse_autoeq_csv(path: &Path) -> Result<Vec<BandSetting>, Box<dyn Error>> {
    let file = File::open(path)?;
    let mut rdr = csv::Reader::from_reader(file);
    let mut bands = Vec::new();

    for result in rdr.deserialize() {
        let record: ParsedEqBand = result?;
        let band_setting = BandSetting {
            enabled: true,
            filter_type: map_filter_type(&record.filter_type_str)?,
            frequency: record.frequency,
            q: record.q,
            gain: record.gain,
        };
        bands.push(band_setting);
    }

    Ok(bands)
}
