use crate::gpsdata::WaypointOrigin;
use crate::utm::UTMPoint;

#[derive(Clone)]
pub struct Step {
    pub wgs84: (f64, f64, f64),
    pub utm: UTMPoint,
    pub origin: WaypointOrigin,
    pub distance: f64,
    pub elevation: f64,
    pub inter_distance: f64,
    pub inter_elevation_gain: f64,
    pub inter_slope: f64,
    pub name: String,
    pub time: String,
    pub track_index: usize,
}

impl Step {
    pub fn profileLabel(&self) -> String {
        use chrono::*;
        let time: DateTime<Utc> = self.time.parse().unwrap();
        return format!("{}", time.format("%H:%M"));
    }
}
