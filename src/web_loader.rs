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

//! # AutoEQ Data Loader
//!
//! This module handles fetching and parsing AutoEQ data from the web or a local cache.
//!
//! **Note:** This module is currently a placeholder and not fully integrated into the
//! plugin's logic. The functionality is designed for a future feature where the plugin
//! can automatically download and manage headphone EQ profiles.

#![allow(dead_code)] // This module is not yet fully integrated
#![allow(unused_imports)] // This module is not yet fully integrated

use crate::autoeq_parser::{BandSetting, ParsedEqBand};
use crate::dsp::parametric_eq::FilterType;
use std::error::Error;
use std::fs;
use std::path::{Path, PathBuf};

/// The root directory for the local cache of AutoEQ results.
const PRESERVE_DIR: &str = "PRESERVE/AutoEq/results";

/// A prioritized list of AutoEQ data sources, from most to least reliable.
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

/// The types of headphones to search for within the AutoEQ database.
const HEADPHONE_TYPES: &[&str] = &["over-ear", "in-ear", "earbud"];

/// Finds the best available EQ data for a given headphone model.
///
/// This function iterates through the prioritized sources and headphone types, searching for a
/// matching data directory within the local `PRESERVE` cache. When a match is found, it
/// attempts to parse the corresponding "ParametricEQ.txt" file.
///
/// # Arguments
///
/// * `headphone_name` - The name of the headphone model to search for.
///
/// # Returns
///
/// A `Result` containing a `Vec<BandSetting>` on success, or an error if no data is found
/// or parsing fails.
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
                    let content = fs::read_to_string(&eq_file_path)?;
                    return parse_autoeq_text(&content);
                }
            }
        }
    }

    Err(format!("No data found for headphone: {}", headphone_name).into())
}

/// Maps a filter type string from an AutoEQ file to the internal `FilterType` enum.
///
/// # Arguments
///
/// * `autoeq_type` - The filter type string (e.g., "PK", "LS", "HS").
///
/// # Returns
///
/// A `Result` containing the corresponding `FilterType` or an error string if unsupported.
fn map_filter_type(autoeq_type: &str) -> Result<FilterType, String> {
    match autoeq_type {
        "PK" => Ok(FilterType::Peak),
        "LS" => Ok(FilterType::LowShelf),
        "HS" => Ok(FilterType::HighShelf),
        _ => Err(format!("Unsupported filter type: {}", autoeq_type)),
    }
}

/// Parses a string containing AutoEQ CSV data into a vector of `BandSetting`s.
///
/// # Arguments
///
/// * `text_content` - A string slice containing the content of an AutoEQ file.
///
/// # Returns
///
/// A `Result` containing a `Vec<BandSetting>` on success, or a `Box<dyn Error>` if parsing fails.
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