use std::collections::HashMap;

use crate::gpsdata;
use crate::waypoint::Waypoint;
use crate::waypoint::WaypointOrigin;

type Waypoints = Vec<Waypoint>;

fn has_level(waypoint: &Waypoint, level: usize) -> bool {
    match level {
        0 => waypoint.origin == WaypointOrigin::StartEnd,
        1 => waypoint.origin == WaypointOrigin::GPX,
        2 => waypoint.origin == WaypointOrigin::DouglasPeucker,
        3 => waypoint.origin == WaypointOrigin::MaxStepSize,
        _ => {
            debug_assert!(false);
            false
        }
    }
}

fn find_next_index_with_level(waypoints: &Waypoints, start_index: usize, level: usize) -> usize {
    debug_assert!(waypoints.len() >= 2);
    let L = waypoints.len();
    for k in start_index..L {
        if has_level(&waypoints[k], level) {
            return k;
        }
    }
    return L;
}

fn find_max_distance(
    waypoints: &Waypoints,
    track: &gpsdata::Track,
    start: usize,
    end: usize,
    level: usize,
) -> usize {
    let d0 = track.distance(waypoints[start].get_track_index());
    let d1 = track.distance(waypoints[end].get_track_index());
    let dmax = 0f64;
    let mut ret = 0;
    for k in start..end + 1 {
        if !has_level(&waypoints[k], level) {
            continue;
        }
        let dk = track.distance(waypoints[k].get_track_index());
        let dprev = (dk - d0).abs();
        let dnext = (dk - d1).abs();
        let d = dprev.max(dnext);
        if d > dmax {
            ret = k;
        }
    }
    ret
}

fn set_weights_under(
    waypoints: &Waypoints,
    track: &gpsdata::Track,
    level: usize,
    w: usize,
    map: &mut HashMap<usize, usize>,
) {
    let L = waypoints.len();
    let mut start = find_next_index_with_level(waypoints, 0, level);
    loop {
        let end = find_next_index_with_level(waypoints, start + 1, level);
        if end == L {
            break;
        }
        let k = find_max_distance(waypoints, track, start, end, level + 1);
        map.insert(k, w);
        start = end;
    }
}

pub fn weights(waypoints: &mut Waypoints, track: &gpsdata::Track) -> HashMap<usize, usize> {
    let mut ret = HashMap::new();
    waypoints.insert(0, track.create_on_track(0, WaypointOrigin::StartEnd));
    waypoints.push(track.create_on_track(track.wgs84.len() - 1, WaypointOrigin::StartEnd));
    for level in 0..4 {
        set_weights_under(waypoints, track, level, level, &mut ret);
    }
    ret
}
