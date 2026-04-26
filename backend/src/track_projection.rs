#[allow(dead_code)]
use std::{cmp::Ordering, collections::BTreeSet};

use crate::{
    inputpoint::InputPoint,
    locate,
    mercator::MercatorPoint,
    parameters::TrackPart,
    point_collection::{is_osm, Kind},
    track::Track,
    trackparts::parts_to_ranges,
};

use geo::SimplifyIdx;
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

impl TrackProjection {
    pub fn at_track_index(track: &Track, index: usize) -> Self {
        TrackProjection {
            track_floating_index: index as f64,
            track_index: index,
            euclidean: track.euclidean[index].clone(),
            elevation: track.elevation(index),
            track_distance: 0f64,
            distance_on_track_to_projection: track.distance(index),
        }
    }
}

pub type TrackProjections = BTreeSet<TrackProjection>;

#[allow(dead_code)]
pub fn string_projection(projection: &TrackProjection) -> String {
    format!("proj index:{}", projection.track_index)
}

#[allow(dead_code)]
pub fn string_projections(projections: &TrackProjections) -> String {
    let indices: Vec<_> = projections
        .iter()
        .map(|proj| string_projection(&proj))
        .collect();
    format!("[{}]", indices.join(";"))
}

impl PartialEq for TrackProjection {
    fn eq(&self, other: &Self) -> bool {
        self.track_floating_index
            .total_cmp(&other.track_floating_index)
            .is_eq()
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

fn population_estimate(kind: &Kind) -> i32 {
    match kind {
        Kind::Cities => 10000,
        Kind::Villages => 1000,
        Kind::Hamlets => 300,
        _ => 0,
    }
}

pub fn is_close_to_track(w: &InputPoint) -> bool {
    if w.track_projections.is_empty() {
        return false;
    }
    let d = w.track_projections.first().unwrap().track_distance;
    d < dmax(&w.kind(), &w.population())
}

fn dmax(kind: &Kind, population: &Option<i32>) -> f64 {
    if is_osm(kind) && *kind != Kind::Mountains {
        let pop = population.unwrap_or(population_estimate(kind));
        // the factor 20 was suggested by gemini
        // (is it too large ? (Baden-Baden) with blackforest.gpx).
        return 20f64 * (pop as f64).sqrt();
    }
    300.0
}

pub fn update_track_projection(
    point: &mut InputPoint,
    euclidean: &Vec<MercatorPoint>,
    distance: impl Fn(usize) -> f64,
    elevation: impl Fn(usize) -> f64,
    tree: &locate::IndexedPointsTree,
) {
    let new_projection =
        locate::compute_track_projection(euclidean, distance, elevation, tree, point);
    if point.track_projections.is_empty() {
        point.track_projections.insert(new_projection);
        return;
    }

    let dmax = dmax(&point.kind(), &point.population());
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
    graphics_tree: locate::IndexedPointsTree,
    subtrees: Vec<locate::IndexedPointsTree>,
    parts: Vec<TrackPart>,
}

pub enum Resolution {
    #[allow(dead_code)]
    Graphics,
    Topology,
}

impl ProjectionTrees {
    pub fn parts(&self) -> Vec<TrackPart> {
        self.parts.clone()
    }
    pub fn make_parts(euclidean: &Vec<MercatorPoint>, resolution: &Resolution) -> Vec<TrackPart> {
        let start = 0;
        let end = euclidean.len();

        let coords: Vec<geo::Coord> = euclidean
            .iter()
            .map(|p| geo::coord!(x: p.x(), y: p.y()))
            .collect();
        let line = geo::LineString::new(coords);
        let epsilon = match resolution {
            Resolution::Graphics => {
                let start_point = euclidean.first().unwrap();
                let distance_from_start =
                    |index: &usize| -> f64 { start_point.d2(&euclidean[*index]) };
                let far_index = find_global_max(start, end, distance_from_start);
                let extend = distance_from_start(&far_index);
                extend * 500f64 / 500_000f64
            }
            Resolution::Topology => 10_000f64,
        };
        let split_indices = line.simplify_idx(epsilon);
        log::trace!("topology: {} parts", split_indices.len() - 1);
        let ranges: Vec<std::ops::Range<usize>> = split_indices
            .windows(2)
            .map(|window| window[0]..window[1])
            .collect();
        ranges
            .iter()
            .enumerate()
            .map(|(index, range)| TrackPart {
                name: format!("part-{}", index),
                length: range.len(),
                part_index: index,
            })
            .collect()
    }

    fn make_projection_trees_from_parts(
        euclidean: &Vec<MercatorPoint>,
        parts: &Vec<TrackPart>,
    ) -> Vec<locate::IndexedPointsTree> {
        parts_to_ranges(parts)
            .iter()
            .map(|range| locate::IndexedPointsTree::from_track(&euclidean, &range))
            .collect()
    }

    pub fn make_from_parts(
        euclidean: &Vec<MercatorPoint>,
        simplified: &Vec<MercatorPoint>,
        parts: &Vec<TrackPart>,
    ) -> Self {
        Self {
            total_tree: locate::IndexedPointsTree::from_track(&euclidean, &(0..euclidean.len())),
            graphics_tree: locate::IndexedPointsTree::from_track(
                &simplified,
                &(0..simplified.len()),
            ),
            subtrees: Self::make_projection_trees_from_parts(euclidean, parts),
            parts: parts.clone(),
        }
    }

    pub fn project(
        &self,
        point: &mut InputPoint,
        euclidean: &Vec<MercatorPoint>,
        distance: &impl Fn(usize) -> f64,
        elevation: &impl Fn(usize) -> f64,
    ) {
        update_track_projection(point, euclidean, distance, elevation, &self.total_tree);
        let index = point.track_projections.first().unwrap().track_index;
        if point.kind() == Kind::GPXWaypoints {
            log::trace!("[x] [1] project name: {:?}", point.name());
            log::trace!("[x] [1] project index:{}", index);
            log::trace!("[x] [1] project len:{}", point.track_projections.len());
        }
        if is_close_to_track(&point) {
            for tree in &self.subtrees {
                // consider a tree only if it does *not* contain the already known index.
                if !tree.range.contains(&index) {
                    update_track_projection(point, euclidean, distance, elevation, tree);
                    if point.kind() == Kind::GPXWaypoints {
                        log::trace!("[x] [2] project name: {:?}", point.name());
                        log::trace!("[x] [2] project len:{}", point.track_projections.len());
                    }
                }
            }
        }
    }

    pub fn project_graphics(
        &self,
        point: &MercatorPoint,
        euclidean: &Vec<MercatorPoint>,
    ) -> TrackProjection {
        locate::compute_track_projection_2d(&euclidean, &self.graphics_tree, point)
    }
}

#[cfg(test)]
mod tests {
    use crate::{gpsdata::GpxData, inputpoint::InputPointMap, wgs84point::WGS84Point};

    fn read(filename: String) -> GpxData {
        use crate::gpsdata;
        let mut f = std::fs::File::open(filename).unwrap();
        let mut content = Vec::new();
        // read the whole file
        use std::io::prelude::*;
        f.read_to_end(&mut content).unwrap();
        gpsdata::GpxData::read_content(&content).unwrap()
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
        let mut map = InputPointMap::new();
        map.insert_point(&mortagne);
        track.project_map(&mut map);
        map.iter().for_each(|p| {
            assert_eq!(p.track_projections.len(), 2);
            log::info!("p={:?}", p);
        });
    }
}
