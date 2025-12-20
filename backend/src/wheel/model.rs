use std::collections::HashSet;

use crate::{
    inputpoint::{InputPoint, InputType},
    segment::SegmentData,
    track::Track,
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

fn angle(point: &InputPoint, track: &Track) -> f64 {
    let proj = point.track_projections.first().unwrap();
    let index = proj.track_index;
    let part = track.distance(index);
    let total = track.total_distance();
    assert!(part <= total);
    if part == total {
        return 0.0;
    }
    360.0 * part / total
}

fn get_control_points(segment: &SegmentData) -> Vec<InputPoint> {
    match segment
        .pointmaps
        .read()
        .unwrap()
        .maps
        .get(&InputType::Control)
    {
        Some(points) => {
            let ret = points.as_vector();
            log::trace!("segment.id={} controls={}", segment.id(), ret.len());
            if !ret.is_empty() {
                return ret;
            }
        }
        None => {}
    }
    Vec::new()
}

fn get_mid_points(segment: &SegmentData) -> Vec<InputPoint> {
    match segment
        .pointmaps
        .read()
        .unwrap()
        .maps
        .get(&InputType::UserStep)
    {
        Some(map) => {
            let points = map.as_vector();
            if !points.is_empty() {
                return points.clone();
            }
        }
        None => {}
    }
    Vec::new()
}

impl WheelModel {
    pub fn make(segment: &SegmentData, kinds: HashSet<InputType>) -> WheelModel {
        let mut control_points = Vec::new();
        if kinds.contains(&InputType::Control) {
            for c in get_control_points(segment) {
                let cp = CirclePoint {
                    angle: angle(&c, &segment.track),
                    name: c.name(),
                };
                control_points.push(cp);
            }
            control_points.sort_by_key(|p| p.angle.floor() as i32);
        }
        let mut mid_points = Vec::new();
        if kinds.contains(&InputType::UserStep) {
            for c in get_mid_points(segment) {
                let cp = CirclePoint {
                    angle: angle(&c, &segment.track),
                    name: c.name(),
                };
                mid_points.push(cp);
            }
            mid_points.sort_by_key(|p| p.angle.floor() as i32);
        }
        WheelModel {
            control_points,
            mid_points,
        }
    }
}
