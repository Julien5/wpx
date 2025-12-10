use crate::{
    backend::Segment,
    inputpoint::{InputPoint, InputType, OSMType},
    label_placement::prioritize,
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

use std::collections::{BTreeMap, HashSet};

struct SectorClassifier {
    map: BTreeMap<usize, Vec<InputPoint>>,
}

impl SectorClassifier {
    fn make(segment: &Segment, n: &usize) -> SectorClassifier {
        let mut map = BTreeMap::new();
        let sector_size = 360.0 / *n as f64;
        for sector in 0..*n {
            map.insert(sector, Vec::new());
        }
        let packets = prioritize::profile(segment);
        for packet in packets {
            for point in packet {
                if point.kind() == InputType::UserStep {
                    continue;
                }
                if point.name().is_none() {
                    continue;
                }
                let a = angle(&point, &segment.track);
                assert!(0.0 <= a && a < 360.0);
                let index = (a / sector_size).floor() as usize;
                map.get_mut(&index).unwrap().push(point.clone());
            }
        }
        for (sector, points) in &mut map {
            if points.is_empty() {
                log::trace!("no point in sector {}", sector);
                continue;
            }
            points.sort_by_key(|w| -Self::control_point_goodness(w));
        }
        SectorClassifier { map }
    }

    fn control_point_goodness(point: &InputPoint) -> i32 {
        match point.kind() {
            InputType::UserStep => {
                return i32::MIN;
            }
            InputType::GPX => {
                return i32::MAX;
            }
            InputType::OSM => {
                let min_population = match point.osmkind().unwrap() {
                    OSMType::City => 10000,
                    OSMType::Village => 1000,
                    OSMType::Hamlet => 100,
                    _ => 0,
                };
                let population = point.population().unwrap_or(min_population);
                if population > 0 {
                    return population;
                }
                return 0;
            }
        };
    }

    fn result(&mut self) -> Vec<InputPoint> {
        let mut ret = Vec::new();
        for (sector, points) in &mut self.map {
            if points.is_empty() {
                log::trace!("sector:{} => nothing found", sector,);
                continue;
            }
            let selected = points.first().unwrap().clone();
            log::trace!(
                "sector:{} => {}",
                sector,
                selected.name().unwrap_or("noname".to_string())
            );
            ret.push(selected);
        }
        ret
    }
}

fn get_control_points(segment: &Segment, n: usize) -> Vec<InputPoint> {
    match segment.points.get(&InputType::GPX) {
        Some(points) => {
            if !points.is_empty() {
                return points.clone();
            }
        }
        None => {}
    }
    let mut classifier = SectorClassifier::make(&segment, &n);
    let ret = classifier.result();
    assert!(ret.len() <= n);
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
    pub fn make(segment: &Segment, kinds: HashSet<InputType>) -> WheelModel {
        let mut control_points = Vec::new();
        if kinds.contains(&InputType::GPX) {
            let track_distance_km = segment.track.total_distance() / 1000f64;
            let n_controls = ((track_distance_km / 70f64).ceil() as usize).max(4);
            for c in get_control_points(segment, n_controls) {
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
