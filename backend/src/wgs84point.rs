use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct WGS84Point(f64, f64, f64);

impl WGS84Point {
    pub fn new(lon: &f64, lat: &f64, ele: &f64) -> WGS84Point {
        WGS84Point(*lon, *lat, *ele)
    }
    pub fn new_lonlat(lon: &f64, lat: &f64) -> WGS84Point {
        WGS84Point(*lon, *lat, 0f64)
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
