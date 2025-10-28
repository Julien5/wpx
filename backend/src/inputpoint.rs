use core::fmt;
use std::{collections::BTreeMap, str::FromStr};

use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::{
    mercator::{EuclideanBoundingBox, MercatorPoint},
    waypoint::{Waypoint, WaypointOrigin},
    wgs84point::WGS84Point,
};

pub type Tags = std::collections::BTreeMap<String, String>;

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum InputType {
    City,
    MountainPass,
    Peak,
    Village,
    Hamlet,
    GPX,
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
    pub euclidian: MercatorPoint,
    pub tags: Tags,
    pub track_projection: Option<TrackProjection>,
    // <= 5 => the label is forcefully placed, with overlap
    // >5 => the label is not placed if no non-overlaping candidate is found.
    pub label_placement_order: usize,
}

impl PartialEq for InputPoint {
    fn eq(&self, other: &Self) -> bool {
        // do not take track_projection and label_placement_order into account.
        // they are transient.
        self.wgs84 == other.wgs84 && self.euclidian == other.euclidian && self.tags == other.tags
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

/*
fn shorten_name(name: &String) -> String {
    if name.len() < 10 {
        return name.clone();
    }
    let parts = name.split_whitespace().collect::<Vec<_>>();
    let n = 1;
    for n in 0..parts.len() {
        let mut ret = parts.clone();
        ret.truncate(n);
        let candidate = ret.join(" ");
        if candidate.len() > 5 {
            return candidate;
        }
    }
    name.clone()
}
*/

impl InputPoint {
    pub fn from_wgs84(wgs84: &WGS84Point, euclidean: &MercatorPoint) -> InputPoint {
        InputPoint {
            wgs84: wgs84.clone(),
            euclidian: euclidean.clone(),
            track_projection: None,
            tags: Tags::new(),
            label_placement_order: usize::MAX,
        }
    }
    pub fn from_gpx(
        wgs84: &WGS84Point,
        euclidean: &MercatorPoint,
        name: &Option<String>,
        description: &Option<String>,
    ) -> InputPoint {
        let mut tags = Tags::new();
        tags.insert("fromgpx".to_string(), "yes".to_string());
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
            euclidian: euclidean.clone(),
            label_placement_order: usize::MAX,
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
    pub fn name(&self) -> Option<String> {
        let ret = self.tags.get("name");
        if ret.is_some() {
            return Some(ret.unwrap().clone());
        }
        for (k, v) in &self.tags {
            if k.contains("name") {
                return Some(v.as_str().to_string());
            }
        }
        return None;
    }
    pub fn short_name(&self) -> Option<String> {
        match self.name() {
            Some(n) => Some(n),
            None => None,
        }
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
    pub fn kind(&self) -> InputType {
        match self.tags.get("fromgpx") {
            Some(_) => return InputType::GPX,
            _ => {}
        };
        match self.tags.get("place") {
            Some(place) => {
                if place == "city" {
                    return InputType::City;
                }
                if place == "town" {
                    return InputType::City;
                }
                if place == "village" {
                    return InputType::Village;
                }
                if place == "hamlet" {
                    return InputType::Hamlet;
                }
            }
            _ => {}
        }
        match self.tags.get("mountain_pass") {
            Some(pass) => {
                if pass == "yes" {
                    return InputType::MountainPass;
                }
            }
            _ => {}
        }
        match self.tags.get("natural") {
            Some(natural) => {
                if natural == "peak" {
                    return InputType::Peak;
                }
            }
            _ => {}
        }
        InputType::Village
    }
    pub fn waypoint(&self) -> Waypoint {
        Waypoint {
            wgs84: self.wgs84.clone(),
            track_index: self.round_track_index(),
            name: self.name().clone(),
            description: None,
            info: None,
            origin: WaypointOrigin::OpenStreetMap,
        }
    }
}

impl fmt::Display for InputPoint {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}=({:.2},{:.2},{:.1})",
            if self.name().is_none() {
                String::new()
            } else {
                self.name().clone().unwrap()
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

impl InputPointMap {
    pub fn new() -> InputPointMap {
        InputPointMap {
            map: BTreeMap::new(),
        }
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
    pub fn extend(&mut self, other: &Self) {
        for (bbox, points) in &other.map {
            self.insert_points(bbox, points);
        }
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
}

#[cfg(test)]
mod tests {
    use super::*;

    fn testpoint() -> InputPoint {
        InputPoint {
            wgs84: WGS84Point::new(&1.0f64, &1.1f64, &0f64),
            euclidian: MercatorPoint::from_xy(&(0f64, 0f64)),
            tags: Tags::new(),
            track_projection: None,
            label_placement_order: usize::MAX,
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
