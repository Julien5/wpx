use crate::{
    elevation, inputpoint::InputType, mercator::MercatorPoint, parameters::Parameters, speed,
    track, wgs84point::WGS84Point,
};

#[derive(Clone)]
pub struct WaypointInfo {
    pub distance: f64,
    pub elevation: f64,
    pub gpx_name: String,
    pub inter_distance: f64,
    pub inter_elevation_gain: f64,
    pub inter_slope: f64,
    pub name: String,
    pub origin: InputType,
    pub time: String,
    pub track_index: Option<usize>,
    pub description: String,
}

use crate::format::WaypointInfoData;

impl WaypointInfo {
    fn make_gpx_name(data: &WaypointInfoData, parameters: &Parameters) -> String {
        use crate::format;
        format::make_gpx_name(data, parameters)
    }
    pub fn profile_label(&self) -> String {
        if !self.name.is_empty() {
            return self.name.clone();
        }
        return format!("{:4.0}", self.distance / 1000f64);
    }
}

#[derive(Clone)]
pub struct Waypoint {
    pub wgs84: WGS84Point,
    pub euclidean: MercatorPoint,
    pub track_index: Option<usize>,
    pub origin: InputType,
    pub name: String,
    pub description: String,
    pub info: Option<WaypointInfo>,
}

pub type Waypoints = Vec<Waypoint>;

impl Waypoint {
    pub fn create(wgs: WGS84Point, euc: &MercatorPoint, indx: usize, kind: InputType) -> Waypoint {
        Waypoint {
            wgs84: wgs.clone(),
            euclidean: euc.clone(),
            track_index: Some(indx),
            name: String::new(),
            description: String::new(),
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

impl WaypointInfo {
    fn create_waypoint_info(
        track: &track::Track,
        smooth: &Vec<f64>,
        parameters: &Parameters,
        w: &Waypoint,
        wprev: Option<&Waypoint>,
    ) -> WaypointInfo {
        assert!(w.get_track_index() < track.len());
        let distance = track.distance(w.get_track_index());
        let (inter_distance, inter_elevation_gain, inter_slope) = match wprev {
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
        let time = speed::time_at_distance(&distance, parameters);
        let name = w.name.clone();
        let description = w.description.clone();
        let elevation = track.elevation(w.get_track_index());
        let origin = w.origin.clone();
        let data = WaypointInfoData {
            distance,
            elevation,
            inter_distance,
            inter_elevation_gain,
            inter_slope,
            name: name.clone(),
            origin: origin.clone(),
        };
        let gpx_name = Self::make_gpx_name(&data, parameters);
        WaypointInfo {
            description,
            distance,
            elevation,
            gpx_name,
            inter_distance,
            inter_elevation_gain,
            inter_slope,
            name,
            time: time.to_rfc3339(),
            track_index: w.track_index,
            origin,
        }
    }
    pub fn make_waypoint_infos(
        waypoints: &mut Waypoints,
        track: &track::Track,
        parameters: &Parameters,
    ) {
        waypoints.sort_by_key(|w| w.get_track_index());
        let mut infos = Vec::new();
        for k in 0..waypoints.len() {
            let w = &waypoints[k];
            let wprev = match k {
                0 => None,
                _ => Some(&waypoints[k - 1]),
            };
            let step =
                Self::create_waypoint_info(track, &track.smooth_elevation, parameters, w, wprev);
            infos.push(step.clone());
        }
        for k in 0..waypoints.len() {
            let w = &mut waypoints[k];
            w.info = Some(infos[k].clone());
        }
    }
}
