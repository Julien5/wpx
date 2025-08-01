#![allow(non_snake_case)]

use crate::gpsdata;
use crate::waypoint;

pub fn shows_waypoint(w: &waypoint::Waypoint, bbox: &gpsdata::ProfileBoundingBox) -> bool {
    let distance = w.info.as_ref().unwrap().distance;
    bbox.xmin <= distance && distance <= bbox.xmax
}

pub fn show_waypoints_in_table(
    waypoints: &Vec<waypoint::Waypoint>,
    bbox: &gpsdata::ProfileBoundingBox,
) -> Vec<usize> {
    // the waypoints indices visible in this profile..
    let mut indices: Vec<usize> = (0..waypoints.len())
        .collect::<Vec<usize>>()
        .into_iter()
        .filter(|k| shows_waypoint(&waypoints[*k], bbox))
        .collect();
    // sorted by value
    indices.sort_by_key(|k| waypoints[*k].info.as_ref().unwrap().value.unwrap());
    indices.truncate(15);
    indices.sort();
    indices
}
