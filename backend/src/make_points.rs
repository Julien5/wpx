use crate::{inputpoint::InputPoint, parameters::UserStepsOptions, track::Track};

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
                let d = p.track_projections.first().unwrap().track_distance;
                assert_eq!(d, 0f64);
            }
            ret.extend_from_slice(&loc);
        }
    }
    ret
}
