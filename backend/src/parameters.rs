use crate::speed;

#[derive(Clone)]
pub struct Parameters {
    pub epsilon: f64,
    pub max_step_size: f64,
    pub start_time: String,
    pub speed: f64,
    pub segment_length: f64,
    pub smooth_width: f64,
}

impl Default for Parameters {
    fn default() -> Parameters {
        Parameters {
            epsilon: 100f64,
            max_step_size: 15f64 * 1000f64,
            start_time: chrono::offset::Utc::now().to_rfc3339(),
            speed: speed::mps(15f64),
            segment_length: 100f64 * 1000f64,
            smooth_width: 200f64,
        }
    }
}
