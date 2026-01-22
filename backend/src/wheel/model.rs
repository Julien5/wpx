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

pub struct Arc {
    pub start_angle: f64,
    pub middle_angle: Option<f64>,
    pub end_angle: f64,
    pub label: String,
}

impl Arc {
    fn distances(segments: &[Segment]) -> Vec<f64> {
        if segments.is_empty() {
            return vec![];
        }
        let mut ret = Vec::new();
        for (index, segment) in segments.iter().enumerate() {
            ret.push(segment.start);
            if index == (segments.len() - 1) {
                ret.push(segment.end);
            }
        }
        ret
    }

    fn clamp_angle(x: f64, total: f64) -> f64 {
        angle(x.min(total), total)
    }

    pub fn from_segments(segments: &[Segment], time_parameters: &TimeParameters) -> Vec<Self> {
        let distances = Self::distances(segments);
        let mut ret = Vec::new();
        let end_distance = time_parameters.total_distance;
        for i in 0..distances.len() - 1 {
            if i % 2 == 0 {
                let begin = distances[i];
                let (middle, end) = if (i + 2) < distances.len() {
                    (Some(distances[i + 1]), distances[i + 2])
                } else {
                    (None, distances[i + 1])
                };
                let a = Arc {
                    start_angle: Self::clamp_angle(begin, end_distance),
                    end_angle: Self::clamp_angle(end, end_distance),
                    middle_angle: if middle.is_some() {
                        Some(Self::clamp_angle(middle.unwrap(), end_distance))
                    } else {
                        None
                    },
                    label: String::new(),
                };
                ret.push(a);
            }
        }
        ret
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
    pub outer_arcs: Vec<Arc>,
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
    pub fn add_pages(&mut self, segments: &Vec<Segment>) {
        self.outer_arcs = Arc::from_segments(segments, &self.time_parameters);
    }
}
