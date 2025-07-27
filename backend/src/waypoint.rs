use std::str::FromStr;

use crate::utm::UTMPoint;

#[derive(Clone, PartialEq)]
pub enum WaypointOrigin {
    GPX,
    DouglasPeucker,
    MaxStepSize,
}

#[derive(Clone)]
pub struct WaypointInfo {
    pub wgs84: (f64, f64, f64),
    pub utm: UTMPoint,
    pub origin: WaypointOrigin,
    pub distance: f64,
    pub elevation: f64,
    pub inter_distance: f64,
    pub inter_elevation_gain: f64,
    pub inter_slope: f64,
    pub name: String,
    pub description: String,
    pub time: String,
    pub track_index: usize,
}

impl WaypointInfo {
    pub fn profile_label(&self) -> String {
        if !self.name.is_empty() {
            return self.name.clone();
        }
        return format!("{:4.0}", self.distance / 1000f64);
        /*
        use chrono::*;
        let time: DateTime<Utc> = self.time.parse().unwrap();
        return format!("{}", time.format("%H:%M"));
        */
    }
}

#[derive(Clone)]
pub struct Waypoint {
    pub wgs84: (f64, f64, f64),
    pub utm: UTMPoint,
    pub track_index: usize,
    pub origin: WaypointOrigin,
    pub name: Option<String>,
    pub description: Option<String>,
    pub info: Option<WaypointInfo>,
}

fn trim_option(s: Option<String>) -> Option<String> {
    match s {
        Some(data) => Some(String::from_str(data.trim()).unwrap()),
        _ => None,
    }
}

impl Waypoint {
    pub fn create(
        wgs: (f64, f64, f64),
        utm: UTMPoint,
        indx: usize,
        origin: WaypointOrigin,
    ) -> Waypoint {
        Waypoint {
            wgs84: wgs.clone(),
            utm: utm,
            track_index: indx,
            name: None,
            description: None,
            info: None,
            origin: origin,
        }
    }

    pub fn from_gpx(
        gpx: &gpx::Waypoint,
        utm: UTMPoint,
        name: Option<String>,
        description: Option<String>,
    ) -> Waypoint {
        let (lon, lat) = gpx.point().x_y();
        let z = match gpx.elevation {
            Some(_z) => _z,
            _ => 0f64,
        };
        Waypoint {
            //wgs84: (lon, lat, gpx.elevation.unwrap()),
            wgs84: (lon, lat, z),
            utm: utm,
            track_index: usize::MAX,
            origin: WaypointOrigin::GPX,
            name: trim_option(name),
            description: trim_option(description),
            info: None,
        }
    }
}
