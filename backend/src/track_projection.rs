use std::{cmp::Ordering, collections::BTreeSet};

use crate::{
    inputpoint::{InputPoint, InputType, OSMType},
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

fn is_close(
    track_distance: &f64,
    kind: &InputType,
    osmkind: &Option<OSMType>,
    population: &Option<i32>,
) -> bool {
    let d = track_distance;
    match kind {
        InputType::OSM => {
            let okind = osmkind.as_ref().unwrap();
            let pop = population.unwrap_or(0);
            if *okind == OSMType::City || pop > 1000 {
                return *d < 2000.0;
            }
        }
        _ => {}
    }
    return *d < 300.0;
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

    if !is_close(
        &new_projection.track_distance,
        &point.kind(),
        &point.osmkind(),
        &point.population(),
    ) {
        return;
    }
    let known = point.track_projections.iter().any(|proj| {
        let d1 = proj.distance_on_track_to_projection;
        let d2 = new_projection.distance_on_track_to_projection;
        (d1 - d2).abs() < 10f64
    });

    if !known {
        point.track_projections.insert(new_projection);
    }
}
