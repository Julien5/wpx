use crate::speed;
use crate::{inputpoint::InputType, parameters::Parameters};

// a bit messy, mostly AI generated
#[allow(dead_code)]
pub struct WaypointInfoData {
    pub distance: f64,
    pub elevation: f64,
    pub inter_distance: f64,
    pub inter_elevation_gain: f64,
    pub inter_slope: f64,
    pub name: String,
    pub origin: InputType,
}

fn format_slope(slope_ratio: f64, specifier: &str) -> String {
    let has_percent = specifier.ends_with('%');
    let specifier_cleaned = specifier.trim_end_matches('%');

    let mut parts = specifier_cleaned.split('.');

    // Extract Width and Precision
    let width_str = parts.next().unwrap_or("0");
    let precision_str = parts.next().unwrap_or("1"); // Default to 1 decimal place

    let width = width_str.parse::<usize>().unwrap_or(0);
    let precision = precision_str.parse::<usize>().unwrap_or(1);

    // Slope is a ratio (e.g., 0.101). Convert to percent grade (e.g., 10.1)
    let slope_grade = slope_ratio * 100.0;

    // Create the formatting string using the standard `format!` syntax:
    // {:>width.precision f} ensures padding and decimal places are applied.
    let formatted_value = format!(
        "{:>width$.precision$}",
        slope_grade,
        width = width,
        precision = precision
    );

    // Append the percent suffix if requested
    if has_percent {
        format!("{}{}", formatted_value, "%")
    } else {
        formatted_value
    }
}

pub fn make_gpx_name(data: &WaypointInfoData, parameters: &Parameters) -> String {
    use regex::Regex;
    // let format_regex: Regex = Regex::new(r"(TIME|SLOPE)\[([^\]]+)\]").unwrap();
    let format_regex: Regex = Regex::new(r"(TIME|SLOPE|NAME)(?:\[([^\]]+)\])?").unwrap();
    let format = match data.origin {
        InputType::UserStep => parameters.user_steps_options.gpx_name_format.clone(),
        InputType::Control => parameters.control_gpx_name_format.clone(),
        _ => String::new(),
    };
    if format.is_empty() {
        return data.name.clone();
    }
    log::debug!(
        "name={} origin={:?} format={}",
        data.name,
        data.origin,
        format
    );
    let mut result = format.to_string();
    let original_format = format.to_string(); // Keep original for iterating
    let time = speed::time_at_distance(&data.distance, parameters);

    // Iterate over all matched placeholders in the format string
    for cap in format_regex.captures_iter(&original_format) {
        let field_type = &cap[1];
        let placeholder = &cap[0];

        // This variable will hold the formatted string for substitution
        let formatted_value = match field_type {
            "TIME" => {
                let specifier = &cap[2];
                // The specifier is a Chrono format string (e.g., "%H:%M")
                // NOTE: Using "%H:%M" corresponds to the example TIME[HH:MM]
                time.format(specifier).to_string()
            }
            "SLOPE" => {
                let specifier = &cap[2];
                // The specifier is a custom W.P[%] string (e.g., "4.1" or "4.1%")
                format_slope(data.inter_slope, specifier)
            }
            "NAME" => data.name.clone(),
            _ => {
                // Should not happen based on the regex
                String::new()
            }
        };

        // Replace the placeholder. We use replacen(..., 1) to replace only the current match,
        // avoiding issues if the same field appears multiple times.
        result = result.replacen(placeholder, &formatted_value, 1);
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{NaiveDate, TimeZone, Utc};

    // Helper function to create a test WaypointInfoData
    fn setup_test_data(slope: f64, hour: u32, minute: u32) -> WaypointInfoData {
        let test_datetime = Utc.from_utc_datetime(
            &NaiveDate::from_ymd_opt(2025, 1, 1)
                .unwrap()
                .and_hms_opt(hour, minute, 30) // Set seconds to 30 for rounding checks
                .unwrap(),
        );

        WaypointInfoData {
            distance: 15000.0,
            elevation: 10.0,
            inter_distance: 50.0,
            inter_slope: slope,
            inter_elevation_gain: 50.0,
            name: "P2".to_string(),
            origin: InputType::UserStep,
        }
    }

    fn parameters(format: &str) -> Parameters {
        let mut ret = Parameters::default();
        ret.start_time = "1985-04-12T08:00:00.00Z".to_string();
        ret.speed = speed::mps(15f64);
        ret.user_steps_options.gpx_name_format = format.to_string();
        ret
    }

    #[test]
    fn test_time_formatting() {
        // Time is 12:32:30 UTC
        let data = setup_test_data(0.0, 12, 32);

        // Example 1: TIME[HH:MM] -> "12:32" (Using chrono's "%H:%M")
        let format1 = "TIME[%H:%M]";
        assert_eq!(make_gpx_name(&data, &parameters(&format1)), "09:00");

        // Test different time format
        let format2 = "TIME[%H:%M:%S]";
        assert_eq!(make_gpx_name(&data, &parameters(&format2)), "09:00:00");
    }

    #[test]
    fn test_name_formatting() {
        // Time is 12:32:30 UTC
        let data = setup_test_data(0.0, 12, 32);

        // Example 1: TIME[HH:MM] -> "12:32" (Using chrono's "%H:%M")
        let format1 = "NAME-TIME[%H:%M]";
        assert_eq!(make_gpx_name(&data, &parameters(&format1)), "P2-09:00");
    }

    #[test]
    fn test_slope_formatting_standard_examples() {
        // Slope is 9.1% (Ratio 0.091)
        let data = setup_test_data(0.091, 0, 0);

        // Example 2: SLOPE[4.1] -> " 9.1"
        let format1 = "SLOPE[4.1]";
        // 9.1 is 3 characters, width 4 means one leading space.
        assert_eq!(make_gpx_name(&data, &parameters(&format1)), " 9.1");

        // Example 3: SLOPE[4.1%] -> " 9.1%"
        let format2 = "SLOPE[4.1%]";
        assert_eq!(make_gpx_name(&data, &parameters(&format2)), " 9.1%");

        // High slope (10.1% / Ratio 0.101)
        let data_high = setup_test_data(0.101, 0, 0);

        // Test with high slope and tight width
        let format3 = "SLOPE[4.1%]";
        // 10.1% is 5 characters, width 4 is ignored because the number is wider.
        assert_eq!(make_gpx_name(&data_high, &parameters(&format3)), "10.1%");

        // Test with high slope and sufficient width
        let format4 = "SLOPE[6.2]";
        // 10.10 is 5 chars, width 6 means one leading space.
        assert_eq!(make_gpx_name(&data_high, &parameters(&format4)), " 10.10");
    }

    #[test]
    fn test_slope_formatting_edge_cases() {
        // Low slope (1.5%)
        let data_low = setup_test_data(0.015, 0, 0);

        // No width, just precision (defaults to width 0)
        let format1 = "SLOPE[.0%]";
        assert_eq!(make_gpx_name(&data_low, &parameters(&format1)), "2%"); // 1.5 rounded to 2

        // No precision, just width (defaults to precision 1)
        let format2 = "SLOPE[5.]";
        assert_eq!(make_gpx_name(&data_low, &parameters(&format2)), "  1.5"); // ' 1.5'
    }

    #[test]
    fn test_combined_formatting() {
        // Time 12:32, Slope 10.1%
        let data = setup_test_data(0.101, 12, 32);

        // Example 4: TIME[HH:MM]-SLOPE[4.1%] -> "12:32-10.1%"
        let format1 = "TIME[%H:%M]-SLOPE[4.1%]";
        // Note: The example shows "12:32-10.1%". Since 10.1% is 5 chars, width 4 is insufficient,
        // so no padding is applied to the slope.
        assert_eq!(make_gpx_name(&data, &parameters(&format1)), "09:00-10.1%");

        // Test complex string with multiple placeholders
        let format2 = "T:TIME[%H] | S:SLOPE[4.0%] | T:TIME[%M]";
        // Slope 10.10% is 6 chars, width 4 is insufficient.
        assert_eq!(
            make_gpx_name(&data, &parameters(&format2)),
            "T:09 | S:  10% | T:00"
        );

        // Test a format that requires padding
        let data_low_slope = setup_test_data(0.05, 10, 5); // 5.0% slope
        let format3 = "T:TIME[%H]-S:SLOPE[5.1%]"; // Width 5 for slope
                                                  // Slope 5.0% is 4 characters, width 5 adds one space: " 5.0%"
        assert_eq!(
            make_gpx_name(&data_low_slope, &parameters(&format3)),
            "T:09-S:  5.0%"
        );
    }
}
