use crate::{
    elevation, inputpoint::InputType, mercator::MercatorPoint, track, wgs84point::WGS84Point,
};

#[derive(Clone)]
pub struct WaypointInfo {
    pub origin: InputType,
    pub distance: f64,
    pub elevation: f64,
    pub inter_distance: f64,
    pub inter_elevation_gain: f64,
    pub inter_slope: f64,
    pub name: String,
    pub description: String,
    pub time: String,
    pub track_index: Option<usize>,
    pub value: Option<i32>,
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
    pub euclidean: MercatorPoint,
    pub track_index: Option<usize>,
    pub origin: InputType,
    pub name: Option<String>,
    pub description: Option<String>,
    pub info: Option<WaypointInfo>,
}

pub type Waypoints = Vec<Waypoint>;

impl Waypoint {
    pub fn create(wgs: WGS84Point, euc: &MercatorPoint, indx: usize, kind: InputType) -> Waypoint {
        Waypoint {
            wgs84: wgs.clone(),
            euclidean: euc.clone(),
            track_index: Some(indx),
            name: None,
            description: None,
            info: None,
            origin: kind,
        }
    }

    pub fn get_track_index(&self) -> usize {
        self.track_index.unwrap()
    }

    pub fn elevation(&self) -> f64 {
        self.wgs84.z()
    }
}

type DateTime = crate::mercator::DateTime;

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
            origin: w.origin.clone(),
            distance,
            inter_distance,
            inter_elevation_gain,
            inter_slope: inter_slope_prev,
            elevation: track.elevation(w.get_track_index()),
            name,
            description,
            time: time.to_rfc3339(),
            track_index: w.track_index,
            value: None,
        }
    }
    pub fn make_waypoint_infos(
        waypoints: &mut Waypoints,
        track: &track::Track,
        start_time: &String,
        speed: &f64,
    ) {
        waypoints.sort_by_key(|w| w.get_track_index());
        let mut infos = Vec::new();
        for k in 0..waypoints.len() {
            let w = &waypoints[k];
            let wprev = match k {
                0 => None,
                _ => Some(&waypoints[k - 1]),
            };
            let step = Self::create_waypoint_info(
                track,
                &track.smooth_elevation,
                start_time,
                speed,
                w,
                wprev,
            );
            infos.push(step.clone());
        }
        for k in 0..waypoints.len() {
            let w = &mut waypoints[k];
            w.info = Some(infos[k].clone());
        }
    }
}
