use core::fmt;

use serde::{Deserialize, Serialize};
use serde_json::json;

#[derive(Clone, Serialize, Deserialize)]
pub struct OSMPoint {
    pub lat: f64,
    pub lon: f64,
    pub ele: Option<f64>,
    pub name: Option<String>,
}

impl fmt::Display for OSMPoint {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}=({:.2},{:.2},{:.1})",
            if self.name.is_none() {
                String::new()
            } else {
                self.name.clone().unwrap()
            },
            self.lon,
            self.lat,
            if self.ele.is_none() {
                0f64
            } else {
                self.ele.unwrap()
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
            ele: Some(12.0),
            name: Some("hi".to_string()),
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
