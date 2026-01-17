use std::collections::HashSet;

use crate::{
    backend::Segment,
    controls,
    inputpoint::{InputPoint, InputType},
    mercator::DateTime,
    segment::SegmentData,
    track::Track,
    wheel::time_points,
};

pub fn angle(x: f64, total: f64) -> f64 {
    let topmargin = super::constants::ARCANGLE / 2.0;
    let a = (360.0 - 2.0 * topmargin) / total;
    let b = topmargin;
    assert!(x <= total);
    a * x + b
}

#[derive(Clone)]
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

impl OuterArc {
    pub fn from_segments(
        segments: &[Segment],
        time_parameters: &TimeParameters,
        label: &str,
    ) -> Self {
        assert!(segments.len() <= 2);
        OuterArc {
            start_angle: angle(
                segments.first().unwrap().start,
                time_parameters.total_distance,
            ),
            end_angle: angle(segments.last().unwrap().end, time_parameters.total_distance),
            label: label.to_string(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct CirclePoint {
    pub angle: f64,
    pub name: String,
}

pub struct WheelModel {
    pub time_parameters: TimeParameters,
    pub control_points: Vec<CirclePoint>,
    pub mid_points: Vec<CirclePoint>,
    pub has_start_control: bool,
    pub has_end_control: bool,
    pub time_points: Vec<CirclePoint>,
    pub outer_arcs: Vec<OuterArc>,
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
            time_parameters: time_parameters.clone(),
            control_points: Vec::new(),
            mid_points: Vec::new(),
            has_start_control: false,
            has_end_control: false,
            time_points: time_points::generate(time_parameters),
            outer_arcs: Vec::new(),
        }
    }
    pub fn add_points(&mut self, segment: &SegmentData, kinds: HashSet<InputType>) {
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
    pub fn add_pages(&mut self, segments: Vec<Segment>) {
        self.outer_arcs = segments
            .chunks(2)
            .enumerate()
            .map(|(index, segments)| {
                let label = format!("page {}", index + 1);
                OuterArc::from_segments(segments, &self.time_parameters, &label)
            })
            .collect();
    }
}
