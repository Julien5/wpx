use core::fmt;
use std::{
    collections::{BTreeMap, BTreeSet},
    str::FromStr,
};

use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::{
    mercator::{DateTime, MercatorPoint, WebMercatorProjection},
    point_collection::Kind,
    tile::{self, Tile},
    track::Track,
    track_projection::{TrackProjection, TrackProjections},
    waypoint::Waypoint,
    wgs84point::WGS84Point,
};

pub type Tags = std::collections::BTreeMap<String, String>;

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct InputPoint {
    pub wgs84: WGS84Point,
    pub euclidean: MercatorPoint,
    pub data: InputPointData,
    pub track_projections: TrackProjections,
    pub index: Option<usize>,
}

impl PartialEq for InputPoint {
    fn eq(&self, other: &Self) -> bool {
        // do not take track_projection and label_placement_order into account.
        // they are transient.
        self.wgs84 == other.wgs84 && self.euclidean == other.euclidean && self.data == other.data
    }
}

impl Eq for InputPoint {}

#[derive(Default, Clone, Serialize, Deserialize, Debug, PartialEq, Eq)]
pub struct ControlData {
    pub nearest_waypoint_id: Option<usize>,
    pub name: String,
    pub waypoint_name: String,
    pub waypoint_description: String,
    pub segment_name: String,
    pub cutoff_time: Option<DateTime>,
}

impl ControlData {
    fn join_non_empty(parts: &[&str]) -> String {
        parts
            .iter()
            .filter(|s| !s.is_empty())
            .copied()
            .collect::<Vec<_>>()
            .join(", ")
    }

    pub fn description(&self) -> String {
        Self::join_non_empty(&[&self.waypoint_name, &self.waypoint_description, &{
            if !self.segment_name.is_empty() {
                format!("End of {}", self.segment_name)
            } else {
                String::new()
            }
        }])
    }

    pub fn is_end(&self) -> bool {
        self.name == "END"
    }
    pub fn is_start(&self) -> bool {
        self.name == "START"
    }
}

#[derive(Default, Clone, Serialize, Deserialize, Debug, PartialEq, Eq)]
pub struct OSMData {
    pub tags: Tags,
    pub osmid: String,
}

impl OSMData {
    fn read<T: FromStr>(data: &String) -> Option<T> {
        data.parse().ok()
    }
    pub fn population(&self) -> usize {
        //read::<f64>(self.tags.get("population"))
        let min_population = match self.osm_type() {
            Kind::Cities => 4000,
            Kind::Villages => 1000,
            Kind::Hamlets => 100,
            _ => 0,
        };
        if let Some(value) = self.tags.get("population") {
            return OSMData::read::<usize>(value).unwrap_or(min_population);
        }
        for (k, _v) in &self.tags {
            if !k.contains("population") {
                continue;
            }
            if let Some(value) = self.tags.get(k) {
                return OSMData::read::<usize>(value).unwrap_or(min_population);
            }
        }
        min_population
    }
    pub fn name(&self) -> String {
        self.tags
            .get("name")
            .map(|s| s.trim().to_string())
            .or_else(|| {
                self.tags
                    .iter()
                    .find(|(k, _)| k.contains("name"))
                    .map(|(_, v)| v.trim().to_string())
            })
            .unwrap_or_default()
    }
    pub fn description(&self) -> String {
        self.tags
            .get("description")
            .map(|s| s.trim().to_string())
            .or_else(|| {
                self.tags
                    .iter()
                    .find(|(k, _)| k.contains("description"))
                    .map(|(_, v)| v.trim().to_string())
            })
            .unwrap_or_default()
    }
    pub fn elevation(&self) -> f64 {
        if let Some(value) = self.tags.get("ele") {
            return OSMData::read::<f64>(value).unwrap_or(0f64);
        }
        0f64
    }

    pub fn osm_type(&self) -> Kind {
        match self.tags.get("mountain_pass") {
            Some(pass) => {
                if pass == "yes" {
                    return Kind::Mountains;
                }
            }
            _ => {}
        }
        match self.tags.get("natural") {
            Some(natural) => {
                if natural == "peak" {
                    return Kind::Mountains;
                }
            }
            _ => {}
        }
        match self.tags.get("place") {
            Some(place) => {
                if place == "city" {
                    return Kind::Cities;
                }
                if place == "town" {
                    return Kind::Cities;
                }
                if place == "village" {
                    return Kind::Villages;
                }
                if place == "hamlet" {
                    return Kind::Hamlets;
                }
            }
            _ => {}
        }
        debug_assert!(false);
        Kind::Cities
    }
}

#[derive(Default, Clone, Serialize, Deserialize, Debug, PartialEq, Eq)]
pub struct GPXWaypointData {
    pub name: Option<String>,
    pub description: Option<String>,
}

#[derive(Clone, Serialize, Deserialize, Debug, PartialEq, Eq)]
pub enum InputPointData {
    Control(ControlData),
    OSM(OSMData),
    GPXWaypoint(GPXWaypointData),
    CutOff,
}

impl InputPointData {
    pub fn as_control_mut(&mut self) -> Option<&mut ControlData> {
        if let InputPointData::Control(ref mut control) = self {
            Some(control)
        } else {
            None
        }
    }
    pub fn as_control(&self) -> Option<&ControlData> {
        if let InputPointData::Control(ref control) = self {
            Some(control)
        } else {
            None
        }
    }

    pub fn as_gpxwaypoint_mut(&mut self) -> Option<&mut GPXWaypointData> {
        if let InputPointData::GPXWaypoint(ref mut point) = self {
            Some(point)
        } else {
            None
        }
    }
    pub fn as_gpxwaypoint(&self) -> Option<&GPXWaypointData> {
        if let InputPointData::GPXWaypoint(ref point) = self {
            Some(point)
        } else {
            None
        }
    }

    pub fn as_osm_mut(&mut self) -> Option<&mut OSMData> {
        if let InputPointData::OSM(ref mut point) = self {
            Some(point)
        } else {
            None
        }
    }
    pub fn as_osm(&self) -> Option<&OSMData> {
        if let InputPointData::OSM(ref point) = self {
            Some(point)
        } else {
            None
        }
    }
}

impl InputPoint {
    pub fn index(&self) -> Option<usize> {
        self.index.clone()
    }
    pub fn map_id(&self) -> String {
        // The track projection is not taken into account
        format!(
            "{}|{}|{}|{:?}",
            self.wgs84.longitude(),
            self.wgs84.latitude(),
            self.kind(),
            self.data
        )
    }
    pub fn create_user_step_on_track(wgs: &WGS84Point, proj: TrackProjection) -> InputPoint {
        let euc = &proj.euclidean;
        let mut p = InputPoint::from_wgs84(&wgs, &euc, Kind::CutOff);
        p.track_projections = BTreeSet::from([proj]);
        p
    }

    pub fn control_waypoint_origin_index(&self) -> Option<usize> {
        if self.kind() != Kind::Controls {
            return None;
        }
        self.data.as_control().unwrap().nearest_waypoint_id.clone()
    }

    pub fn create_control_on_track(
        proj: TrackProjection,
        segment_name: &str,
        waypoint_name: &str,
        waypoint_description: &str,
        nearest_waypoint_id: &Option<usize>,
    ) -> InputPoint {
        let euc = proj.euclidean.clone();
        let wgs = WebMercatorProjection::make().unproject(&euc);
        let mut p = InputPoint::from_wgs84(&wgs, &euc, Kind::Controls);
        let data = ControlData {
            name: format!("{}", waypoint_name),
            waypoint_name: format!("{}", waypoint_name),
            waypoint_description: format!("{}", waypoint_description),
            segment_name: format!("{}", segment_name),
            nearest_waypoint_id: nearest_waypoint_id.clone(),
            cutoff_time: None,
        };
        p.track_projections = BTreeSet::from([{ proj }]);
        p.data = InputPointData::Control(data);
        p
    }

    pub fn clone_with_proj(&self, proj: &TrackProjection) -> InputPoint {
        let mut w = self.clone();
        w.track_projections.clear();
        w.track_projections.insert(proj.clone());
        w
    }

    pub fn dmax(&self) -> f64 {
        match &self.data {
            InputPointData::OSM(d) => {
                if self.kind() != Kind::Mountains {
                    let pop = d.population();
                    return 20f64 * (pop as f64).sqrt();
                }
            }
            _ => {}
        }
        300.0
    }

    pub fn is_close_to_track(&self) -> bool {
        if self.track_projections.is_empty() {
            return false;
        }
        let d = self.track_projections.first().unwrap().track_distance;
        d < self.dmax()
    }

    pub fn from_wgs84(wgs84: &WGS84Point, euclidean: &MercatorPoint, kind: Kind) -> InputPoint {
        let data: InputPointData = match kind {
            Kind::Controls => InputPointData::Control(ControlData::default()),
            Kind::GPXWaypoints => InputPointData::GPXWaypoint(GPXWaypointData::default()),
            Kind::CutOff => InputPointData::CutOff,
            Kind::Cities | Kind::Hamlets | Kind::Villages | Kind::Mountains => {
                InputPointData::OSM(OSMData::default())
            }
        };
        InputPoint {
            wgs84: wgs84.clone(),
            euclidean: euclidean.clone(),
            track_projections: TrackProjections::new(),
            data,
            index: None,
        }
    }
    pub fn from_gpx(
        wgs84: &WGS84Point,
        euclidean: &MercatorPoint,
        name: &Option<String>,
        description: &Option<String>,
    ) -> InputPoint {
        let data = GPXWaypointData {
            name: name.clone(),
            description: description.clone(),
        };
        InputPoint {
            wgs84: wgs84.clone(),
            track_projections: TrackProjections::new(),
            data: InputPointData::GPXWaypoint(data),
            euclidean: euclidean.clone(),
            index: None,
        }
    }

    pub fn is_in_range(&self, range: &std::ops::Range<usize>) -> bool {
        for proj in &self.track_projections {
            if range.contains(&proj.track_index) {
                return true;
            }
        }
        false
    }

    pub fn is_on_segment(&self, start: f64, end: f64) -> bool {
        for proj in &self.track_projections {
            let d = &proj.distance_on_track_to_projection;
            if start <= *d && *d <= end {
                return true;
            }
        }
        false
    }

    pub fn is_in_distance_range(&self, start: f64, end: f64) -> bool {
        for proj in &self.track_projections {
            let d = proj.distance_on_track_to_projection;
            if start <= d && d <= end {
                return true;
            }
        }
        false
    }

    pub fn distance_to_track(&self) -> f64 {
        if self.track_projections.is_empty() {
            return f64::MAX;
        }
        // returns the minimum of all track_distances
        self.track_projections
            .iter()
            .map(|proj| proj.track_distance)
            .fold(f64::INFINITY, f64::min)
    }
    pub fn name(&self) -> String {
        match &self.data {
            InputPointData::OSM(d) => d.name(),
            InputPointData::GPXWaypoint(d) => d.name.as_deref().unwrap_or("").to_string(),
            InputPointData::Control(d) => d.name.clone(),
            InputPointData::CutOff => String::new(),
        }
    }
    pub fn description(&self) -> String {
        match &self.data {
            InputPointData::OSM(d) => d.description(),
            InputPointData::GPXWaypoint(d) => d.description.as_deref().unwrap_or("").to_string(),
            InputPointData::Control(d) => d.description(),
            InputPointData::CutOff => String::new(),
        }
    }
    pub fn kind(&self) -> Kind {
        match &self.data {
            InputPointData::Control(_) => Kind::Controls,
            InputPointData::GPXWaypoint(_) => Kind::GPXWaypoints,
            InputPointData::CutOff => Kind::CutOff,
            InputPointData::OSM(d) => d.osm_type(),
        }
    }

    pub fn flatten_projections(points: &[InputPoint]) -> Vec<(usize, TrackProjection)> {
        // return a (index,projection) vector, sorted by projections, that can be used to
        // get the points in order of their projections on the track.
        let mut result: Vec<(usize, TrackProjection)> = points
            .iter()
            .enumerate()
            .flat_map(|(idx, point)| {
                point
                    .track_projections
                    .iter()
                    .map(move |proj| (idx, proj.clone()))
            })
            .collect();
        // the index does not matter for the sort.
        result.sort_by(|(_indexa, proja), (_indexb, projb)| {
            proja
                .track_floating_index
                .total_cmp(&projb.track_floating_index)
        });
        debug_assert!(result.len() >= points.len());
        result
    }

    pub fn waypoint(&self, projection: &TrackProjection) -> Waypoint {
        let has_custom_time = if let InputPointData::Control(control) = &self.data {
            control.cutoff_time.is_some()
        } else {
            false
        };
        Waypoint {
            wgs84: self.wgs84.clone(),
            euclidean: self.euclidean.clone(),
            track_index: Some(projection.track_index),
            name: self.name(),
            description: self.description(),
            has_custom_time,
            info: None,
            origin: self.kind(),
            index: self.index(),
        }
    }
}

impl fmt::Display for InputPoint {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}=({:.2},{:.2},{:.1})",
            self.name(),
            self.wgs84.longitude(),
            self.wgs84.latitude(),
            self.wgs84.z(),
        )
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct InputPoints {
    pub points: Vec<InputPoint>,
}

impl InputPoints {
    pub fn new() -> InputPoints {
        InputPoints { points: Vec::new() }
    }
    pub fn from_string(data: &String) -> InputPoints {
        match serde_json::from_str(data.as_str()) {
            Ok(points) => points,
            Err(e) => {
                log::error!("could not read osmpoints from: {}", data);
                log::error!("because: {}", e);
                InputPoints::new()
            }
        }
    }
    pub fn as_string(&self) -> String {
        json!(self).to_string()
    }
}

#[derive(Clone)]
pub struct InputPointMap {
    pub map: BTreeMap<Tile, Vec<InputPoint>>,
}

impl InputPointMap {
    pub fn new() -> InputPointMap {
        InputPointMap {
            map: BTreeMap::new(),
        }
    }

    pub fn iter(&self) -> impl Iterator<Item = &InputPoint> {
        self.map.values().flat_map(|vector| vector.iter())
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut InputPoint> {
        self.map.values_mut().flat_map(|vector| vector.iter_mut())
    }

    pub fn from_string(data: &str) -> Result<InputPointMap, serde_json::Error> {
        let map: Vec<(Tile, Vec<InputPoint>)> = serde_json::from_str(data)?;
        Ok(InputPointMap {
            map: map.into_iter().collect(),
        })
    }

    pub fn as_string(&self) -> Result<String, serde_json::Error> {
        let entries: Vec<(&Tile, &Vec<InputPoint>)> = self.map.iter().collect();
        serde_json::to_string(&entries)
    }

    pub fn from_points(points: &[InputPoint]) -> InputPointMap {
        let mut ret = InputPointMap::new();
        for w in points {
            ret.insert_point(&w);
        }
        ret
    }

    pub fn insert_point(&mut self, p: &InputPoint) {
        let tile = tile::Tile::for_point(&p.euclidean);
        match self.map.get_mut(&tile) {
            Some(v) => v.push(p.clone()),
            None => {
                self.map.insert(tile, vec![p.clone()]);
            }
        }
    }

    pub fn insert_points(&mut self, b: &Tile, p: &Vec<InputPoint>) {
        match self.map.get_mut(&b) {
            Some(v) => v.extend_from_slice(p),
            None => {
                self.map.insert(b.clone(), p.clone());
            }
        }
    }

    pub fn as_vector(&self) -> Vec<InputPoint> {
        let mut ret = Vec::new();
        for (_bbox, points) in &self.map {
            ret.extend_from_slice(points);
        }
        ret
    }

    pub fn get(&self, tile: &Tile) -> Option<&Vec<InputPoint>> {
        self.map.get(tile)
    }

    pub fn get_mut(&mut self, tile: &Tile) -> Option<&mut Vec<InputPoint>> {
        self.map.get_mut(tile)
    }

    pub fn filter_for_track(&mut self, track: &Track) {
        if track.total_distance() > 200000f64 {
            // discard hamlets
            for (_tile, points) in &mut self.map {
                points.retain(|p| p.kind() != Kind::Hamlets);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::math::Point2D;

    use super::*;

    fn testpoint() -> InputPoint {
        InputPoint {
            wgs84: WGS84Point::new(&1.0f64, &1.1f64, &0f64),
            euclidean: MercatorPoint::from_point2d(&Point2D::new(0f64, 0f64)),
            data: InputPointData::CutOff,
            track_projections: TrackProjections::new(),
            index: None,
        }
    }

    #[test]
    fn point() {
        let p = testpoint();
        let data = json!(p);
        log::info!("{}", data)
    }

    #[test]
    fn points() {
        let p1 = testpoint();
        let p2 = testpoint();
        let points = InputPoints {
            points: vec![p1, p2],
        };
        let data = json!(points);
        log::info!("{}", data)
    }
}
