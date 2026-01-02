use std::collections::HashSet;

use crate::{
    inputpoint::{InputPoint, InputType},
    segment::SegmentData,
    track::Track,
    track_projection::TrackProjection,
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

fn angle(proj: &TrackProjection, total: f64) -> f64 {
    let topmargin = super::constants::ARCANGLE.to_degrees() / 2.0;
    let a = (360.0 - 2.0 * topmargin) / total;
    let b = topmargin;
    let x = proj.distance_on_track_to_projection;
    assert!(x <= total);
    a * x + b
}

fn angles(point: &InputPoint, track: &Track) -> Vec<f64> {
    let total = track.total_distance();
    point
        .track_projections
        .iter()
        .map(|proj| angle(proj, total))
        .collect()
}

fn get_control_points(segment: &SegmentData) -> Vec<InputPoint> {
    segment.points(&InputType::Control)
}

fn get_mid_points(segment: &SegmentData) -> Vec<InputPoint> {
    segment.points(&InputType::UserStep)
}

fn control_name(w: &InputPoint) -> String {
    format!("{}", w.name())
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
