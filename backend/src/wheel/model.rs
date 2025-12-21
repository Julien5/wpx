use std::collections::HashSet;

use crate::{
    inputpoint::{InputPoint, InputType},
    segment::SegmentData,
    track::Track,
    wheel::shorten::shorten_name,
};

#[derive(Debug, Clone)]
pub struct CirclePoint {
    pub angle: f64,
    pub name: String,
}

pub struct WheelModel {
    pub control_points: Vec<CirclePoint>,
    pub mid_points: Vec<CirclePoint>,
}

fn angles(point: &InputPoint, track: &Track) -> Vec<f64> {
    let mut ret = Vec::new();
    for proj in &point.track_projections {
        let index = proj.track_index;
        let part = track.distance(index);
        let total = track.total_distance();
        assert!(part <= total);
        let a = if part == total {
            0.0
        } else {
            360.0 * part / total
        };
        ret.push(a);
    }
    ret
}

fn get_control_points(segment: &SegmentData) -> Vec<InputPoint> {
    segment.points(&InputType::Control)
}

fn get_mid_points(segment: &SegmentData) -> Vec<InputPoint> {
    segment.points(&InputType::UserStep)
}

fn control_name(w: &InputPoint) -> String {
    format!("{} ({})", w.name(), shorten_name(&w.description()))
}

impl WheelModel {
    pub fn make(segment: &SegmentData, kinds: HashSet<InputType>) -> WheelModel {
        let mut control_points = Vec::new();
        if kinds.contains(&InputType::Control) {
            for c in get_control_points(segment) {
                for a in angles(&c, &segment.track) {
                    let cp = CirclePoint {
                        angle: a,
                        name: control_name(&c),
                    };
                    control_points.push(cp)
                }
            }
            control_points.sort_by_key(|p| p.angle.floor() as i32);
        }
        let mut mid_points = Vec::new();
        if kinds.contains(&InputType::UserStep) {
            for c in get_mid_points(segment) {
                for a in angles(&c, &segment.track) {
                    let cp = CirclePoint {
                        angle: a,
                        name: c.name(),
                    };
                    mid_points.push(cp);
                }
            }
            mid_points.sort_by_key(|p| p.angle.floor() as i32);
        }
        WheelModel {
            control_points,
            mid_points,
        }
    }
}
