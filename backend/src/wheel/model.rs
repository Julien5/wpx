use std::collections::HashSet;

use crate::{
    backend::Segment,
    inputpoint::{InputPoint, InputType},
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
    let proj = point.track_projection.as_ref().unwrap();
    let index = proj.track_index;
    let part = track.distance(index);
    let total = track.total_distance();
    // log::debug!("part:{:.1} total:{:.1}", part, total);
    assert!(part <= total);
    if part == total {
        return 0.0;
    }
    360.0 * part / total
}

fn name(point: &InputPoint) -> String {
    match point.name() {
        Some(text) => text,
        None => "noname".to_string(),
    }
}

fn get_control_points(segment: &Segment) -> Vec<InputPoint> {
    match segment.points.get(&InputType::Control) {
        Some(points) => {
            log::trace!("segment.id={} controls={}", segment.id, points.len());
            if !points.is_empty() {
                return points.clone();
            }
        }
        None => {}
    }
    Vec::new()
}

fn get_mid_points(segment: &Segment) -> Vec<InputPoint> {
    match segment.points.get(&InputType::UserStep) {
        Some(points) => {
            if !points.is_empty() {
                return points.clone();
            }
        }
        None => {}
    }
    Vec::new()
}

impl WheelModel {
    pub fn make(segment: &Segment, kinds: HashSet<InputType>) -> WheelModel {
        let mut control_points = Vec::new();
        if kinds.contains(&InputType::Control) {
            for c in get_control_points(segment) {
                let cp = CirclePoint {
                    angle: angle(&c, &segment.track),
                    name: name(&c),
                };
                control_points.push(cp);
            }
            control_points.sort_by_key(|p| p.angle.floor() as i32);
            for p in &control_points {
                log::debug!("control:{} at {:.1}", p.name, p.angle);
            }
        }
        let mut mid_points = Vec::new();
        if kinds.contains(&InputType::UserStep) {
            for c in get_mid_points(segment) {
                let cp = CirclePoint {
                    angle: angle(&c, &segment.track),
                    name: name(&c),
                };
                mid_points.push(cp);
            }
            mid_points.sort_by_key(|p| p.angle.floor() as i32);
        }
        log::debug!("controls:{}", control_points.len());
        log::debug!("mids:{}", mid_points.len());
        WheelModel {
            control_points,
            mid_points,
        }
    }
}
