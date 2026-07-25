use std::range::Range;
#[allow(dead_code)]
use std::{cmp::Ordering, collections::BTreeSet};

use crate::{
    inputpoint::InputPoint,
    locate,
    mercator::{MercatorPoint, WebMercatorProjection},
    track::Track,
    wgs84point::WGS84Point,
};

use geo::SimplifyIdx;
use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct TrackProjection {
    pub track_floating_index: f64,
    pub euclidean: MercatorPoint,
    pub elevation: f64,
    pub track_distance: f64,
    pub distance_on_track_to_projection: f64,
}

impl TrackProjection {
    pub fn unproject(&self) -> WGS84Point {
        let mut wgs = WebMercatorProjection::make().unproject(&self.euclidean);
        wgs.2 = self.elevation;
        wgs
    }

    pub fn ontrack_clone(&self) -> Self {
        let mut ret = self.clone();
        ret.track_distance = 0f64;
        ret
    }

    pub fn track_index(&self) -> usize {
        self.track_floating_index.round() as usize
    }

    pub fn at_track_index(track: &Track, index: usize) -> Self {
        TrackProjection {
            track_floating_index: index as f64,
            euclidean: track.map.point_at(index).clone(),
            elevation: track.elevation(index),
            track_distance: 0f64,
            distance_on_track_to_projection: track.distance(index),
        }
    }

    pub fn at_distance(track: &Track, distance: f64) -> Self {
        let i1 = track.profile.index_before(distance);
        let i2 = track.profile.index_after(distance);
        let p1 = track.map.point_at(i1);
        let p2 = track.map.point_at(i2);
        let d1 = track.distance(i1);
        let d2 = track.distance(i2);
        debug_assert!(d1 <= d2);
        debug_assert!(i1 <= i2);
        let ad = distance - d1;
        let a2 = d2 - d1;
        let alpha = if a2 > 0f64 {
            ad / a2
        } else {
            debug_assert!(
                d1 == d2 && distance == d2,
                "d1={}, distance={} d2={}",
                d1,
                distance,
                d2,
            );
            0f64
        };
        debug_assert!(
            0.0 <= alpha && alpha <= 1.0,
            "d1={}, distance={} d2={} =>alpha={}",
            d1,
            distance,
            d2,
            alpha
        );
        let findex = i1 as f64 + alpha;
        let m = MercatorPoint::from_point2d(&((1.0 - alpha) * p1.point2d() + alpha * p2.point2d()));
        let z = (1.0 - alpha) * track.elevation(i1) + alpha * track.elevation(i2);

        TrackProjection {
            track_floating_index: findex,
            euclidean: m,
            elevation: z,
            track_distance: 0f64,
            distance_on_track_to_projection: distance,
        }
    }
}

pub type TrackProjections = BTreeSet<TrackProjection>;

#[allow(dead_code)]
pub fn string_projection(projection: &TrackProjection) -> String {
    format!("proj index:{}", projection.track_index())
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
        // keep equality based on the full floating index total order
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

    let dmax = point.dmax();
    let d = new_projection.track_distance;
    if d > dmax {
        return;
    }

    let known = point.track_projections.iter().any(|proj| {
        let d1 = proj.distance_on_track_to_projection;
        let d2 = new_projection.distance_on_track_to_projection;
        let delta = (d1 - d2).abs();
        let delta_max = 10f64 * dmax;
        delta < delta_max
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

#[derive(Clone)]
pub struct ProjectionTrees {
    total_tree: locate::IndexedPointsTree,
    subtrees: Vec<locate::IndexedPointsTree>,
    ranges: Vec<Range<usize>>,
}

pub enum Resolution {
    #[allow(dead_code)]
    Graphics,
    Topology,
}

impl ProjectionTrees {
    pub fn ranges(&self) -> Vec<std::range::Range<usize>> {
        self.ranges.clone()
    }

    #[allow(dead_code)]
    pub fn debug(&self) {
        log::trace!("(proj) total tree {:?}", self.total_tree.range);
        for tree in &self.subtrees {
            log::trace!("(proj) sub  tree {:?}", tree.range);
        }
    }

    pub fn make_parts(
        euclidean: &Vec<MercatorPoint>,
        resolution: &Resolution,
    ) -> Vec<std::range::Range<usize>> {
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
            // for small test tours, this is too large.
            Resolution::Topology => 10_000f64,
        };
        let split_indices = line.simplify_idx(epsilon);
        let ranges: Vec<std::range::Range<usize>> = split_indices
            .windows(2)
            .map(|window| {
                if window[1] == end - 1 {
                    (window[0]..end).into()
                } else {
                    (window[0]..window[1]).into()
                }
            })
            .collect();
        ranges
    }

    pub fn make_from_parts(
        euclidean: &Vec<MercatorPoint>,
        ranges: &Vec<std::range::Range<usize>>,
    ) -> Self {
        Self {
            total_tree: locate::IndexedPointsTree::from_track(
                &euclidean,
                &(0..euclidean.len()).into(),
            ),
            subtrees: ranges
                .iter()
                .map(|range| locate::IndexedPointsTree::from_track(&euclidean, &range))
                .collect(),
            ranges: ranges.clone(),
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
        let index = point.track_projections.first().unwrap().track_index();
        if point.is_close_to_track() {
            for tree in &self.subtrees {
                // consider a tree only if it does *not* contain the already known index.
                if !tree.range.contains(&index) {
                    update_track_projection(point, euclidean, distance, elevation, tree);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {

    use crate::{
        gpsdata::GpxData,
        inputpoint::{GPXWaypointData, InputPointData, InputPointMap},
        trackparts::ProtoTrack,
    };

    fn read(filename: String) -> GpxData {
        use crate::gpsdata;
        let mut f = std::fs::File::open(filename).unwrap();
        let mut content = Vec::new();
        // read the whole file
        use std::io::prelude::*;
        f.read_to_end(&mut content).unwrap();
        gpsdata::GpxData::read_content(&content).unwrap()
    }

    #[test]
    fn projection() {
        let _ = env_logger::try_init();
        use crate::track_projection::*;
        //let gpxdata = read("data/ref/pbp2023.gpx".to_string());
        let gpxdata = read("data/ref/pbp2019.gpx".to_string());
        let mut tags = std::collections::BTreeMap::new();
        tags.insert("wpxtype".to_string(), "OSM".to_string());
        tags.insert("name".to_string(), "Mortagne-au-Perche".to_string());
        tags.insert("place".to_string(), "town".to_string());
        tags.insert("population".to_string(), "3815".to_string());
        let pos = MercatorPoint::new(61237.909420542324, 6193890.266343569);
        let mortagne = InputPoint {
            wgs84: WGS84Point::new(&0.5501095, &48.5205106, &0.0),
            euclidean: pos.clone(),
            data: InputPointData::GPXWaypoint(GPXWaypointData::default()),
            track_projections: TrackProjections::new(),
            index: None,
        };
        let proto = ProtoTrack::new(&gpxdata.tracks).unwrap();
        let track = Track::from_proto(&proto).unwrap();
        let mut map = InputPointMap::new();
        map.insert_point(&mortagne);
        track.project_map(&mut map);
        map.iter().for_each(|p| {
            debug_assert_eq!(p.track_projections.len(), 2);
            log::info!("p={:?}", p);
        });
    }
}
