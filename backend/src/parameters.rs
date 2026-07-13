use crate::{error::RenderError, point_collection::Kinds, waypoint::Waypoint};

#[derive(Clone, PartialOrd, Ord, PartialEq, Eq, Debug, Default)]
pub enum RenderFunction {
    Map,
    Profile,
    Wheel,
    WheelPages,
    #[default]
    Unknown,
}

#[derive(Clone, PartialEq, Eq, Debug, Default)]
pub struct RenderInput {
    pub kinds: Kinds,
    pub function: RenderFunction,
    pub size: (i32, i32),
}

#[derive(Clone, Debug, Default)]
pub struct RenderOutput {
    pub svg: String,
    pub render_input: RenderInput,
    pub error: Option<RenderError>,
    pub waypoints: Vec<Waypoint>,
}

#[derive(Debug, Clone)]
pub struct TrackPart {
    pub name: String,
    pub part_index: usize,
    pub length: usize,
}

pub fn karl_order(parts: &Vec<TrackPart>) -> Vec<TrackPart> {
    let mut ret = parts.clone();
    // parts name containing "start" at the begining,
    // parts name containing "end" or "ziel" at the end,
    // the rest in alphabetical order.
    ret.sort_by_key(|part| {
        /* The standard order of precedence is:
           Numbers (0-9)
           Uppercase Letters (A-Z)
           Lowercase Letters (a-z)
        */
        let zero = format!(""); // the empty string comes before "0".
        let infinity = format!("zzzz");
        if part.name.is_empty() {
            return zero;
        }
        let name = part.name.to_lowercase();
        if name.contains("end") {
            return infinity;
        }
        if name.contains("ziel") {
            return infinity;
        }
        if name.contains("start") {
            return zero;
        }
        return name;
    });
    ret
}

pub use crate::trackfile::v1::parameters::*;
