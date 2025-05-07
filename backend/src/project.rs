#![allow(non_snake_case)]
use crate::gpsdata;

// see https://docs.rs/kd-tree/latest/kd_tree/trait.KdPoint.html
impl kd_tree::KdPoint for gpsdata::UTMPoint {
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
    track: &Vec<gpsdata::UTMPoint>,
    waypoints: &Vec<gpsdata::Waypoint>,
) -> Vec<usize> {
    let tree = kd_tree::KdIndexTree::build_by_ordered_float(track);
    let mut ret = Vec::new();
    for point in waypoints {
        let N = tree.nearest(&point.utm);
        ret.push(*N.unwrap().item);
    }
    ret
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn kdtree() {
        let items = vec![[10, 20, 30], [30, 10, 20], [20, 30, 10]];
        let kdtree = kd_tree::KdIndexTree::build(&items);
        assert_eq!(kdtree.nearest(&[30, 10, 20]).unwrap().item, &1);
        assert_eq!(kdtree.nearest(&[29, 9, 20]).unwrap().item, &1);
    }
}
