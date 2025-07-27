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
    pub description: String,
    pub time: String,
    pub track_index: usize,
}

impl Step {
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
