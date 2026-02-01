use crate::{
    mercator::DateTime,
    parameters::{self, Parameters},
};

// from mps to kmh
pub fn _kmh(_mps: f64) -> f64 {
    // m/s => kmh
    _mps * 3.6f64
}

// from kmh to mps
pub fn mps(_kmh: f64) -> f64 {
    _kmh / 3.6f64
}

pub fn time_at_distance(distance: &f64, parameters: &Parameters) -> DateTime {
    let start_time = parameters::parse_time(&parameters.start_time);
    let dt = (distance / parameters.speed).ceil() as i64;
    let delta = chrono::TimeDelta::new(dt, 0).unwrap();
    start_time + delta
}
