use std::str::FromStr;

use crate::{elevation, track, wgs84point::WGS84Point};

#[derive(Clone, PartialEq)]
pub enum WaypointOrigin {
    GPX,
    DouglasPeucker,
    MaxStepSize,
    OpenStreetMap,
}

#[derive(Clone)]
pub struct WaypointInfo {
    pub wgs84: WGS84Point,
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
    pub value: Option<usize>,
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
    pub wgs84: WGS84Point,
    pub track_index: Option<usize>,
    pub origin: WaypointOrigin,
    pub name: Option<String>,
    pub description: Option<String>,
    pub info: Option<WaypointInfo>,
}

pub type Waypoints = Vec<Waypoint>;

fn trim_option(s: Option<String>) -> Option<String> {
    match s {
        Some(data) => Some(String::from_str(data.trim()).unwrap()),
        _ => None,
    }
}

impl Waypoint {
    pub fn create(wgs: WGS84Point, indx: usize, origin: WaypointOrigin) -> Waypoint {
        Waypoint {
            wgs84: wgs.clone(),
            track_index: Some(indx),
            name: None,
            description: None,
            info: None,
            origin: origin,
        }
    }

    pub fn from_gpx(
        gpx: &gpx::Waypoint,
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
            wgs84: WGS84Point::new(&lon, &lat, &z),
            track_index: None,
            origin: WaypointOrigin::GPX,
            name: trim_option(name),
            description: trim_option(description),
            info: None,
        }
    }

    pub fn get_track_index(&self) -> usize {
        self.track_index.unwrap()
    }

    pub fn elevation(&self) -> f64 {
        self.wgs84.z()
    }
}

type DateTime = crate::utm::DateTime;

fn waypoint_time(start_time: DateTime, distance: &f64, speed: &f64) -> DateTime {
    let dt = (distance / speed).ceil() as i64;
    let delta = chrono::TimeDelta::new(dt, 0).unwrap();
    start_time + delta
}

impl WaypointInfo {
    fn create_waypoint_info(
        track: &track::Track,
        smooth: &Vec<f64>,
        start_time: &String,
        speed: &f64,
        w: &Waypoint,
        wprev: Option<&Waypoint>,
    ) -> WaypointInfo {
        assert!(w.get_track_index() < track.len());
        let distance = track.distance(w.get_track_index());
        let (inter_distance, inter_elevation_gain, inter_slope_prev) = match wprev {
            None => (0f64, 0f64, 0f64),
            Some(prev) => {
                let dx =
                    track.distance(w.get_track_index()) - track.distance(prev.get_track_index());
                let dy =
                    elevation::elevation_gain(&smooth, prev.get_track_index(), w.get_track_index());
                let slope = match dx {
                    0f64 => 0f64,
                    _ => dy / dx,
                };
                (dx, dy, slope)
            }
        };
        use chrono::*;
        let start_time: DateTime<Utc> = start_time.parse().unwrap();
        let time = waypoint_time(start_time, &distance, speed);
        let name = match &w.name {
            None => String::new(),
            Some(n) => n.clone(),
        };
        let description = match &w.description {
            None => name.clone(),
            Some(desc) => match name.is_empty() {
                true => format!("{}", desc),
                false => format!("{} - {}", name, desc),
            },
        };
        WaypointInfo {
            wgs84: w.wgs84.clone(),
            origin: w.origin.clone(),
            distance,
            inter_distance,
            inter_elevation_gain,
            inter_slope: inter_slope_prev,
            elevation: track.elevation(w.get_track_index()),
            name,
            description,
            time: time.to_rfc3339(),
            track_index: w.get_track_index(),
            value: None,
        }
    }
    pub fn make_waypoint_infos(
        waypoints: &mut Waypoints,
        track: &track::Track,
        smooth: &Vec<f64>,
        start_time: &String,
        speed: &f64,
    ) {
        let mut infos = Vec::new();
        for k in 0..waypoints.len() {
            let w = &waypoints[k];
            let wprev = match k {
                0 => None,
                _ => Some(&waypoints[k - 1]),
            };
            let step = Self::create_waypoint_info(track, smooth, start_time, speed, w, wprev);
            infos.push(step.clone());
        }
        for k in 0..waypoints.len() {
            let w = &mut waypoints[k];
            w.info = Some(infos[k].clone());
        }
    }
}
