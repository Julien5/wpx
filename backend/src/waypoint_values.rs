#![allow(non_snake_case)]
use std::collections::HashMap;

use crate::track;
use crate::waypoint::WaypointOrigin;
use crate::waypoint::Waypoints;

type Values = HashMap<usize, usize>;

pub fn hard_value(waypoints: &Waypoints, _track: &track::Track, index: usize) -> Option<usize> {
    let L = waypoints.len();
    let w = &waypoints[index];
    let mut ret = 0;
    if w.origin == WaypointOrigin::GPX {
        return Some(ret);
    }
    /* To get a better spreading of the labels on the segment
     * ignoring wheter Douglas-Peucker points or not:
     * return None
     */

    ret = ret + 1;
    // DP
    if w.origin == WaypointOrigin::DouglasPeucker {
        if index == 0 || index == (L - 1) {
            return Some(ret);
        }
        let next = &waypoints[index + 1];
        let prev = &waypoints[index - 1];
        // tops
        if prev.elevation() < w.elevation() && next.elevation() < w.elevation() {
            return Some(ret);
        }
        // ret = ret + 1;
        /*
         * treat lows and others as max-step-size points
         * => they might not come in the table
         */
        return None;
        /*
        if prev.elevation() > w.elevation() && next.elevation() > w.elevation() {
            return Some(ret);
        }
        ret = ret + 1;
        return Some(ret);
        */
    }
    ret = ret + 1;
    if 100f64 * w.info.as_ref().unwrap().inter_slope > 3f64 {
        return Some(ret);
    }
    None
}

fn move_index(ret: usize, inc: i32) -> usize {
    let m = (ret as i32) + inc;
    if m < 0 {
        return 0;
    }
    m as usize
}

fn next_with_value(waypoints: &mut Waypoints, values: &Values, start: usize, inc: i32) -> usize {
    let mut ret = move_index(start, inc);
    loop {
        if ret >= waypoints.len() {
            return waypoints.len();
        }
        if ret == 0 {
            return 0;
        }
        if values.contains_key(&ret) {
            return ret;
        }
        ret = move_index(ret, inc);
    }
}

fn find_max_distance(
    waypoints: &mut Waypoints,
    track: &track::Track,
    values: &mut Values,
) -> usize {
    let mut dmax = 0f64;
    let mut kdmax = 0;
    for k in 0..waypoints.len() {
        if values.contains_key(&k) {
            continue;
        }
        let dk = track.distance(waypoints[k].get_track_index());

        let wprev = next_with_value(waypoints, &values, k, -1);
        let wnext = next_with_value(waypoints, &values, k, 1);
        let dprev = (dk - track.distance(waypoints[wprev].get_track_index())).abs();
        let dnext = (dk - track.distance(waypoints[wnext].get_track_index())).abs();

        let d = dprev.min(dnext);

        if d >= dmax {
            dmax = d;
            kdmax = k;
        }
    }
    kdmax
}

fn set_soft_value(
    waypoints: &mut Waypoints,
    track: &track::Track,
    values: &mut Values,
    value: usize,
) {
    let kdmax = find_max_distance(waypoints, track, values);
    debug_assert!(!values.contains_key(&kdmax));
    values.insert(kdmax, value);
}

pub fn compute_values(waypoints: &mut Waypoints, track: &track::Track) {
    let mut values = Values::new();
    let mut max_value = 0;
    let L = waypoints.len();
    for k in 0..L {
        match hard_value(waypoints, track, k) {
            Some(v) => {
                if v > max_value {
                    max_value = v;
                }
                values.insert(k, v);
            }
            None => {}
        }
    }
    let mut value = max_value + 1;
    while values.len() < L {
        set_soft_value(waypoints, &track, &mut values, value);
        value = value + 1;
    }
    for k in 0..L {
        let w = &mut waypoints[k];
        debug_assert!(w.info.is_some());
        // TODO: too much copy for just changing the info.value.
        let mut copy = w.info.as_ref().unwrap().clone();
        copy.value = values.get(&k).copied();
        w.info = Some(copy.clone());
    }
}
