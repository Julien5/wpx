use crate::{
    backend::Segment,
    inputpoint::{InputPoint, InputType},
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
    log::debug!("part:{:.1} total:{:.1}", part, total);
    assert!(part <= total);
    360.0 * part / total
}

fn name(point: &InputPoint) -> String {
    match point.name() {
        Some(text) => text,
        None => "noname".to_string(),
    }
}

use std::collections::HashMap;

fn decimate_points(points: &mut Vec<CirclePoint>, n: usize) {
    if n == 0 {
        // Cannot divide the circle into 0 quadrants
        points.clear();
        return;
    }

    let sector_size = 360.0 / n as f64;

    // Map to store the *most important* (first encountered) point for each sector index.
    // Key: Sector Index (0 to n-1)
    // Value: The full CirclePoint struct that was chosen
    let mut chosen_points: HashMap<usize, CirclePoint> = HashMap::new();

    // Iterate through the points in the provided order (order of importance)
    for point in points.iter() {
        // 1. Normalize the angle to be in the [0, 360) range.
        let mut angle_normalized = point.angle % 360.0;
        assert!(angle_normalized >= 0.0);
        // Handle the 360-degree case: if angle is exactly 360, treat it as 0
        if angle_normalized == 360.0 {
            angle_normalized = 0.0;
        }

        // 2. Calculate the sector index (k)
        // k = floor(angle_normalized / sector_size)
        let sector_index = (angle_normalized / sector_size).floor() as usize;

        // Ensure the index is within [0, n-1] bounds
        let final_sector_index = sector_index.min(n - 1);

        // 3. Keep only the *first* point for this sector.
        // The entry().or_insert_with() pattern is perfect:
        // it only inserts if the key (sector_index) is not already present.
        // Since we are iterating in order of importance, the first one seen is the one to keep.
        chosen_points
            .entry(final_sector_index)
            .or_insert_with(|| point.clone());
    }

    // 4. Clear the original vector and populate it with the chosen points.
    // Since the order of the output vector is not specified, we can collect the chosen points
    // from the HashMap's values. If a specific output order is required (e.g., by angle),
    // an additional step would be needed here.
    let mut result_vec: Vec<CirclePoint> = chosen_points.into_values().collect();

    // Replace the content of the original vector with the result
    points.clear();
    points.append(&mut result_vec);
}

fn get_control_points(segment: &Segment) -> Vec<InputPoint> {
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
            if point.kind() == InputType::UserStep {
                continue;
            }
            if point.name().is_none() {
                continue;
            }
            ret.push(point.clone());
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
        for c in get_control_points(segment) {
            let cp = CirclePoint {
                angle: angle(&c, &segment.track),
                name: name(&c),
            };
            control_points.push(cp);
        }
        decimate_points(&mut control_points, 10);
        control_points.sort_by_key(|p| p.angle.floor() as i32);
        for p in &control_points {
            log::debug!("control:{} at {:.1}", p.name, p.angle);
        }
        let mut mid_points = Vec::new();
        for c in get_mid_points(segment) {
            let cp = CirclePoint {
                angle: angle(&c, &segment.track),
                name: name(&c),
            };
            mid_points.push(cp);
        }
        mid_points.sort_by_key(|p| p.angle.floor() as i32);
        for p in &mid_points {
            log::debug!("mid:{} at {:.1}", p.name, p.angle);
        }
        log::debug!("controls:{}", control_points.len());
        log::debug!("mids:{}", mid_points.len());
        WheelModel {
            control_points,
            mid_points,
        }
    }
}
