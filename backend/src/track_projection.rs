#[allow(dead_code)]
use std::{cmp::Ordering, collections::BTreeSet};

use crate::{
    inputpoint::{InputPoint, InputPointMap, InputType, OSMType},
    locate,
    mercator::MercatorPoint,
    track::Track,
};

use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct TrackProjection {
    pub track_floating_index: f64,
    pub track_index: usize,
    pub euclidean: MercatorPoint,
    pub elevation: f64,
    pub track_distance: f64,
    pub distance_on_track_to_projection: f64,
}

pub type TrackProjections = BTreeSet<TrackProjection>;

impl PartialEq for TrackProjection {
    fn eq(&self, other: &Self) -> bool {
        self.track_index.cmp(&other.track_index).is_eq()
    }
}

impl Eq for TrackProjection {}

impl PartialOrd for TrackProjection {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for TrackProjection {
    fn cmp(&self, other: &Self) -> Ordering {
        self.track_floating_index
            .total_cmp(&other.track_floating_index)
    }
}

pub fn is_close_to_track(w: &InputPoint) -> bool {
    if w.track_projections.is_empty() {
        return false;
    }
    let d = w.track_projections.first().unwrap().track_distance;
    match w.kind() {
        InputType::OSM => {
            let kind = w.osmkind().unwrap();
            let pop = w.population().unwrap_or(0);
            if kind == OSMType::City || pop > 1000 {
                return d < 2000.0;
            }
        }
        _ => {}
    }
    return d < 300.0;
}

fn dmax(kind: &InputType, osmkind: &Option<OSMType>, population: &Option<i32>) -> f64 {
    match kind {
        InputType::OSM => {
            let okind = osmkind.as_ref().unwrap();
            let pop = population.unwrap_or(0);
            if *okind == OSMType::City || pop > 1000 {
                return 2000.0;
            }
        }
        _ => {}
    }
    return 300.0;
}

pub fn update_track_projection(
    point: &mut InputPoint,
    track: &Track,
    tracktree: &locate::IndexedPointsTree,
) {
    let new_projection = locate::compute_track_projection(track, tracktree, point);
    if point.track_projections.is_empty() {
        point.track_projections.insert(new_projection);
        return;
    }

    let dmax = dmax(&point.kind(), &point.osmkind(), &point.population());
    let d = new_projection.track_distance;
    if d > dmax {
        return;
    }

    let known = point.track_projections.iter().any(|proj| {
        let d1 = proj.distance_on_track_to_projection;
        let d2 = new_projection.distance_on_track_to_projection;
        (d1 - d2).abs() < 10f64 * dmax
    });

    if !known {
        point.track_projections.insert(new_projection);
    }
}

fn find_global_max<F>(start: usize, end: usize, f: F) -> usize
where
    F: Fn(&usize) -> f64,
{
    let mut best_idx = start;
    // Handle empty range case
    if start >= end {
        return start;
    }

    let mut max_val = f(&start);

    for i in (start + 1)..end {
        let current_val = f(&i);
        // Using partial_cmp to safely handle f64 (NaNs)
        if current_val > max_val {
            max_val = current_val;
            best_idx = i;
        }
    }
    best_idx
}

pub struct ProjectionTrees {
    total_tree: locate::IndexedPointsTree,
    trees: Vec<locate::IndexedPointsTree>,
}

impl ProjectionTrees {
    fn find_appropriate_projection_ranges(track: &Track) -> Vec<std::ops::Range<usize>> {
        let start = 0;
        let end = track.wgs84.len();
        let start_point = track.euclidian.first().unwrap();
        let f = |index: &usize| -> f64 { start_point.d2(&track.euclidian[*index]) };
        let extremity = find_global_max(start, end, f);
        vec![0..extremity, extremity..end]
    }

    fn make_appropriate_projection_trees(track: &Track) -> Vec<locate::IndexedPointsTree> {
        Self::find_appropriate_projection_ranges(track)
            .iter()
            .map(|range| locate::IndexedPointsTree::from_track(track, range))
            .collect()
    }

    pub fn make(track: &Track) -> Self {
        let range = 0..track.len();
        let total = locate::IndexedPointsTree::from_track(track, &range);
        Self {
            total_tree: total,
            trees: Self::make_appropriate_projection_trees(track),
        }
    }

    pub fn iter_on(&self, map: &mut InputPointMap, track: &Track) {
        let boxes = &track.boxes;
        for b in boxes {
            if map.get_mut(b).is_none() {
                continue;
            }
            let points = map.get_mut(b).unwrap();
            for mut point in points {
                update_track_projection(&mut point, track, &self.total_tree);
                let index = point.track_projections.first().unwrap().track_index;
                if is_close_to_track(&point) {
                    for tree in &self.trees {
                        if !tree.range.contains(&index) {
                            update_track_projection(&mut point, track, tree);
                        }
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{bboxes, gpsdata::GpxData, osm, track::Track, wgs84point::WGS84Point};

    fn read(filename: String) -> GpxData {
        use crate::gpsdata;
        let mut f = std::fs::File::open(filename).unwrap();
        let mut content = Vec::new();
        // read the whole file
        use std::io::prelude::*;
        f.read_to_end(&mut content).unwrap();
        gpsdata::read_content(&content).unwrap()
    }

    #[tokio::test]
    async fn projection() {
        let _ = env_logger::try_init();
        use crate::track_projection::*;
        //let gpxdata = read("data/ref/pbp2023.gpx".to_string());
        let gpxdata = read("data/ref/pbp2019.gpx".to_string());
        let mut tags = std::collections::BTreeMap::new();
        tags.insert("wpxtype".to_string(), "OSM".to_string());
        tags.insert("name".to_string(), "Mortagne-au-Perche".to_string());
        tags.insert("place".to_string(), "town".to_string());
        tags.insert("population".to_string(), "3815".to_string());
        let pos = MercatorPoint::new(&61237.909420542324, &6193890.266343569);
        let mortagne = InputPoint {
            wgs84: WGS84Point::new(&0.5501095, &48.5205106, &0.0),
            euclidean: pos.clone(),
            tags: tags,
            track_projections: TrackProjections::new(),
        };
        let track = Track::from_tracks(&gpxdata.tracks).unwrap();
        let trees = ProjectionTrees::make(&track);
        let mut map = InputPointMap::new();
        map.insert_point(&bboxes::pointbox(&pos), &mortagne);
        trees.iter_on(&mut map, &track);
        map.iter().for_each(|p| {
            assert_eq!(p.track_projections.len(), 2);
            log::info!("p={:?}", p);
        });
    }
}
