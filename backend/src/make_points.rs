use crate::{inputpoint::InputPoint, parameters::UserStepsOptions, track::Track};

fn profile_points_elevation_gain_track(track: &Track, d: &f64) -> Vec<InputPoint> {
    let mut ret: Vec<InputPoint> = Vec::new();
    let len = track.len();
    let max_elevation = track.profile.elevation_gain(len - 1);
    loop {
        let start_search = match ret.last() {
            Some(w) => w.track_projections.first().unwrap().track_index,
            None => 0,
        };
        let di = (ret.len() + 1) as f64 * d;
        if di > max_elevation {
            break;
        }
        let (wgs, proj) = track.point_at_elevation_gain(di, start_search);
        let w = InputPoint::create_user_step_on_track(&wgs, proj);
        ret.push(w);
    }
    ret
}

fn profile_points_distance_track(track: &Track, d: &f64) -> Vec<InputPoint> {
    let mut ret: Vec<InputPoint> = Vec::new();
    loop {
        let start_search = match ret.last() {
            Some(w) => w.track_projections.first().unwrap().track_index,
            None => 0,
        };
        let di = (ret.len() + 1) as f64 * d;
        if di > track.total_distance() {
            break;
        }
        let (wgs, proj) = track.point_at_distance(di, start_search);
        let w = InputPoint::create_user_step_on_track(&wgs, proj);
        ret.push(w);
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
