use std::str::FromStr;

#[derive(Clone, Debug)]
pub struct WGS84Point(f64, f64, f64);

impl WGS84Point {
    pub fn new(lon: &f64, lat: &f64, ele: &f64) -> WGS84Point {
        WGS84Point(*lon, *lat, *ele)
    }
    pub fn from_xy(xy: &(f64, f64)) -> WGS84Point {
        WGS84Point(xy.0, xy.1, 0f64)
    }
    pub fn x(&self) -> f64 {
        return self.0;
    }
    pub fn y(&self) -> f64 {
        return self.1;
    }
    pub fn z(&self) -> f64 {
        return self.2;
    }
    pub fn xy(&self) -> (f64, f64) {
        (self.0, self.1)
    }
    pub fn latitude(&self) -> f64 {
        return self.y();
    }
    pub fn longitude(&self) -> f64 {
        return self.x();
    }
}

#[derive(Clone, PartialEq)]
pub enum WaypointOrigin {
    GPX,
    DouglasPeucker,
    MaxStepSize,
    OpenStreetMap,
}

#[derive(Clone)]
pub struct WaypointInfo {
    pub wgs84: WGS84Point,
    pub origin: WaypointOrigin,
    pub distance: f64,
    pub elevation: f64,
    pub inter_distance: f64,
    pub inter_elevation_gain: f64,
    pub inter_slope: f64,
    pub name: String,
    pub description: String,
    pub time: String,
    pub track_index: usize,
    pub value: Option<usize>,
}

impl WaypointInfo {
    pub fn profile_label(&self) -> String {
        if !self.name.is_empty() {
            return self.name.clone();
        }
        return format!("{:4.0}", self.distance / 1000f64);
        /*
        use chrono::*;
        let time: DateTime<Utc> = self.time.parse().unwrap();
        return format!("{}", time.format("%H:%M"));
        */
    }
}

#[derive(Clone)]
pub struct Waypoint {
    pub wgs84: WGS84Point,
    pub track_index: Option<usize>,
    pub origin: WaypointOrigin,
    pub name: Option<String>,
    pub description: Option<String>,
    pub info: Option<WaypointInfo>,
}

pub type Waypoints = Vec<Waypoint>;

fn trim_option(s: Option<String>) -> Option<String> {
    match s {
        Some(data) => Some(String::from_str(data.trim()).unwrap()),
        _ => None,
    }
}

impl Waypoint {
    pub fn create(wgs: WGS84Point, indx: usize, origin: WaypointOrigin) -> Waypoint {
        Waypoint {
            wgs84: wgs.clone(),
            track_index: Some(indx),
            name: None,
            description: None,
            info: None,
            origin: origin,
        }
    }

    pub fn from_gpx(
        gpx: &gpx::Waypoint,
        name: Option<String>,
        description: Option<String>,
    ) -> Waypoint {
        let (lon, lat) = gpx.point().x_y();
        let z = match gpx.elevation {
            Some(_z) => _z,
            _ => 0f64,
        };
        Waypoint {
            //wgs84: (lon, lat, gpx.elevation.unwrap()),
            wgs84: WGS84Point::new(&lon, &lat, &z),
            track_index: None,
            origin: WaypointOrigin::GPX,
            name: trim_option(name),
            description: trim_option(description),
            info: None,
        }
    }

    pub fn get_track_index(&self) -> usize {
        self.track_index.unwrap()
    }

    pub fn elevation(&self) -> f64 {
        self.wgs84.2
    }
}
