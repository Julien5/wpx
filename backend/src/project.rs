#![allow(non_snake_case)]

use crate::gpsdata::Track;
use crate::utm;
use crate::waypoint;
use crate::waypoint::Waypoints;

// see https://docs.rs/kd-tree/latest/kd_tree/trait.KdPoint.html
impl kd_tree::KdPoint for utm::UTMPoint {
    type Scalar = f64;
    type Dim = typenum::U2;
    fn at(&self, i: usize) -> Self::Scalar {
        match i {
            0 => self.x(),
            1 => self.y(),
            _ => 0 as Self::Scalar,
        }
    }
    fn dim() -> usize {
        2usize
    }
}

pub fn nearest_neighboor(
    track: &Vec<utm::UTMPoint>,
    waypoints: &Vec<waypoint::Waypoint>,
) -> Vec<usize> {
    let tree = kd_tree::KdIndexTree::build_by_ordered_float(track);
    let mut ret = Vec::new();
    for point in waypoints {
        let N = tree.nearest(&point.utm);
        ret.push(*N.unwrap().item);
    }
    ret
}

pub fn project_on_track(track: &Track, waypoints: &mut Waypoints) {
    let indexes = nearest_neighboor(&track.utm, &waypoints);
    debug_assert_eq!(waypoints.len(), indexes.len());
    for k in 0..indexes.len() {
        waypoints[k].track_index = Some(indexes[k]);
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn kdtree() {
        let items = vec![[10, 20, 30], [30, 10, 20], [20, 30, 10]];
        let kdtree = kd_tree::KdIndexTree::build(&items);
        assert_eq!(kdtree.nearest(&[30, 10, 20]).unwrap().item, &1);
        assert_eq!(kdtree.nearest(&[29, 9, 20]).unwrap().item, &1);
    }
}
