use std::collections::BTreeMap;

use crate::backend::Segment;
use crate::inputpoint::InputPoint;
use crate::mercator::DateTime;
use crate::parameters;
use crate::point_collection::{is_osm, Kind};
use crate::segment::SegmentData;
use crate::speed::TimeParameters;
use crate::track_projection::TrackProjection;
use crate::{
    elevation, mercator::MercatorPoint, parameters::Parameters, track, wgs84point::WGS84Point,
};

#[derive(Clone, Debug)]
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

pub struct ExportParameters {
    pub parameters: Parameters,
    pub time_parameters: TimeParameters,
}

impl WaypointInfo {
    fn make_gpx_name(data: &WaypointInfoData, parameters: &ExportParameters) -> String {
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

#[derive(Clone, Debug)]
pub struct Waypoint {
    pub wgs84: WGS84Point,
    pub euclidean: MercatorPoint,
    pub track_index: Option<usize>,
    pub origin: Kind,
    pub name: String,
    pub description: String,
    pub info: Option<WaypointInfo>,
    pub id: String,
}

pub type Waypoints = Vec<Waypoint>;
pub type WaypointsMap = BTreeMap<TrackProjection, Waypoint>;

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
            id: String::new(),
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
    fn create_waypoint_info_simple(
        track: &track::Track,
        time: &DateTime,
        w: &Waypoint,
    ) -> WaypointInfo {
        assert!(w.get_track_index() < track.len());
        let distance = track.distance(w.get_track_index());
        let name = w.name.clone();
        let description = w.description.clone();
        let elevation = track.elevation(w.get_track_index());
        let origin = w.origin.clone();
        let inter_distance = 0f64;
        let inter_elevation_gain = 0f64;
        let inter_slope = 0f64;
        WaypointInfo {
            description,
            distance,
            elevation,
            gpx_name: String::new(),
            inter_distance,
            inter_elevation_gain,
            inter_slope,
            name,
            time: parameters::time_to_iso8601(&time),
            track_index: w.track_index,
            origin,
        }
    }
    fn create_waypoint_info_cross(
        track: &track::Track,
        smooth: &Vec<f64>,
        parameters: &ExportParameters,
        proj: &TrackProjection,
        w: &Waypoint,
        wprev: &Waypoint,
    ) -> WaypointInfo {
        assert!(w.get_track_index() < track.len());
        let time = parameters
            .time_parameters
            .time(proj.distance_on_track_to_projection);
        let mut ret = Self::create_waypoint_info_simple(track, &time, w);
        (
            ret.inter_distance,
            ret.inter_elevation_gain,
            ret.inter_slope,
        ) = {
            let dx = track.distance(w.get_track_index()) - track.distance(wprev.get_track_index());
            let dy =
                elevation::elevation_gain(&smooth, wprev.get_track_index(), w.get_track_index());
            let slope = match dx {
                0f64 => 0f64,
                _ => dy / dx,
            };
            (dx, dy, slope)
        };

        let data = WaypointInfoData {
            distance: ret.distance,
            elevation: ret.elevation,
            inter_distance: ret.inter_distance,
            inter_elevation_gain: ret.inter_elevation_gain,
            inter_slope: ret.inter_slope,
            name: ret.name.clone(),
            description: ret.description.clone(),
            origin: ret.origin.clone(),
        };
        ret.gpx_name = Self::make_gpx_name(&data, parameters);

        ret
    }
    pub fn make_waypoint_infos(
        waypoints: &mut WaypointsMap,
        track: &track::Track,
        parameters: &ExportParameters,
    ) {
        let wgs0 = track.wgs84.first().unwrap();
        let euc0 = track.euclidean.first().unwrap();
        let w0 = Waypoint::create(*wgs0, euc0, 0, Kind::CutOff);
        let mut wprev = w0.clone();
        for (proj, w) in waypoints.iter_mut() {
            let info = Self::create_waypoint_info_cross(
                track,
                &track.smooth_elevation,
                parameters,
                proj,
                w,
                &wprev,
            );
            w.info = Some(info.clone());
            wprev = w.clone();
        }
    }
}

pub fn waypoint_for_segment(points: &Vec<InputPoint>, segment: &SegmentData) -> Waypoints {
    let mut waypoints = Vec::new();
    for p in points {
        for proj in &p.track_projections {
            let d = proj.distance_on_track_to_projection;
            if segment.start() <= d && d <= segment.end() {
                let mut w = p.waypoint(&proj);
                let time = segment
                    .time_parameters
                    .time(proj.distance_on_track_to_projection);
                let info = WaypointInfo::create_waypoint_info_simple(&segment.track, &time, &w);
                w.info = Some(info);
                waypoints.push(w);
            }
        }
    }
    waypoints
}

pub fn decimate(segment: &Segment, waypoints: &Vec<Waypoint>, n: usize) -> Vec<Waypoint> {
    let mut remains = waypoints.clone();
    let mut ret = Vec::new();
    let dmin = (segment.end - segment.start) * 0.1;
    while !remains.is_empty() {
        let (pos, name) = {
            let c0 = remains.first().unwrap();
            (c0.euclidean.point2d(), c0.name.clone())
        };
        let mut candidates = remains.clone();
        candidates.retain(|c| {
            let same_position = c.euclidean.point2d() == pos;
            let same_name = c.name == name;
            same_position && same_name
        });
        assert!(candidates.len() >= 1);
        remains.retain(|c| {
            let same_position = c.euclidean.point2d() == pos;
            let same_name = c.name == name;
            if same_position && same_name {
                return false;
            }
            // remove only OSM points (no Controls and GPX waypoints)
            if !is_osm(&c.origin) {
                return true;
            }
            let d = c.euclidean.point2d().distance_to(&pos);
            d > dmin
        });
        let next_n = ret.len() + candidates.len();
        if next_n > n {
            break;
        }
        ret.extend_from_slice(&candidates);
    }
    // now we can sort.
    ret.sort_by_key(|w| w.track_index);
    ret
}

pub fn table(segment: &SegmentData, points: &Vec<InputPoint>) -> Vec<Waypoint> {
    waypoint_for_segment(&points, segment)
}

pub fn group_waypoints(waypoints: &[Waypoint], split_indices: &[usize]) -> Vec<Vec<Waypoint>> {
    let mut ranges: Vec<(usize, usize)> = Vec::with_capacity(split_indices.len() + 1);
    let mut start = 0;

    for &split in split_indices {
        let end = start + waypoints[start..].partition_point(|w| w.track_index.unwrap() < split);
        ranges.push((start, end));
        start = end;
    }
    ranges.push((start, waypoints.len()));

    // Merge small groups into the previous one
    let mut merged: Vec<(usize, usize)> = Vec::with_capacity(ranges.len());
    for (start, end) in ranges {
        if end - start < 3 && !merged.is_empty() {
            merged.last_mut().unwrap().1 = end;
        } else {
            merged.push((start, end));
        }
    }

    merged
        .iter()
        .map(|&(s, e)| waypoints[s..e].to_vec())
        .collect()
}
