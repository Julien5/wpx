use std::collections::BTreeSet;

use crate::{
    inputpoint::{InputType, OSM},
    label_placement::PointFeature,
};

pub fn default(points: &Vec<PointFeature>) -> Vec<BTreeSet<usize>> {
    let mut user1 = BTreeSet::new();
    let mut user2 = BTreeSet::new();
    let mut osm1 = BTreeSet::new();
    let mut osm2 = BTreeSet::new();
    let mut gpx = BTreeSet::new();
    for k in 0..points.len() {
        let w = &points[k];
        let wi = w.input_point().unwrap();
        match wi.kind() {
            InputType::GPX => {
                gpx.insert(k);
            }
            InputType::OSM { kind: osm } => match osm {
                OSM::City => {
                    if wi.track_projection.as_ref().unwrap().track_distance < 2000f64 {
                        osm1.insert(k);
                    } else {
                        osm2.insert(k);
                    }
                }
                OSM::Village | OSM::MountainPass | OSM::Peak => {
                    if wi.track_projection.unwrap().track_distance < 300f64 {
                        osm1.insert(k);
                    } else {
                        osm2.insert(k);
                    }
                }
                _ => {
                    osm2.insert(k);
                }
            },
            InputType::UserStep => {
                if wi.name().unwrap_or("".to_string()).ends_with("0") {
                    user1.insert(k);
                } else {
                    user2.insert(k);
                }
            }
        }
    }
    vec![gpx, user1, user2, osm1, osm2]
}
