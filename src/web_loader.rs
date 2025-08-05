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

//! Handles fetching and parsing AutoEQ data from the web or local cache.

#![allow(dead_code)] // This module is not yet fully integrated
#![allow(unused_imports)] // This module is not yet fully integrated

use crate::autoeq_parser::{BandSetting, ParsedEqBand};
use crate::dsp::parametric_eq::FilterType;
use std::error::Error;
use std::fs;
use std::path::{Path, PathBuf};

const PRESERVE_DIR: &str = "PRESERVE/AutoEq/results";

const SOURCE_PRIORITY_LIST: &[&str] = &[
    // S-Tier
    "oratory1990",
    "Rtings",
    // A-Tier
    "crinacle",
    "Innerfidelity",
    // B-Tier
    "Headphone.com Legacy",
    "Super Review",
    // C-Tier (and the rest)
    "Auriculares Argentina",
    "Bakkwatan",
    "DHRME",
    "Fahryst",
    "Filk",
    "Harpo",
    "Hi End Portable",
    "HypetheSonics",
    "Jaytiss",
    "Kazi",
    "Kuulokenurkka",
    "Regan Cipher",
    "RikudouGoku",
    "Ted's Squig Hoard",
    "ToneDeafMonk",
    "freeryder05",
    "kr0mka",
];

const HEADPHONE_TYPES: &[&str] = &["over-ear", "in-ear", "earbud"];

/// Finds the best available EQ data for a given headphone model based on the source priority list.
///
/// This function iterates through the prioritized sources and headphone types, searching for a
/// matching data directory within the local `PRESERVE` cache.
pub fn find_best_headphone_data(
    headphone_name: &str,
) -> Result<Vec<BandSetting>, Box<dyn Error>> {
    for source in SOURCE_PRIORITY_LIST {
        for headphone_type in HEADPHONE_TYPES {
            let mut path = PathBuf::from(PRESERVE_DIR);
            path.push(source);
            path.push(headphone_type);
            path.push(headphone_name);

            if path.is_dir() {
                let eq_file_path = path.join(format!("{} ParametricEQ.txt", headphone_name));
                if eq_file_path.is_file() {
                    println!("Found best data at: {:?}", eq_file_path); // For debugging
                    let content = fs::read_to_string(&eq_file_path)?;
                    return parse_autoeq_text(&content);
                }
            }
        }
    }

    Err(format!("No data found for headphone: {}", headphone_name).into())
}

fn map_filter_type(autoeq_type: &str) -> Result<FilterType, String> {
    match autoeq_type {
        "PK" => Ok(FilterType::Peak),
        "LS" => Ok(FilterType::LowShelf),
        "HS" => Ok(FilterType::HighShelf),
        _ => Err(format!("Unsupported filter type: {}", autoeq_type)),
    }
}

/// Parses a string containing AutoEQ CSV data into a vector of BandSettings.
pub fn parse_autoeq_text(text_content: &str) -> Result<Vec<BandSetting>, Box<dyn Error>> {
    let mut rdr = csv::Reader::from_reader(text_content.as_bytes());
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