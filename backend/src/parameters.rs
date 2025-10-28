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
    pub step_distance: Option<f64>,
    pub step_elevation_gain: Option<f64>,
}

impl Default for ProfileOptions {
    fn default() -> ProfileOptions {
        ProfileOptions {
            elevation_indicators: std::collections::HashSet::default(),
            step_distance: None,
            step_elevation_gain: None,
        }
    }
}

#[derive(Clone)]
pub struct MapOptions {
    pub nmax: Option<usize>,
}

impl Default for MapOptions {
    fn default() -> MapOptions {
        MapOptions { nmax: None }
    }
}

#[derive(Clone)]
pub struct Parameters {
    pub debug: bool,
    pub start_time: String,
    pub speed: f64,
    pub segment_length: f64,
    pub segment_overlap: f64,
    pub smooth_width: f64,
    pub profile_options: ProfileOptions,
    pub map_options: MapOptions,
}

impl Default for Parameters {
    fn default() -> Parameters {
        Parameters {
            start_time: chrono::Local::now().to_rfc3339(),
            speed: speed::mps(15f64),
            segment_length: 100f64 * 1000f64,
            segment_overlap: 10f64 * 1000f64,
            smooth_width: 200f64,
            debug: false,
            profile_options: ProfileOptions::default(),
            map_options: MapOptions::default(),
        }
    }
}
