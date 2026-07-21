use chrono::{Local, NaiveDateTime, TimeZone};
use serde::{Deserialize, Serialize};

use crate::mercator::DateTime;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[allow(non_snake_case)]
pub struct PowerParameters {
    pub W: f64,              // total weight (rider + bike), kg
    pub Crr: f64,            // rolling resistance coefficient, unitless
    pub Vhw: f64,            // headwind speed, km/h (positive = headwind, negative = tailwind)
    pub A: f64,              // frontal area, m^2
    pub Rho: f64,            // air density, kg/m^3
    pub Cd: f64,             // drag coefficient, unitless
    pub DrivetrainLoss: f64, // drivetrain loss, percent (e.g. 3.0 for 3%)
}

impl Default for PowerParameters {
    fn default() -> PowerParameters {
        PowerParameters {
            W: 90.0,
            Crr: 0.005,
            Vhw: 0.0,
            A: 0.5,
            Rho: 1.225,
            Cd: 0.66,
            DrivetrainLoss: 3f64,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct UserStepsOptions {
    pub step_distance: Option<f64>,
    pub step_elevation_gain: Option<f64>,
    pub gpx_name_format: String,
}

impl Default for UserStepsOptions {
    fn default() -> UserStepsOptions {
        UserStepsOptions {
            step_distance: Some(10_000.0),
            step_elevation_gain: None,
            gpx_name_format: "TIME[%H:%M]-SLOPE[4.1%]".to_string(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum TimeAxis {
    ConstantSpeed,
    ConstantPower,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ProfileOptions {
    pub time_axis: TimeAxis,
}

impl Default for ProfileOptions {
    fn default() -> ProfileOptions {
        ProfileOptions {
            time_axis: TimeAxis::ConstantSpeed,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct MapOptions {
    // empty
}

impl Default for MapOptions {
    fn default() -> MapOptions {
        MapOptions {}
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Parameters {
    pub control_gpx_name_format: String,
    pub debug: bool,
    pub map_options: MapOptions,
    pub profile_options: ProfileOptions,
    pub segment_length: f64,
    pub segment_overlap: f64,
    pub power_parameters: PowerParameters,
    pub smooth_width: f64,
    /// "17.8" (kmh) or "ACP"
    pub speed: String,
    pub start_time: String,
    pub user_steps_options: UserStepsOptions,
}

impl Default for Parameters {
    fn default() -> Parameters {
        Parameters {
            control_gpx_name_format: "NAME[3]-TIME[%H:%M]-SLOPE[4.1%]".to_string(),
            start_time: time_to_iso8601(&chrono::Local::now()),
            speed: format!("KMH-{:.1}", 15.0),
            segment_length: 110f64 * 1000f64,
            segment_overlap: 10f64 * 1000f64,
            power_parameters: PowerParameters::default(),
            smooth_width: 200f64,
            debug: false,
            profile_options: ProfileOptions::default(),
            map_options: MapOptions::default(),
            user_steps_options: UserStepsOptions::default(),
        }
    }
}

pub fn time_to_iso8601(time: &DateTime) -> String {
    time.to_rfc3339()
}

pub fn current_time_as_string() -> String {
    time_to_iso8601(&chrono::Local::now())
}

pub fn parse_time(data: &str) -> DateTime {
    // 1. Try to parse with an explicit TimeZone (RFC3339 / ISO8601 with Z or +HH:MM)
    if let Ok(parsed_with_tz) = chrono::DateTime::parse_from_rfc3339(data) {
        return DateTime::from(parsed_with_tz);
    }

    if let Ok(naive) = NaiveDateTime::parse_from_str(data, "%Y-%m-%dT%H:%M:%S") {
        // Attach the local system timezone to the naive time
        if let Some(local_dt) = Local.from_local_datetime(&naive).single() {
            return local_dt;
        }
    }
    panic!("cannot parse time string:{}", data);
}
