use std::collections::HashSet;

use crate::{error::RenderError, mercator::DateTime, point_collection::Kind, speed};

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
    pub kinds: HashSet<Kind>,
    pub function: RenderFunction,
    pub size: (i32, i32),
}

#[derive(Clone, Debug, Default)]
pub struct RenderOutput {
    pub svg: String,
    pub render_input: RenderInput,
    pub error: Option<RenderError>,
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProfileIndication {
    None,
    NumericSlope,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ControlSource {
    Segments,
    Waypoints,
    OSM,
}

#[derive(Clone, Debug, PartialEq)]
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

#[derive(Clone, Debug, PartialEq)]
pub struct ProfileOptions {
    pub elevation_indicators: Vec<ProfileIndication>,
    pub max_area_ratio: f64,
}

impl Default for ProfileOptions {
    fn default() -> ProfileOptions {
        ProfileOptions {
            elevation_indicators: vec![ProfileIndication::NumericSlope],
            max_area_ratio: 0.20f64,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct MapOptions {
    pub max_area_ratio: f64,
}

impl Default for MapOptions {
    fn default() -> MapOptions {
        MapOptions {
            max_area_ratio: 0.15f64,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Parameters {
    pub control_gpx_name_format: String,
    pub debug: bool,
    pub map_options: MapOptions,
    pub profile_options: ProfileOptions,
    pub segment_length: f64,
    pub segment_overlap: f64,
    pub smooth_width: f64,
    pub speed: f64,
    pub start_time: String,
    pub user_steps_options: UserStepsOptions,
}

impl Default for Parameters {
    fn default() -> Parameters {
        Parameters {
            control_gpx_name_format: "NAME[3]-TIME[%H:%M]-SLOPE[4.1%]".to_string(),
            start_time: time_to_iso8601(&chrono::Local::now()),
            speed: speed::mps(15f64),
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
    let parsed = chrono::DateTime::parse_from_rfc3339(data).expect("Failed to parse");
    use chrono::{DateTime, Local};
    let local_dt: DateTime<Local> = DateTime::from(parsed);
    local_dt
}

pub fn time_to_iso8601(time: &DateTime) -> String {
    time.to_rfc3339()
}
