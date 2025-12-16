use core::fmt;
use std::{
    collections::{BTreeMap, HashSet},
    str::FromStr,
};

use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::{
    bboxes,
    mercator::{EuclideanBoundingBox, MercatorPoint},
    track::Track,
    waypoint::Waypoint,
    wgs84point::WGS84Point,
};

pub type Tags = std::collections::BTreeMap<String, String>;

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum OSMType {
    City,
    MountainPass,
    Peak,
    Village,
    Hamlet,
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, Clone, Hash)]
pub enum InputType {
    GPX,
    OSM,
    UserStep,
    Control,
}

pub type Kinds = HashSet<InputType>;
pub fn allkinds() -> Kinds {
    HashSet::from([
        InputType::UserStep,
        InputType::GPX,
        InputType::OSM,
        InputType::Control,
    ])
}

#[derive(Clone, Serialize, Deserialize, PartialEq, Debug)]
pub struct TrackProjection {
    pub track_floating_index: f64,
    pub track_index: usize,
    pub euclidean: MercatorPoint,
    pub elevation: f64,
    pub track_distance: f64,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct InputPoint {
    pub wgs84: WGS84Point,
    pub euclidean: MercatorPoint,
    pub tags: Tags,
    pub track_projection: Option<TrackProjection>,
}

impl PartialEq for InputPoint {
    fn eq(&self, other: &Self) -> bool {
        // do not take track_projection and label_placement_order into account.
        // they are transient.
        self.wgs84 == other.wgs84 && self.euclidean == other.euclidean && self.tags == other.tags
    }
}

fn read<T>(data: Option<&String>) -> Option<T>
where
    T: FromStr,
{
    match data {
        Some(text) => match text.parse::<T>() {
            Ok(f) => {
                return Some(f);
            }
            Err(_e) => {}
        },
        None => {}
    }
    return None;
}

impl InputPoint {
    pub fn create_user_step_on_track(track: &Track, index: usize, name: &String) -> InputPoint {
        let wgs = track.wgs84[index].clone();
        let euc = track.euclidian[index].clone();
        let mut p = InputPoint::from_wgs84(&wgs, &euc, InputType::UserStep);
        p.tags.insert("name".to_string(), name.clone());
        p.track_projection = Some(TrackProjection {
            track_floating_index: index as f64,
            track_index: index,
            track_distance: 0f64,
            elevation: wgs.z(),
            euclidean: euc.clone(),
        });

        p
    }

    pub fn create_control_on_track(
        track: &Track,
        index: usize,
        name: &String,
        description: &String,
    ) -> InputPoint {
        let wgs = track.wgs84[index].clone();
        let euc = track.euclidian[index].clone();
        let mut p = InputPoint::from_wgs84(&wgs, &euc, InputType::Control);
        p.tags.insert("name".to_string(), name.clone());
        p.tags
            .insert("description".to_string(), description.clone());
        p.track_projection = Some(TrackProjection {
            track_floating_index: index as f64,
            track_index: index,
            track_distance: 0f64,
            elevation: wgs.z(),
            euclidean: euc.clone(),
        });

        p
    }

    pub fn from_wgs84(
        wgs84: &WGS84Point,
        euclidean: &MercatorPoint,
        kind: InputType,
    ) -> InputPoint {
        InputPoint {
            wgs84: wgs84.clone(),
            euclidean: euclidean.clone(),
            track_projection: None,
            tags: Self::tags_for_type(kind),
        }
    }
    pub fn from_gpx(
        wgs84: &WGS84Point,
        euclidean: &MercatorPoint,
        name: &Option<String>,
        description: &Option<String>,
    ) -> InputPoint {
        let mut tags = Self::tags_for_type(InputType::GPX);
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
            track_projection: None,
            tags,
            euclidean: euclidean.clone(),
        }
    }
    pub fn round_track_index(&self) -> Option<usize> {
        match &self.track_projection {
            None => None,
            Some(p) => Some(p.track_floating_index.round() as usize),
        }
    }
    pub fn distance_to_track(&self) -> f64 {
        return self.track_projection.as_ref().unwrap().track_distance;
    }
    pub fn ele(&self) -> Option<f64> {
        read::<f64>(self.tags.get("ele"))
    }
    pub fn name(&self) -> String {
        let ret = self.tags.get("name");
        if ret.is_some() {
            return ret.unwrap().clone();
        }
        for (k, v) in &self.tags {
            if k.contains("name") {
                return v.as_str().to_string();
            }
        }
        return String::new();
    }
    pub fn description(&self) -> String {
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
    pub fn tags_for_type(kind: InputType) -> Tags {
        let mut tags = Tags::new();
        let value = match kind {
            InputType::GPX => "GPX",
            InputType::OSM => "OSM",
            InputType::UserStep => "UserStep",
            InputType::Control => "Control",
        };
        tags.insert("wpxtype".to_string(), value.to_string());
        tags
    }

    pub fn osmkind(&self) -> Option<OSMType> {
        match self.tags.get("place") {
            Some(place) => {
                if place == "city" {
                    return Some(OSMType::City);
                }
                if place == "town" {
                    return Some(OSMType::City);
                }
                if place == "village" {
                    return Some(OSMType::Village);
                }
                if place == "hamlet" {
                    return Some(OSMType::Hamlet);
                }
            }
            _ => {}
        }
        match self.tags.get("mountain_pass") {
            Some(pass) => {
                if pass == "yes" {
                    return Some(OSMType::MountainPass);
                }
            }
            _ => {}
        }
        match self.tags.get("natural") {
            Some(natural) => {
                if natural == "peak" {
                    return Some(OSMType::Peak);
                }
            }
            _ => {}
        }
        None
    }

    pub fn kind(&self) -> InputType {
        match self.tags.get("wpxtype") {
            Some(t) => {
                match t.as_str() {
                    "GPX" => {
                        return InputType::GPX;
                    }
                    "UserStep" => {
                        return InputType::UserStep;
                    }
                    "Control" => {
                        return InputType::Control;
                    }
                    _ => {}
                };
            }
            _ => {}
        }
        return InputType::OSM;
    }

    pub fn waypoint(&self) -> Waypoint {
        Waypoint {
            wgs84: self.wgs84.clone(),
            euclidean: self.euclidean.clone(),
            track_index: self.round_track_index(),
            name: self.name(),
            description: self.description(),
            info: None,
            origin: self.kind(),
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
                log::error!("because: {:?}", e);
                InputPoints::new()
            }
        }
    }
    pub fn as_string(&self) -> String {
        json!(self).to_string()
    }
}

pub struct InputPointMap {
    map: BTreeMap<EuclideanBoundingBox, Vec<InputPoint>>,
}

pub struct InputPointMaps {
    pub maps: BTreeMap<InputType, InputPointMap>,
}

impl InputPointMap {
    pub fn new() -> InputPointMap {
        InputPointMap {
            map: BTreeMap::new(),
        }
    }

    pub fn from_vector(points: &Vec<InputPoint>) -> InputPointMap {
        let mut ret = InputPointMap::new();
        for w in points {
            let euc = &w.euclidean;
            let bbox = bboxes::snap_point(euc, &bboxes::BBOXWIDTH);
            ret.insert_point(&bbox, &w);
        }
        ret
    }

    pub fn insert_point(&mut self, b: &EuclideanBoundingBox, p: &InputPoint) {
        match self.map.get_mut(&b) {
            Some(v) => v.push(p.clone()),
            None => {
                self.map.insert(b.clone(), vec![p.clone()]);
            }
        }
    }
    pub fn insert_points(&mut self, b: &EuclideanBoundingBox, p: &Vec<InputPoint>) {
        match self.map.get_mut(&b) {
            Some(v) => v.extend_from_slice(p),
            None => {
                self.map.insert(b.clone(), p.clone());
            }
        }
    }
    pub fn sort_and_insert(&mut self, p: &Vec<InputPoint>) {
        for w in p {
            let euc = &w.euclidean;
            let bbox = bboxes::snap_point(euc, &bboxes::BBOXWIDTH);
            self.insert_point(&bbox, &w);
        }
    }
    pub fn extend(&mut self, other: &Self) {
        for (bbox, points) in &other.map {
            self.insert_points(bbox, points);
        }
    }

    pub fn clear(&mut self) {
        self.map.clear();
    }

    pub fn as_vector(&self) -> Vec<InputPoint> {
        let mut ret = Vec::new();
        for (_bbox, points) in &self.map {
            ret.extend_from_slice(points);
        }
        ret
    }
    pub fn get(&self, bbox: &EuclideanBoundingBox) -> Option<&Vec<InputPoint>> {
        self.map.get(bbox)
    }
    pub fn retain_points(&mut self, predicate: impl Fn(&InputPoint) -> bool) {
        for (_bbox, points) in &mut self.map {
            points.retain(|w| predicate(w));
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
            track_projection: None,
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
