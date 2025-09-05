pub fn _kmh(_mps: f64) -> f64 {
    // m/s => kmh
    _mps * 3.6f64
}
pub fn mps(_kmh: f64) -> f64 {
    _kmh / 3.6f64
}
