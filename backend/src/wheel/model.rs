use std::collections::HashSet;

use crate::{
    controls,
    inputpoint::{InputPoint, InputType},
    segment::SegmentData,
    track::Track,
    wheel::time_points,
};

#[derive(Debug, Clone)]
pub struct CirclePoint {
    pub angle: f64,
    pub name: String,
}

pub struct WheelModel {
    pub control_points: Vec<CirclePoint>,
    pub mid_points: Vec<CirclePoint>,
    pub has_start_control: bool,
    pub has_end_control: bool,
    pub time_points: Vec<CirclePoint>,
}

pub fn angle(x: f64, total: f64) -> f64 {
    let topmargin = super::constants::ARCANGLE / 2.0;
    let a = (360.0 - 2.0 * topmargin) / total;
    let b = topmargin;
    assert!(x <= total);
    a * x + b
}

fn angles(point: &InputPoint, track: &Track) -> Vec<f64> {
    let total = track.total_distance();
    point
        .track_projections
        .iter()
        .map(|proj| angle(proj.distance_on_track_to_projection, total))
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
        let (mut has_start_control, mut has_end_control) = (false, false);
        if kinds.contains(&InputType::Control) {
            let controls = get_control_points(segment);
            (has_start_control, has_end_control) =
                controls::has_startend_controls(&segment.track, &controls);
            for c in &controls {
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
            has_start_control,
            has_end_control,
            time_points: time_points::generate(&segment),
        }
    }
}
