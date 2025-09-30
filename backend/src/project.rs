#![allow(non_snake_case)]

use crate::inputpoint::InputPoint;
use crate::track::Track;
use sphere_knn::SphereKnnGetters;

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
    subset: &Vec<usize>,
) -> std::collections::BTreeMap<usize, usize> {
    log::trace!("build tree");
    let find_nearest = sphere_knn::run(convert(&track));
    log::trace!("project {} points", subset.len());
    let mut ret = std::collections::BTreeMap::new();
    for k in subset {
        let point = &waypoints[*k];
        let result = find_nearest(
            point.latitude(),
            point.longitude(),
            sphere_knn::Opts {
                max_distance_threshold_meters: Some(500f64),
                number_results: Some(1 as usize),
            },
        );
        match result.first() {
            Some(res) => {
                ret.insert(*k, res.index);
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
    fn set_track_index(&self, index: usize);
    fn get_track_index(&self) -> Option<usize>;
}

impl Projectable for InputPoint {
    fn latitude(&self) -> f64 {
        self.wgs84.latitude()
    }
    fn longitude(&self) -> f64 {
        self.wgs84.longitude()
    }
    fn set_track_index(&self, index: usize) {
        self.track_index.set(Some(index));
    }
    fn get_track_index(&self) -> Option<usize> {
        self.track_index.get()
    }
}

pub fn project_on_track<T: Projectable>(track: &Track, waypoints: &Vec<T>, _subset: &Vec<usize>) {
    let mut subset = _subset.clone();
    subset.retain(|k| waypoints[*k].get_track_index().is_none());
    let indexmap = nearest_neighboor(&track, &waypoints, &subset);
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
