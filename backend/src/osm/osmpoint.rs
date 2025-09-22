use core::fmt;
use std::str::FromStr;

use serde::{Deserialize, Serialize};
use serde_json::json;

pub type Tags = std::collections::BTreeMap<String, String>;

#[derive(Clone, Serialize, Deserialize)]
pub struct OSMPoint {
    pub lat: f64,
    pub lon: f64,
    pub tags: Tags,
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

impl OSMPoint {
    pub fn ele(&self) -> Option<f64> {
        read::<f64>(self.tags.get("ele"))
    }
    pub fn name(&self) -> Option<String> {
        let mut ret = self.tags.get("short_name");
        if ret.is_some() {
            return ret.cloned();
        }
        ret = self.tags.get("name");
        if ret.is_some() {
            return ret.cloned();
        }
        for (k, v) in &self.tags {
            if k.contains("name") {
                return Some(v.as_str().to_string());
            }
        }
        return None;
    }
    pub fn population(&self) -> Option<i32> {
        read::<i32>(self.tags.get("population"))
    }
}

impl fmt::Display for OSMPoint {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}=({:.2},{:.2},{:.1})",
            if self.name().is_none() {
                String::new()
            } else {
                self.name().clone().unwrap()
            },
            self.lon,
            self.lat,
            if self.ele().is_none() {
                0f64
            } else {
                self.ele().unwrap()
            },
        )
    }
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum OSMType {
    City,
    MountainPass,
    Village,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct OSMPoints {
    pub points: Vec<OSMPoint>,
}

impl OSMPoints {
    pub fn new() -> OSMPoints {
        OSMPoints { points: Vec::new() }
    }
    pub fn from_string(data: &String) -> OSMPoints {
        match serde_json::from_str(data.as_str()) {
            Ok(points) => points,
            Err(e) => {
                log::error!("could not read osmpoints from: {}", data);
                log::error!("because: {:?}", e);
                OSMPoints::new()
            }
        }
    }
    pub fn as_string(&self) -> String {
        json!(self).to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn testpoint() -> OSMPoint {
        OSMPoint {
            lat: 1.0,
            lon: 1.1,
            tags: Tags::new(),
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
        let points = OSMPoints {
            points: vec![p1, p2],
        };
        let data = json!(points);
        log::info!("{}", data)
    }
}
