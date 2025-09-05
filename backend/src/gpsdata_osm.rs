use crate::gpsdata::distance_wgs84;
use crate::gpsdata::Track;
use crate::project;
use crate::utm::UTMPoint;
use crate::waypoint::Waypoint;
use crate::waypoint::WaypointOrigin;
use crate::waypoint::Waypoints;
use std::collections::HashMap;
use std::fmt;

use proj4rs::Proj;
use serde_json::Value;

fn read_tags(tags: &serde_json::Value) -> (Option<String>, Option<f64>) {
    let map = tags.as_object().unwrap();
    let mut name = match map.get("name") {
        Some(value) => Some(value.as_str().unwrap().to_string()),
        None => None,
    };
    if name.is_none() {
        name = match map.get("loc_name") {
            Some(value) => Some(value.as_str().unwrap().to_string()),
            None => None,
        };
    }
    let ele = match map.get("ele") {
        Some(value) => Some(value.as_str().unwrap().parse::<f64>().unwrap()),
        None => None,
    };
    (name, ele)
}

fn read_f64(map: &serde_json::Map<String, Value>, name: &str) -> f64 {
    map.get(name).unwrap().as_f64().unwrap()
}

struct OSMPoint {
    lat: f64,
    lon: f64,
    ele: Option<f64>,
    name: Option<String>,
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

fn read_element(element: &serde_json::Value) -> Result<OSMPoint, &'static str> {
    assert!(element.is_object());
    let map = element.as_object().unwrap();
    match map.get("type") {
        Some(value) => {
            if value != "node" {
                return Err("no city");
            }
        }
        None => {
            return Err("no city");
        }
    }
    let lat = read_f64(map, "lat");
    let lon = read_f64(map, "lon");
    let (name, ele) = read_tags(map.get("tags").unwrap());
    let city = OSMPoint {
        lat,
        lon,
        ele,
        name,
    };
    Ok(city)
}

fn read_elements(elements: &serde_json::Value) -> Vec<OSMPoint> {
    assert!(elements.is_array());
    let mut ret = Vec::new();
    for e in elements.as_array().unwrap() {
        match read_element(e) {
            Ok(city) => ret.push(city),
            Err(msg) => {
                println!("{}", msg);
            }
        }
    }
    ret
}

fn read_content(content: &[u8]) -> Vec<OSMPoint> {
    let json: serde_json::Value =
        serde_json::from_slice(content).expect("JSON was not well-formatted");
    assert!(json.is_object());
    //assert!(json.as_object().unwrap().len() == 1);
    let mut ret = Vec::new();
    let map = json.as_object().unwrap();
    ret.extend(read_elements(map.get("elements").unwrap()));
    ret
}

fn read_osm_points(content: &[u8]) -> Waypoints {
    let cities = read_content(content);
    let mut ret = Vec::new();
    // TODO: fix proj!
    let spec = "+proj=longlat +ellps=WGS84 +datum=WGS84 +no_defs";
    let wgs84 = Proj::from_proj_string(spec).unwrap();
    let spec = "+proj=utm +zone=32 +datum=WGS84 +units=m +no_defs +type=crs";
    let utm32n = Proj::from_proj_string(spec).unwrap();
    for city in cities {
        let (lon, lat) = (city.lon, city.lat);
        let mut p = (lon.to_radians(), lat.to_radians());
        proj4rs::transform::transform(&wgs84, &utm32n, &mut p).unwrap();
        let ele = match city.ele {
            Some(m) => m,
            None => 0f64,
        };
        let w = Waypoint {
            wgs84: (lon, lat, ele),
            utm: UTMPoint(p.0, p.1),
            track_index: None,
            name: city.name,
            description: None,
            info: None,
            origin: WaypointOrigin::OpenStreetMap,
        };
        ret.push(w);
    }
    ret
}

#[derive(Debug, PartialEq, Eq, Hash)]
pub enum OSMType {
    City,
    MountainPass,
}

pub type OSMWaypoints = HashMap<OSMType, Waypoints>;

fn retain(waypoints: &mut Waypoints, track: &Track, delta: f64) {
    project::project_on_track(track, waypoints);
    waypoints.retain(|w| {
        let index = w.track_index.unwrap();
        let p1 = (track.wgs84[index].0, track.wgs84[index].1);
        let p2 = (w.wgs84.0, w.wgs84.1);
        let d = distance_wgs84(p1.0, p1.1, p2.0, p2.1);
        d < delta
    })
}

pub fn read(track: &Track, distance: f64) -> OSMWaypoints {
    let mut ret = HashMap::new();
    let mut cities = read_osm_points(include_bytes!("../data/cities-south.json"));
    retain(&mut cities, track, distance);
    let mut passes = read_osm_points(include_bytes!("../data/moutain-pass-south.json"));
    retain(&mut passes, track, distance);
    ret.insert(OSMType::City, cities);
    ret.insert(OSMType::MountainPass, passes);
    ret
}
