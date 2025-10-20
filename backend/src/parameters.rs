use crate::speed;

#[derive(Debug, Clone, Hash, Eq, PartialEq)]
pub enum ProfileIndication {
    None,
    GainTicks,
    NumericSlope,
}

#[derive(Clone)]
pub struct ProfileOptions {
    pub elevation_indicators: std::collections::HashSet<ProfileIndication>,
}

impl Default for ProfileOptions {
    fn default() -> ProfileOptions {
        ProfileOptions {
            elevation_indicators: std::collections::HashSet::default(),
        }
    }
}

#[derive(Clone)]
pub struct Parameters {
    pub debug: bool,
    pub max_step_size: f64,
    pub start_time: String,
    pub speed: f64,
    pub segment_length: f64,
    pub segment_overlap: f64,
    pub smooth_width: f64,
    pub profile_options: ProfileOptions,
}

impl Default for Parameters {
    fn default() -> Parameters {
        Parameters {
            max_step_size: 15f64 * 1000f64,
            start_time: chrono::offset::Utc::now().to_rfc3339(),
            speed: speed::mps(15f64),
            segment_length: 100f64 * 1000f64,
            segment_overlap: 10f64 * 1000f64,
            smooth_width: 200f64,
            debug: false,
            profile_options: ProfileOptions::default(),
        }
    }
}
