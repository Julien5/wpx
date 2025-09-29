#![allow(non_snake_case)]

use crate::inputpoint::InputPoint;
use crate::track::Track;
use crate::waypoint::Waypoint;
use crate::wgs84point::WGS84Point;
use sphere_knn::SphereKnnGetters;

#[derive(Clone)]
struct IndexedWGS84Point {
    pub wgs84: WGS84Point,
    pub index: usize,
}

impl std::fmt::Debug for IndexedWGS84Point {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("IndexWGS84Point")
            .field("lon", &self.wgs84.longitude())
            .field("lat", &self.wgs84.latitude())
            .field("index", &self.index)
            .finish()
    }
}

impl SphereKnnGetters for IndexedWGS84Point {
    fn get_lat(&self) -> f64 {
        self.wgs84.latitude()
    }
    fn get_lng(&self) -> f64 {
        self.wgs84.longitude()
    }
}

fn convert(track: &Track) -> Vec<IndexedWGS84Point> {
    let mut ret = Vec::new();
    for k in 0..track.wgs84.len() {
        ret.push(IndexedWGS84Point {
            wgs84: track.wgs84[k].clone(),
            index: k,
        });
    }
    ret
}

pub fn nearest_neighboor<T: Projectable>(
    track: &Track,
    waypoints: &Vec<T>,
) -> std::collections::BTreeMap<usize, usize> {
    log::trace!("build tree");
    let find_nearest = sphere_knn::run(convert(&track));
    log::trace!("project {} points", waypoints.len());
    let mut ret = std::collections::BTreeMap::new();
    for k in 0..waypoints.len() {
        let point = &waypoints[k];
        let result = find_nearest(
            point.latitude(),
            point.longitude(),
            sphere_knn::Opts {
                max_distance_threshold_meters: Some(20000f64),
                number_results: Some(1 as usize),
            },
        );
        match result.first() {
            Some(res) => {
                ret.insert(k, res.index);
            }
            None => {}
        }
    }
    log::trace!("project done");
    ret
}

pub trait Projectable {
    fn latitude(&self) -> f64;
    fn longitude(&self) -> f64;
    fn set_track_index(&mut self, index: usize);
}

impl Projectable for Waypoint {
    fn latitude(&self) -> f64 {
        self.wgs84.latitude()
    }
    fn longitude(&self) -> f64 {
        self.wgs84.longitude()
    }
    fn set_track_index(&mut self, index: usize) {
        self.track_index = Some(index);
    }
}

impl Projectable for InputPoint {
    fn latitude(&self) -> f64 {
        self.wgs84.latitude()
    }
    fn longitude(&self) -> f64 {
        self.wgs84.longitude()
    }
    fn set_track_index(&mut self, index: usize) {
        self.track_index = Some(index);
    }
}

pub fn project_on_track<T: Projectable>(track: &Track, waypoints: &mut Vec<T>) {
    let indexmap = nearest_neighboor(&track, &waypoints);
    debug_assert!(waypoints.len() >= indexmap.len());
    for (src, dest) in indexmap {
        waypoints[src].set_track_index(dest);
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
