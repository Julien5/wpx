use std::collections::HashSet;

use crate::{
    controls,
    inputpoint::{InputPoint, InputType},
    mercator::DateTime,
    segment::SegmentData,
    track::Track,
    wheel::time_points,
};

pub struct TimeParameters {
    pub start: DateTime,
    pub speed: f64,
    pub total_distance: f64,
}

impl TimeParameters {
    pub fn duration_seconds(&self) -> f64 {
        self.total_distance / self.speed
    }
}

pub struct OuterArc {
    pub start_angle: f64,
    pub end_angle: f64,
    pub label: String,
}

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
    pub outer_arcs: Vec<OuterArc>,
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
    pub fn new(time_parameters: &TimeParameters) -> Self {
        Self {
            control_points: Vec::new(),
            mid_points: Vec::new(),
            has_start_control: false,
            has_end_control: false,
            time_points: time_points::generate(time_parameters),
            outer_arcs: Vec::new(),
        }
    }
    pub fn add(&mut self, segment: &SegmentData, kinds: HashSet<InputType>) {
        if kinds.contains(&InputType::Control) {
            let controls = get_control_points(segment);
            (self.has_start_control, self.has_end_control) =
                controls::has_startend_controls(&segment.track, &controls);
            for c in &controls {
                for a in angles(&c, &segment.track) {
                    let cp = CirclePoint {
                        angle: a,
                        name: control_name(&c),
                    };
                    self.control_points.push(cp)
                }
            }
            self.control_points.sort_by_key(|p| p.angle.floor() as i32);
        }
        if kinds.contains(&InputType::UserStep) {
            for c in get_mid_points(segment) {
                for a in angles(&c, &segment.track) {
                    let cp = CirclePoint {
                        angle: a,
                        name: c.name(),
                    };
                    self.mid_points.push(cp);
                }
            }
            self.mid_points.sort_by_key(|p| p.angle.floor() as i32);
        }
    }
}
