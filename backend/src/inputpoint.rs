use core::fmt;
use std::{
    collections::{BTreeMap, BTreeSet},
    str::FromStr,
};

use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::{
    mercator::MercatorPoint,
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
    pub tags: Tags,
    pub track_projections: TrackProjections,
}

impl PartialEq for InputPoint {
    fn eq(&self, other: &Self) -> bool {
        // do not take track_projection and label_placement_order into account.
        // they are transient.
        self.wgs84 == other.wgs84 && self.euclidean == other.euclidean && self.tags == other.tags
    }
}

impl Eq for InputPoint {}

fn read<T: FromStr>(data: Option<&String>) -> Option<T> {
    data.and_then(|text| text.parse().ok())
}

impl InputPoint {
    pub fn id(&self) -> String {
        let zero = String::new();
        let index = self.tags.get("index").unwrap_or(&zero);
        format!(
            "{}|{}|{}|{}",
            self.wgs84.longitude(),
            self.wgs84.latitude(),
            self.kind(),
            index,
        )
    }
    pub fn create_user_step_on_track(
        wgs: &WGS84Point,
        proj: TrackProjection,
        name: &String,
    ) -> InputPoint {
        let euc = &proj.euclidean;
        let mut p = InputPoint::from_wgs84(&wgs, &euc, Kind::UserStep);
        p.tags.insert("name".to_string(), name.clone());
        p.track_projections = BTreeSet::from([proj]);
        p
    }

    pub fn control_waypoint_origin_id(&self) -> String {
        if self.kind() != Kind::Controls {
            return String::new();
        }
        self.tags.get("nearest_waypoint_id").unwrap().clone()
    }

    pub fn create_control_on_track(
        track: &Track,
        proj: TrackProjection,
        control_index: usize,
        segment_name: &str,
        waypoint_name: &str,
        waypoint_description: &str,
        nearest_waypoint_id: &str,
    ) -> InputPoint {
        let index = proj.track_index;
        let wgs = track.wgs84[index].clone();
        let euc = track.euclidean[index].clone();
        let mut p = InputPoint::from_wgs84(&wgs, &euc, Kind::Controls);
        p.tags
            .insert("name".to_string(), format!("K{}", control_index));
        p.tags
            .insert("waypoint_name".to_string(), waypoint_name.into());
        p.tags
            .insert("control_index".to_string(), format!("K{}", control_index));
        p.tags.insert(
            "waypoint_description".to_string(),
            waypoint_description.into(),
        );
        p.tags
            .insert("segment_name".to_string(), segment_name.into());
        p.tags.insert(
            "nearest_waypoint_id".to_string(),
            nearest_waypoint_id.into(),
        );
        p.track_projections = BTreeSet::from([{ proj }]);

        p
    }

    pub fn from_wgs84(wgs84: &WGS84Point, euclidean: &MercatorPoint, kind: Kind) -> InputPoint {
        InputPoint {
            wgs84: wgs84.clone(),
            euclidean: euclidean.clone(),
            track_projections: TrackProjections::new(),
            tags: Self::tags_for_type(kind),
        }
    }
    pub fn from_gpx(
        wgs84: &WGS84Point,
        euclidean: &MercatorPoint,
        name: &Option<String>,
        description: &Option<String>,
    ) -> InputPoint {
        let mut tags = Self::tags_for_type(Kind::GPXWaypoints);
        if name.is_some() {
            tags.insert("name".to_string(), name.as_ref().unwrap().clone());
        }
        if description.is_some() {
            tags.insert(
                "description".to_string(),
                description.as_ref().unwrap().clone(),
            );
        }
        InputPoint {
            wgs84: wgs84.clone(),
            track_projections: TrackProjections::new(),
            tags,
            euclidean: euclidean.clone(),
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
    pub fn ele(&self) -> Option<f64> {
        read::<f64>(self.tags.get("ele"))
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
        if self.kind() == Kind::Controls {
            return self.control_description();
        }
        let desc = self.tags.get("description");
        match desc {
            Some(data) => data.clone(),
            None => String::new(),
        }
    }
    pub fn population(&self) -> Option<i32> {
        match self.tags.get("population") {
            None => {
                for (k, _v) in &self.tags {
                    if k.contains("population") {
                        return read::<i32>(self.tags.get(k));
                    }
                }
            }
            _ => {
                return read::<i32>(self.tags.get("population"));
            }
        }
        None
    }
    pub fn tags_for_type(kind: Kind) -> Tags {
        let mut tags = Tags::new();
        let value = match kind {
            Kind::GPXWaypoints => "GPX",
            Kind::Villages => "village",
            Kind::Hamlets => "hamlet",
            Kind::Cities => "city",
            Kind::Mountains => "mountains",
            Kind::UserStep => "UserStep",
            Kind::Controls => "Control",
        };
        tags.insert("wpxtype".to_string(), value.to_string());
        tags
    }

    pub fn kind(&self) -> Kind {
        match self.tags.get("wpxtype") {
            Some(t) => match t.as_str() {
                "GPX" => {
                    return Kind::GPXWaypoints;
                }
                "UserStep" => {
                    return Kind::UserStep;
                }
                "Control" => {
                    return Kind::Controls;
                }
                _ => {}
            },
            _ => {}
        }
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
        assert!(false);
        return Kind::Mountains;
    }

    pub fn flatten_projections(points: &[InputPoint]) -> Vec<(usize, TrackProjection)> {
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
        assert!(result.len() >= points.len());
        result.sort_by(|a, b| {
            a.1.track_floating_index
                .partial_cmp(&b.1.track_floating_index)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        assert!(result.len() >= points.len());
        result
    }

    fn join_non_empty(parts: &[&str]) -> String {
        parts
            .iter()
            .filter(|s| !s.is_empty())
            .copied()
            .collect::<Vec<_>>()
            .join(", ")
    }

    fn control_description(&self) -> String {
        let empty = String::new();
        let segment_name = self.tags.get("segment_name").unwrap_or(&empty);
        let waypoint_name = self.tags.get("waypoint_name").unwrap_or(&empty);
        let waypoint_description = self.tags.get("waypoint_description").unwrap_or(&empty);
        Self::join_non_empty(&[waypoint_name, waypoint_description, &{
            if !segment_name.is_empty() {
                format!("End of {}", segment_name)
            } else {
                String::new()
            }
        }])
    }

    pub fn waypoint(&self, projection: &TrackProjection) -> Waypoint {
        Waypoint {
            wgs84: self.wgs84.clone(),
            euclidean: self.euclidean.clone(),
            track_index: Some(projection.track_index),
            name: self.name(),
            description: self.description(),
            info: None,
            origin: self.kind(),
            id: self.id(),
        }
    }
}

impl fmt::Display for InputPoint {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}=({:.2},{:.2},{:.1})",
            if self.name().is_empty() {
                String::new()
            } else {
                self.name()
            },
            self.wgs84.longitude(),
            self.wgs84.latitude(),
            if self.ele().is_none() {
                0f64
            } else {
                self.ele().unwrap()
            },
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

    pub fn from_vector(points: &Vec<InputPoint>) -> InputPointMap {
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
            tags: Tags::new(),
            track_projections: TrackProjections::new(),
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
