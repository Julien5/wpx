#![allow(non_snake_case)]

use crate::gpsdata;
use crate::parameters;
use crate::project;
use crate::waypoint;
use crate::waypoint::Waypoint;
use crate::waypoint::WaypointOrigin;

type Waypoints = Vec<Waypoint>;
type Parameters = parameters::Parameters;

fn sort(ret: &mut Waypoints, track: &gpsdata::Track) {
    // TODO: avoid re-computation of the tree
    let indexes = project::nearest_neighboor(&track.utm, &ret);
    debug_assert_eq!(ret.len(), indexes.len());
    for k in 0..indexes.len() {
        assert!(indexes[k] < track.len());
        match ret[k].track_index {
            // the waypoint would be projected to another point of the track.
            // do not do this.
            Some(index) => {
                if index != indexes[k] {
                    println!("before: {} => distance={}", index, track.distance(index));
                    println!(
                        "after: {} => distance={}",
                        indexes[k],
                        track.distance(indexes[k])
                    )
                }
                // the two indices can be different if the track has several overlapping segments
                // debug_assert!(index == indexes[k]);
            }
            None => {
                ret[k].track_index = Some(indexes[k]);
            }
        }
    }
    for w in &mut *ret {
        assert!(w.get_track_index() < track.len());
    }
    // .. and sort them
    ret.sort_by(|w1, w2| w1.track_index.cmp(&w2.track_index));
    for k in 1..ret.len() {
        let k1 = ret[k].track_index;
        let k0 = ret[k - 1].track_index;
        debug_assert!(k1 >= k0);
    }
    for w in ret {
        debug_assert!(w.get_track_index() < track.len());
    }
}

fn douglas(track: &gpsdata::Track, params: &Parameters) -> Waypoints {
    let mut ret = Vec::new();
    let indexes = track.interesting_indexes(params.epsilon);
    for idx in indexes {
        let wgs = track.wgs84[idx].clone();
        let utm = track.utm[idx].clone();
        let w = waypoint::Waypoint::create(wgs, utm, idx, WaypointOrigin::DouglasPeucker);
        ret.push(w);
    }
    ret
}

fn gpxwaypoints(waypoints: &Waypoints) -> Waypoints {
    let mut ret = Vec::new();
    // take GPX waypoints
    for w in waypoints {
        match w.origin {
            WaypointOrigin::GPX => ret.push(w.clone()),
            _ => {}
        }
    }
    ret
}

fn find_index_with_distance(track: &gpsdata::Track, dist: f64, from: usize) -> usize {
    for k in from..track.len() {
        if track.distance(k) > dist {
            return k;
        }
    }
    track.len()
}

fn max_step_size_subsample(
    track: &gpsdata::Track,
    k0: usize,
    k1: usize,
    params: &Parameters,
) -> Waypoints {
    let mut ret = Vec::new();
    debug_assert!(k1 > k0 && k1 < track.len());
    let d0 = track.distance(k0);
    let d = track.distance(k1) - d0;
    let dmax = params.max_step_size;
    let nstep = (d / dmax).ceil();
    let dstep = d / nstep;
    let mut kn = k0;
    // n takes the values 1, 2, ..., nstep-1.
    for n in 1..(nstep as usize) {
        let dnstep = d0 + (n as f64) * dstep;
        kn = find_index_with_distance(track, dnstep, kn);
        /* make waypoint at index kn */
        let w = Waypoint::create(
            track.wgs84[kn].clone(),
            track.utm[kn].clone(),
            kn,
            waypoint::WaypointOrigin::MaxStepSize,
        );
        ret.push(w);
    }
    ret
}

fn find_next_index(track: &gpsdata::Track, waypoints: &Waypoints, start: usize) -> Option<usize> {
    if start >= (track.len() - 1) {
        return None;
    }
    for w in waypoints {
        if w.get_track_index() > start {
            return w.track_index;
        }
    }
    Some(track.len() - 1)
}

fn max_step_size(track: &gpsdata::Track, waypoints: &Waypoints, params: &Parameters) -> Waypoints {
    debug_assert!(!track.wgs84.is_empty());

    let mut k0 = 0;
    let mut k1 = match waypoints.is_empty() {
        true => track.len() - 1,
        false => waypoints[0].get_track_index(),
    };
    let mut ret = Waypoints::new();
    loop {
        let d = track.distance(k1) - track.distance(k0);
        if d > params.max_step_size {
            ret.extend(max_step_size_subsample(track, k0, k1, params));
        }
        k0 = k1;
        match find_next_index(&track, &waypoints, k1) {
            Some(index) => {
                k1 = index;
            }
            _ => {
                break;
            }
        }
    }
    ret
}

fn waypoints_within_distance(
    track: &gpsdata::Track,
    W: &Waypoints,
    k: usize,
    dmax: f64,
) -> Vec<usize> {
    let distance_to = |w: &waypoint::Waypoint| -> f64 { track.distance(w.get_track_index()) };
    let distance_between = |w1: &waypoint::Waypoint, w2: &waypoint::Waypoint| -> f64 {
        (distance_to(w2) - distance_to(w1)).abs()
    };
    let mut ret = Vec::new();
    for d in 1..(k + 1) {
        let candidate = k - d;
        let delta = distance_between(&W[k], &W[candidate]);
        if delta < dmax {
            ret.push(candidate);
        }
    }
    for candidate in (k + 1)..W.len() {
        let delta = distance_between(&W[k], &W[candidate]);
        if delta < dmax {
            ret.push(candidate);
        }
    }
    ret
}

fn remove_near_waypoints(track: &gpsdata::Track, W: &mut Waypoints) -> Waypoints {
    let mut hide = std::collections::BTreeSet::new();
    for k in 0..W.len() {
        // hide around gpx waypoints only
        if W[k].origin != WaypointOrigin::GPX {
            continue;
        }
        let neighbors = waypoints_within_distance(track, W, k, 2000f64);
        for l in neighbors {
            if W[l].origin != WaypointOrigin::GPX {
                hide.insert(l);
            }
        }
    }
    let mut ret = Vec::new();
    for k in 0..W.len() {
        if !hide.contains(&k) {
            ret.push(W[k].clone());
        }
    }
    ret
}

pub fn generate(
    track: &gpsdata::Track,
    waypoints: &Waypoints,
    params: &Parameters,
) -> Vec<Waypoint> {
    let mut ret = Vec::new();
    ret.extend(gpxwaypoints(waypoints));
    ret.extend(douglas(track, params));
    sort(&mut ret, &track);
    ret = remove_near_waypoints(&track, &mut ret);
    for w in &ret {
        debug_assert!(w.get_track_index() < track.len());
    }
    ret.extend(max_step_size(track, &ret, params));
    sort(&mut ret, &track);
    ret
}
