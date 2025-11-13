use crate::{
    backend::Segment,
    inputpoint::{InputPoint, InputType, OSM},
    track::Track,
};

fn is_close_to_track(w: &InputPoint) -> bool {
    let d = w.track_projection.as_ref().unwrap().track_distance;
    match w.kind() {
        InputType::OSM { kind } => {
            let pop = w.population().unwrap_or(0);
            if kind == OSM::City || pop > 1000 {
                return d < 2000.0;
            }
        }
        _ => {}
    }
    return d < 300.0;
}

fn profile_points_elevation_gain_track(track: &Track, d: &f64) -> Vec<InputPoint> {
    let mut ret = Vec::new();
    let mut prev = 0;
    let mut index = prev + 1;
    let length = track.len();
    let mut count = 1;
    loop {
        if index >= length {
            break;
        }
        let g = track.elevation_gain_on_range(&std::ops::Range {
            start: prev,
            end: index,
        });
        if g >= *d {
            let w = InputPoint::create_point_on_track(
                &track,
                index,
                &format!("P{}", count),
                InputType::UserStep,
            );
            ret.push(w);
            count += 1;
            prev = index;
        }
        index += 1;
    }
    ret
}

fn profile_points_elevation_gain(segment: &Segment, d: &f64) -> Vec<InputPoint> {
    let mut ret = profile_points_elevation_gain_track(&segment.track, &d);
    ret.retain(|w| {
        segment
            .range
            .contains(&w.track_projection.as_ref().unwrap().track_index)
    });
    ret
}

fn profile_points_distance_track(track: &Track, d: &f64) -> Vec<InputPoint> {
    let mut ret = Vec::new();
    let mut prev = 0;
    let mut index = prev + 1;
    let mut count = 1;
    let length = track.len();
    loop {
        if index >= length {
            break;
        }
        if track.distance(index) - track.distance(prev) >= *d {
            let w = InputPoint::create_point_on_track(
                &track,
                index,
                &format!("P{}", count),
                InputType::UserStep,
            );
            ret.push(w);
            count += 1;
            prev = index;
        }
        index += 1;
    }
    ret
}

fn profile_points_distance(segment: &Segment, d: &f64) -> Vec<InputPoint> {
    let mut ret = profile_points_distance_track(&segment.track, d);
    ret.retain(|w| {
        segment
            .range
            .contains(&w.track_projection.as_ref().unwrap().track_index)
    });
    ret
}

fn user_points(segment: &Segment) -> Vec<InputPoint> {
    let mut ret = Vec::new();
    match segment.parameters.profile_options.step_distance {
        None => {}
        Some(d) => {
            ret.extend_from_slice(&profile_points_distance(segment, &d));
        }
    };

    match segment.parameters.profile_options.step_elevation_gain {
        None => {}
        Some(d) => {
            ret.extend_from_slice(&profile_points_elevation_gain(segment, &d));
        }
    }
    ret
}

pub fn profile_points(segment: &Segment) -> Vec<InputPoint> {
    let mut ret = segment.points.clone();
    ret.retain(|w| is_close_to_track(&w));
    ret.extend_from_slice(&user_points(segment));
    ret
}

pub fn map_points(segment: &Segment) -> Vec<InputPoint> {
    let ret = segment.profile_points();
    ret
}
