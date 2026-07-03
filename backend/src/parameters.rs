use crate::{error::RenderError, mercator::DateTime, point_collection::Kinds, waypoint::Waypoint};
use serde::{Deserialize, Serialize};

#[derive(Clone, PartialOrd, Ord, PartialEq, Eq, Debug, Default)]
pub enum RenderFunction {
    Map,
    Profile,
    Wheel,
    WheelPages,
    #[default]
    Unknown,
}

#[derive(Clone, PartialEq, Eq, Debug, Default)]
pub struct RenderInput {
    pub kinds: Kinds,
    pub function: RenderFunction,
    pub size: (i32, i32),
}

#[derive(Clone, Debug, Default)]
pub struct RenderOutput {
    pub svg: String,
    pub render_input: RenderInput,
    pub error: Option<RenderError>,
    pub waypoints: Vec<Waypoint>,
}

#[derive(Debug, Clone)]
pub struct TrackPart {
    pub name: String,
    pub part_index: usize,
    pub length: usize,
}

pub fn karl_order(parts: &Vec<TrackPart>) -> Vec<TrackPart> {
    let mut ret = parts.clone();
    // parts name containing "start" at the begining,
    // parts name containing "end" or "ziel" at the end,
    // the rest in alphabetical order.
    ret.sort_by_key(|part| {
        /* The standard order of precedence is:
           Numbers (0-9)
           Uppercase Letters (A-Z)
           Lowercase Letters (a-z)
        */
        let zero = format!(""); // the empty string comes before "0".
        let infinity = format!("zzzz");
        if part.name.is_empty() {
            return zero;
        }
        let name = part.name.to_lowercase();
        if name.contains("end") {
            return infinity;
        }
        if name.contains("ziel") {
            return infinity;
        }
        if name.contains("start") {
            return zero;
        }
        return name;
    });
    ret
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ProfileIndication {
    None,
    NumericSlope,
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

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ProfileOptions {
    pub elevation_indicators: Vec<ProfileIndication>,
}

impl Default for ProfileOptions {
    fn default() -> ProfileOptions {
        ProfileOptions {
            elevation_indicators: vec![ProfileIndication::NumericSlope],
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
            smooth_width: 200f64,
            debug: false,
            profile_options: ProfileOptions::default(),
            map_options: MapOptions::default(),
            user_steps_options: UserStepsOptions::default(),
        }
    }
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

pub fn time_to_iso8601(time: &DateTime) -> String {
    time.to_rfc3339()
}

pub fn current_time_as_string() -> String {
    time_to_iso8601(&chrono::Local::now())
}
