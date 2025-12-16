use crate::{
    inputpoint::{InputPoint, InputType, OSMType},
    parameters::UserStepsOptions,
    track::Track,
};

pub fn is_close_to_track(w: &InputPoint) -> bool {
    let d = w.track_projection.as_ref().unwrap().track_distance;
    match w.kind() {
        InputType::OSM => {
            let kind = w.osmkind().unwrap();
            let pop = w.population().unwrap_or(0);
            if kind == OSMType::City || pop > 1000 {
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
            let w = InputPoint::create_user_step_on_track(&track, index, &format!("P{}", count));
            ret.push(w);
            count += 1;
            prev = index;
        }
        index += 1;
    }
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
            let w = InputPoint::create_user_step_on_track(&track, index, &format!("P{}", count));
            ret.push(w);
            count += 1;
            prev = index;
        }
        index += 1;
    }
    ret
}

pub fn user_points(track: &Track, options: &UserStepsOptions) -> Vec<InputPoint> {
    let mut ret = Vec::new();
    match options.step_distance {
        None => {}
        Some(d) => {
            ret.extend_from_slice(&profile_points_distance_track(track, &d));
        }
    };

    match options.step_elevation_gain {
        None => {}
        Some(d) => {
            let loc = profile_points_elevation_gain_track(track, &d);
            for p in &loc {
                let d = p.track_projection.as_ref().unwrap().track_distance;
                assert_eq!(d, 0f64);
            }
            ret.extend_from_slice(&loc);
        }
    }
    ret
}
