use std::collections::HashSet;

use crate::{error::RenderError, mercator::DateTime, point_collection::Kind, speed};

#[derive(Clone, PartialOrd, Ord, PartialEq, Eq, Debug)]
pub enum RenderFunction {
    Map,
    Profile,
    Wheel,
    WheelPages,
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct RenderInput {
    pub kinds: HashSet<Kind>,
    pub function: RenderFunction,
    pub size: (i32, i32),
}

#[derive(Clone, Debug, Default)]
pub struct RenderOutput {
    pub svg: String,
    pub error: Option<RenderError>,
}

#[derive(Debug, Clone, Hash, Eq, PartialEq)]
pub enum ProfileIndication {
    None,
    NumericSlope,
}

#[derive(Debug, Clone)]
pub enum ControlSource {
    Segments,
    Waypoints,
    OSM,
}

#[derive(Clone, Debug)]
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
            gpx_name_format: "NAME[*]-TIME[%H:%M]-SLOPE[4.1%]".to_string(),
        }
    }
}

#[derive(Clone, Debug)]
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

#[derive(Clone, Debug)]
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

#[derive(Clone, Debug)]
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
