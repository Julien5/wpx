use crate::parameters;
use crate::point_collection::Kind;
use crate::{
    elevation, mercator::MercatorPoint, parameters::Parameters, speed, track,
    wgs84point::WGS84Point,
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
    pub origin: Kind,
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
    pub origin: Kind,
    pub name: String,
    pub description: String,
    pub info: Option<WaypointInfo>,
}

pub type Waypoints = Vec<Waypoint>;

impl Waypoint {
    pub fn create(wgs: WGS84Point, euc: &MercatorPoint, indx: usize, kind: Kind) -> Waypoint {
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

    pub fn get_info(&self) -> &WaypointInfo {
        self.info.as_ref().expect("Waypoint info is missing")
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
        wprev: &Waypoint,
    ) -> WaypointInfo {
        assert!(w.get_track_index() < track.len());
        let distance = track.distance(w.get_track_index());
        let (inter_distance, inter_elevation_gain, inter_slope) = {
            let dx = track.distance(w.get_track_index()) - track.distance(wprev.get_track_index());
            let dy =
                elevation::elevation_gain(&smooth, wprev.get_track_index(), w.get_track_index());
            let slope = match dx {
                0f64 => 0f64,
                _ => dy / dx,
            };
            (dx, dy, slope)
        };
        let time = speed::time_at_distance(distance, parameters);
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
            description: description.clone(),
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
            time: parameters::time_to_iso8601(&time),
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
        let wgs0 = track.wgs84.first().unwrap();
        let euc0 = track.euclidean.first().unwrap();
        let w0 = Waypoint::create(*wgs0, euc0, 0, Kind::UserStep);
        for k in 0..waypoints.len() {
            let w = &waypoints[k];
            let wprev = match k {
                0 => &w0,
                _ => &waypoints[k - 1],
            };
            let info =
                Self::create_waypoint_info(track, &track.smooth_elevation, parameters, w, wprev);
            infos.push(info.clone());
        }
        for k in 0..waypoints.len() {
            let w = &mut waypoints[k];
            w.info = Some(infos[k].clone());
        }
    }
}
