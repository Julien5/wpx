use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Parameters {
    pub debug: bool,
}

impl Into<crate::parameters::Parameters> for Parameters {
    fn into(self) -> crate::parameters::Parameters {
        let def = crate::parameters::Parameters::default();
        crate::parameters::Parameters {
            control_gpx_name_format: def.control_gpx_name_format,
            debug: self.debug,
            map_options: def.map_options.into(),
            profile_options: def.profile_options.into(),
            segment_length: def.segment_length,
            segment_overlap: def.segment_overlap,
            power_parameters: def.power_parameters,
            smooth_width: def.smooth_width,
            speed: def.speed,
            start_time: def.start_time,
            user_steps_options: def.user_steps_options.into(),
        }
    }
}
