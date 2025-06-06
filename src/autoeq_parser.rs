// src/autoeq_parser.rs

// Re-export FilterType from the parametric_eq module for use here
// This requires parametric_eq to be accessible, e.g. via crate::dsp::parametric_eq
// or by moving FilterType here if it's more general.
// For now, let's assume it's accessible via its original path.
use crate::dsp::parametric_eq::FilterType;

#[derive(Debug, Clone, PartialEq)]
pub struct EqSetting {
    pub filter_type: FilterType,
    pub fc: f32,  // Center/Corner frequency
    pub gain: f32, // Gain in dB
    pub q: f32,   // Q factor
                  // AutoEQ files sometimes have a "BW" (bandwidth) instead of Q for shelves.
                  // We will assume Q is always provided or can be derived if needed.
                  // The RBJ cookbook formulas use Q for shelves as well.
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct AutoEqProfile {
    pub preamp: f32, // Preamp gain in dB
    pub filters: Vec<EqSetting>,
}

impl AutoEqProfile {
    pub fn new() -> Self {
        Default::default()
    }
}

/// Parses text content from an AutoEQ CSV-like file.
/// Example format:
/// Preamp: -5.5 dB
/// Filter 1: ON PK Fc 150 Hz Gain -2.0 dB Q 1.41
/// Filter 2: ON LSC Fc 1000 Hz Gain 3.0 dB Q 0.71
/// Filter 3: ON HSC Fc 5000 Hz Gain -1.5 dB Q 0.71
pub fn parse_autoeq_file_content(content: &str) -> Result<AutoEqProfile, String> {
    let mut profile = AutoEqProfile::new();
    let mut preamp_parsed = false;

    for line in content.lines() {
        let trimmed_line = line.trim();
        if trimmed_line.is_empty() || trimmed_line.starts_with('#') {
            continue; // Skip empty lines and comments
        }

        if trimmed_line.starts_with("Preamp:") {
            if preamp_parsed {
                return Err("Multiple Preamp lines found.".to_string());
            }
            let parts: Vec<&str> = trimmed_line.split_whitespace().collect();
            // Expected: ["Preamp:", value, "dB"]
            if parts.len() >= 2 {
                match parts[1].parse::<f32>() {
                    Ok(val) => {
                        profile.preamp = val;
                        preamp_parsed = true;
                    }
                    Err(_) => return Err(format!("Failed to parse preamp value: '{}'", parts[1])),
                }
            } else {
                return Err(format!("Malformed Preamp line: '{}'", trimmed_line));
            }
        } else if trimmed_line.starts_with("Filter") {
            // Example: "Filter 1: ON PK Fc 150 Hz Gain -2.0 dB Q 1.41"
            // Parts after "Filter N: ON": TYPE Fc X Hz Gain Y dB Q Z
            // We need to find "ON" first.
            let on_keyword = "ON ";
            if let Some(on_pos) = trimmed_line.find(on_keyword) {
                let filter_details_str = trimmed_line[on_pos + on_keyword.len()..].trim();
                let parts: Vec<&str> = filter_details_str.split_whitespace().collect();

                // Expected format: TYPE Fc VAL Hz Gain VAL dB Q VAL
                // Indices:         0    1   2   3  4    5   6  7 8
                if parts.len() < 9 {
                    return Err(format!("Malformed Filter line (not enough parts): '{}'", trimmed_line));
                }

                let filter_type_str = parts[0];
                let fc_str = parts[2];
                let gain_str = parts[5];
                let q_str = parts[8];

                let filter_type = match filter_type_str {
                    "PK" => FilterType::Peak,
                    "LSC" => FilterType::LowShelf,
                    "HSC" => FilterType::HighShelf,
                    _ => return Err(format!("Unknown filter type: '{}' in line '{}'", filter_type_str, trimmed_line)),
                };

                let fc = fc_str.parse::<f32>().map_err(|_| format!("Failed to parse Fc value: '{}' in line '{}'", fc_str, trimmed_line))?;
                let gain = gain_str.parse::<f32>().map_err(|_| format!("Failed to parse Gain value: '{}' in line '{}'", gain_str, trimmed_line))?;
                let q = q_str.parse::<f32>().map_err(|_| format!("Failed to parse Q value: '{}' in line '{}'", q_str, trimmed_line))?;

                profile.filters.push(EqSetting {
                    filter_type,
                    fc,
                    gain,
                    q,
                });

            } else {
                // Line starts with "Filter" but doesn't contain "ON " - could be an inactive filter or malformed.
                // For now, we only parse active "ON" filters.
                // If inactive filters should be noted (e.g. to show them in UI but disabled), this logic would change.
                continue;
            }
        } else {
            // Unknown line format
            return Err(format!("Unknown line format: '{}'", trimmed_line));
        }
    }

    if !preamp_parsed && !profile.filters.is_empty() {
        // Filters found but no preamp line; this might be an error depending on strictness.
        // AutoEQ typically always includes a preamp line, even if it's 0.0 dB.
        return Err("Filters found but no Preamp line specified.".to_string());
    }
    // If no filters and no preamp, it's an empty but valid profile.

    Ok(profile)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dsp::parametric_eq::FilterType; // Ensure FilterType is in scope

    #[test]
    fn test_parse_valid_autoeq_content() {
        let content = r#"
Preamp: -6.2 dB
Filter 1: ON PK Fc 25 Hz Gain 5.5 dB Q 0.62
Filter 2: ON PK Fc 63 Hz Gain -2.7 dB Q 2.29
Filter 3: ON LSC Fc 105 Hz Gain 5.6 dB Q 0.71
Filter 4: ON PK Fc 153 Hz Gain -1.7 dB Q 1.41
Filter 5: ON PK Fc 349 Hz Gain 0.9 dB Q 2.51
Filter 6: ON PK Fc 1000 Hz Gain -1.0 dB Q 1.00
Filter 7: ON PK Fc 1986 Hz Gain -3.2 dB Q 1.78
Filter 8: ON PK Fc 4579 Hz Gain 6.0 dB Q 0.84
Filter 9: ON HSC Fc 10000 Hz Gain -2.0 dB Q 0.71
Filter 10: ON PK Fc 10177 Hz Gain -1.6 dB Q 2.99
        "#;

        let profile = parse_autoeq_file_content(content).expect("Should parse valid content");

        assert_eq!(profile.preamp, -6.2, "Preamp parsing mismatch");
        assert_eq!(profile.filters.len(), 10, "Incorrect number of filters parsed");

        // Check first filter
        assert_eq!(profile.filters[0].filter_type, FilterType::Peak);
        assert_eq!(profile.filters[0].fc, 25.0);
        assert_eq!(profile.filters[0].gain, 5.5);
        assert_eq!(profile.filters[0].q, 0.62);

        // Check third filter (LowShelf)
        assert_eq!(profile.filters[2].filter_type, FilterType::LowShelf);
        assert_eq!(profile.filters[2].fc, 105.0);
        assert_eq!(profile.filters[2].gain, 5.6);
        assert_eq!(profile.filters[2].q, 0.71);

        // Check ninth filter (HighShelf)
        assert_eq!(profile.filters[8].filter_type, FilterType::HighShelf);
        assert_eq!(profile.filters[8].fc, 10000.0);
        assert_eq!(profile.filters[8].gain, -2.0);
        assert_eq!(profile.filters[8].q, 0.71);
    }

    #[test]
    fn test_parse_preamp_only() {
        let content = "Preamp: -3.0 dB";
        let profile = parse_autoeq_file_content(content).expect("Should parse preamp only");
        assert_eq!(profile.preamp, -3.0);
        assert!(profile.filters.is_empty());
    }

    #[test]
    fn test_parse_empty_content() {
        let content = "";
        let profile = parse_autoeq_file_content(content).expect("Should parse empty content as empty profile");
        assert_eq!(profile.preamp, 0.0); // Default preamp
        assert!(profile.filters.is_empty());
    }

    #[test]
    fn test_parse_comments_and_empty_lines() {
        let content = r#"
# This is a comment
Preamp: -1.0 dB

Filter 1: ON PK Fc 100 Hz Gain 1.0 dB Q 1.0
# Another comment
        "#;
        let profile = parse_autoeq_file_content(content).expect("Should parse with comments/empty lines");
        assert_eq!(profile.preamp, -1.0);
        assert_eq!(profile.filters.len(), 1);
        assert_eq!(profile.filters[0].fc, 100.0);
    }

    #[test]
    fn test_malformed_preamp() {
        let content = "Preamp: -3.0dB"; // Missing space
        assert!(parse_autoeq_file_content(content).is_err(), "Should fail on malformed preamp (value)");

        let content2 = "Preamp: dB";
        assert!(parse_autoeq_file_content(content2).is_err(), "Should fail on malformed preamp (no value)");
    }

    #[test]
    fn test_malformed_filter_line() {
        let content = "Filter 1: ON PK Fc 100 Hz Gain -2.0 dB Q"; // Missing Q value
        assert!(parse_autoeq_file_content(content).is_err(), "Should fail on missing Q value");

        let content2 = "Filter 1: ON PK Fc 100 Hz Gain -2.0 Q 1.0"; // Missing 'dB'
        assert!(parse_autoeq_file_content(content2).is_err(), "Should fail on missing 'dB'");

        let content3 = "Filter 1: ON PK Fc 100 Gain -2.0 dB Q 1.0"; // Missing 'Hz'
        assert!(parse_autoeq_file_content(content3).is_err(), "Should fail on missing 'Hz'");

        let content4 = "Filter 1: ON XYZ Fc 100 Hz Gain -2.0 dB Q 1.0"; // Unknown filter type
        assert!(parse_autoeq_file_content(content4).is_err(), "Should fail on unknown filter type");
    }

    #[test]
    fn test_filter_not_on() {
        // Filters that are not "ON" should be skipped currently
        let content = r#"
Preamp: 0.0 dB
Filter 1: OFF PK Fc 100 Hz Gain 1.0 dB Q 1.0
Filter 2: ON PK Fc 200 Hz Gain 2.0 dB Q 2.0
        "#;
        let profile = parse_autoeq_file_content(content).expect("Should parse");
        assert_eq!(profile.filters.len(), 1, "Should only parse 'ON' filters");
        assert_eq!(profile.filters[0].fc, 200.0);
    }

    #[test]
    fn test_multiple_preamp_lines_error() {
        let content = r#"
Preamp: -1.0 dB
Preamp: -2.0 dB
Filter 1: ON PK Fc 100 Hz Gain 1.0 dB Q 1.0
        "#;
        assert!(parse_autoeq_file_content(content).is_err(), "Should fail on multiple preamp lines");
    }

    #[test]
    fn test_filters_without_preamp_error() {
        let content = "Filter 1: ON PK Fc 100 Hz Gain 1.0 dB Q 1.0";
        assert!(parse_autoeq_file_content(content).is_err(), "Should fail if filters exist without preamp");
    }
}
