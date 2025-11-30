use crate::{
    backend::Segment,
    inputpoint::{InputPoint, InputType},
    label_placement::prioritize,
    track::Track,
};

pub struct ControlPoint {
    pub angle: f64,
    pub name: String,
}

pub struct MidPoint {
    pub angle: f64,
    pub name: String,
}

pub struct WheelModel {
    pub control_points: Vec<ControlPoint>,
    pub mid_points: Vec<MidPoint>,
}

fn angle(point: &InputPoint, track: &Track) -> f64 {
    let proj = point.track_projection.as_ref().unwrap();
    let index = proj.track_index;
    let part = track.distance(index);
    let total = track.total_distance();
    log::debug!("part:{:.1} total:{:.1}", part, total);
    360.0 * part / total
}

fn name(point: &InputPoint) -> String {
    match point.name() {
        Some(text) => text,
        None => "noname".to_string(),
    }
}

fn get_control_points(segment: &Segment, maxlen: usize) -> Vec<InputPoint> {
    match segment.points.get(&InputType::GPX) {
        Some(points) => {
            if !points.is_empty() {
                return points.clone();
            }
        }
        None => {}
    }
    let mut ret = Vec::new();
    let packets = prioritize::profile(segment);
    for packet in packets {
        for point in packet {
            ret.push(point.clone());
            if ret.len() >= maxlen {
                return ret;
            }
        }
    }
    ret
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
    pub fn make(segment: &Segment) -> WheelModel {
        let mut control_points = Vec::new();
        for c in get_control_points(segment, 10) {
            let cp = ControlPoint {
                angle: angle(&c, &segment.track),
                name: name(&c),
            };
            log::debug!("control:{} at {}", cp.name, cp.angle);
            control_points.push(cp);
        }
        let mut mid_points = Vec::new();
        for c in get_mid_points(segment) {
            let cp = MidPoint {
                angle: angle(&c, &segment.track),
                name: name(&c),
            };
            log::debug!("mid:{} at {}", cp.name, cp.angle);
            mid_points.push(cp);
        }
        log::debug!("controls:{}", control_points.len());
        log::debug!("mids:{}", mid_points.len());
        WheelModel {
            control_points,
            mid_points,
        }
    }
}
