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

#[derive(Debug, Clone)]
pub struct BandSetting {
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
            filter_type: map_filter_type(&record.filter_type_str)?,
            frequency: record.frequency,
            q: record.q,
            gain: record.gain,
        };
        bands.push(band_setting);
    }

    Ok(bands)
}
