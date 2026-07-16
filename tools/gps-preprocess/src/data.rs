#[derive(Debug, Clone, Copy)]
pub struct Input {
    pub time: f64,
    pub distance: f64,
    pub elevation: f64,
    pub speed: f64,
    pub vertical_speed: f64,
    pub measured_power: f64,
}

#[derive(Debug, Clone, Copy)]
pub struct Smooth {
    pub time: f64,
    pub distance: f64,
    pub elevation: f64,
    pub speed: f64,
    pub smooth_speed: f64,
    #[allow(dead_code)]
    pub vertical_speed: f64,
    pub slope: f64,
    pub measured_power: f64,
}

const EARTH_RADIUS: f64 = 6_371_000.0;

pub fn haversine(lat1: f64, lon1: f64, lat2: f64, lon2: f64) -> f64 {
    let dlat = (lat2 - lat1).to_radians();
    let dlon = (lon2 - lon1).to_radians();
    let a = (dlat / 2.0).sin().powi(2)
        + lat1.to_radians().cos() * lat2.to_radians().cos() * (dlon / 2.0).sin().powi(2);
    let c = 2.0 * a.sqrt().asin();
    EARTH_RADIUS * c
}

pub fn interpolate_nans_slice(values: &mut [f64]) {
    let n = values.len();
    let mut i = 0;
    while i < n {
        if values[i].is_nan() {
            let start = i;
            while i < n && values[i].is_nan() {
                i += 1;
            }
            let end = i;
            if start > 0 && end < n {
                let left = values[start - 1];
                let right = values[end];
                let span = (end - start + 1) as f64;
                for j in start..end {
                    let t = (j - start + 1) as f64 / span;
                    values[j] = left + t * (right - left);
                }
            }
        } else {
            i += 1;
        }
    }
}
