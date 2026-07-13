use serde::{Deserialize, Serialize};

use crate::mercator::DateTime;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct UserStepsOptions {
    pub step_distance: Option<f64>,
    pub step_elevation_gain: Option<f64>,
    pub gpx_name_format: String,
}

impl Into<crate::parameters::UserStepsOptions> for UserStepsOptions {
    fn into(self) -> crate::parameters::UserStepsOptions {
        crate::parameters::UserStepsOptions {
            step_distance: self.step_distance,
            step_elevation_gain: self.step_elevation_gain,
            gpx_name_format: self.gpx_name_format,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum TimeAxis {
    ConstantSpeed,
    ConstantPower,
}

impl Into<crate::parameters::TimeAxis> for TimeAxis {
    fn into(self) -> crate::parameters::TimeAxis {
        match self {
            TimeAxis::ConstantSpeed => crate::parameters::TimeAxis::ConstantSpeed,
            TimeAxis::ConstantPower => crate::parameters::TimeAxis::ConstantPower,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ProfileOptions {
    pub time_axis: TimeAxis,
}

impl Into<crate::parameters::ProfileOptions> for ProfileOptions {
    fn into(self) -> crate::parameters::ProfileOptions {
        crate::parameters::ProfileOptions {
            time_axis: self.time_axis.into(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct MapOptions {
    // empty
}

impl Into<crate::parameters::MapOptions> for MapOptions {
    fn into(self) -> crate::parameters::MapOptions {
        crate::parameters::MapOptions {}
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
    pub smooth_width: f64,
    /// "17.8" (kmh) or "ACP"
    pub speed: String,
    pub start_time: String,
    pub user_steps_options: UserStepsOptions,
}

pub fn parse_time(data: &str) -> DateTime {
    use chrono::{Local, NaiveDateTime, TimeZone};
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

impl Into<crate::parameters::Parameters> for Parameters {
    fn into(self) -> crate::parameters::Parameters {
        crate::parameters::Parameters {
            control_gpx_name_format: self.control_gpx_name_format,
            debug: self.debug,
            map_options: self.map_options.into(),
            profile_options: self.profile_options.into(),
            segment_length: self.segment_length,
            segment_overlap: self.segment_overlap,
            smooth_width: self.smooth_width,
            speed: self.speed,
            start_time: self.start_time,
            user_steps_options: self.user_steps_options.into(),
        }
    }
}
