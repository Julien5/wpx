use crate::{
    inputpoint::{InputType, OSM},
    label_placement::PointFeature,
};

fn merge_flip_flop(_a: &Vec<usize>, _b: &Vec<usize>) -> Vec<usize> {
    let mut a = _a.clone();
    let mut b = _b.clone();
    let mut ret = Vec::new();
    while !a.is_empty() || !b.is_empty() {
        match ret.len() % 2 {
            0 => {
                if !a.is_empty() {
                    ret.push(a.first().unwrap().clone());
                    a.remove(0);
                } else {
                    ret.push(b.first().unwrap().clone());
                    b.remove(0);
                }
            }
            1 => {
                if !b.is_empty() {
                    ret.push(b.first().unwrap().clone());
                    b.remove(0);
                } else {
                    ret.push(a.first().unwrap().clone());
                    a.remove(0);
                }
            }
            _ => {
                assert!(false);
            }
        }
    }
    ret
}

fn sort_by_elevation(mountains: &mut Vec<usize>, points: &Vec<PointFeature>) {
    mountains.sort_by_key(|k| {
        std::cmp::Reverse(
            points
                .get(*k)
                .unwrap()
                .input_point
                .as_ref()
                .unwrap()
                .ele()
                .unwrap_or(0f64)
                .floor() as i32,
        )
    });
}

fn sort_by_distance_to_track(mountains: &mut Vec<usize>, points: &Vec<PointFeature>) {
    mountains.sort_by_key(|k| {
        points
            .get(*k)
            .unwrap()
            .input_point
            .as_ref()
            .unwrap()
            .track_projection
            .as_ref()
            .unwrap()
            .track_distance
            .floor() as i32
    });
}

fn sort_by_population(cities: &mut Vec<usize>, points: &Vec<PointFeature>) {
    cities.sort_by_key(|k| {
        std::cmp::Reverse(
            points
                .get(*k)
                .unwrap()
                .input_point
                .as_ref()
                .unwrap()
                .population()
                .unwrap_or(0),
        )
    });
}

pub fn profile(points: &Vec<PointFeature>) -> Vec<Vec<usize>> {
    let mut user1 = Vec::new();
    let mut user2 = Vec::new();
    let mut cities = Vec::new();
    let mut mountains = Vec::new();
    let mut villages = Vec::new();
    let mut osmrest = Vec::new();
    let mut gpx = Vec::new();
    for k in 0..points.len() {
        let w = &points[k];
        let wi = w.input_point().unwrap();
        match wi.kind() {
            InputType::GPX => {
                gpx.push(k);
            }
            InputType::OSM { kind: osm } => match osm {
                OSM::City => {
                    cities.push(k);
                }
                OSM::MountainPass | OSM::Peak => {
                    mountains.push(k);
                }
                OSM::Village => {
                    villages.push(k);
                }
                _ => {
                    osmrest.push(k);
                }
            },
            InputType::UserStep => {
                if wi.name().unwrap_or("".to_string()).ends_with("0") {
                    user1.push(k);
                } else {
                    user2.push(k);
                }
            }
        }
    }
    // sort (peaks and passes) by elevation
    sort_by_elevation(&mut mountains, points);
    sort_by_population(&mut cities, points);
    let osm1 = merge_flip_flop(&cities, &mountains);
    sort_by_population(&mut villages, points);
    vec![gpx, user1, osm1, villages, osmrest]
}

pub fn map(points: &Vec<PointFeature>, profile_indices: Vec<usize>) -> Vec<Vec<usize>> {
    let mut cities_far = Vec::new();
    for k in 0..points.len() {
        let w = &points[k];
        let iw = w.input_point.as_ref().unwrap();
        let distance = iw.track_projection.as_ref().unwrap().track_distance;
        match iw.kind() {
            InputType::OSM { kind: osm } => match osm {
                OSM::City => {
                    if distance < 2000f64 {
                    } else {
                        cities_far.push(k);
                    }
                }
                _ => {}
            },
            _ => {}
        }
    }
    sort_by_distance_to_track(&mut cities_far, points);
    // assert!(!profile_indices.is_empty());
    log::trace!("map-prioritize:{}", profile_indices.len());
    vec![profile_indices, cities_far]
}
