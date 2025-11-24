use crate::{
    inputpoint::{InputPoint, InputType, OSM},
    make_points::is_close_to_track,
    segment::Segment,
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

fn sort_by_elevation(mountains: &mut Vec<usize>, points: &Vec<InputPoint>) {
    mountains.sort_by_key(|k| {
        std::cmp::Reverse(
            points
                .get(*k)
                .as_ref()
                .unwrap()
                .ele()
                .unwrap_or(0f64)
                .floor() as i32,
        )
    });
}

fn sort_by_distance_to_track(mountains: &mut Vec<usize>, points: &Vec<InputPoint>) {
    mountains.sort_by_key(|k| {
        points
            .get(*k)
            .as_ref()
            .unwrap()
            .track_projection
            .as_ref()
            .unwrap()
            .track_distance
            .floor() as i32
    });
}

fn sort_by_population(cities: &mut Vec<usize>, points: &Vec<InputPoint>) {
    cities.sort_by_key(|k| {
        std::cmp::Reverse(points.get(*k).as_ref().unwrap().population().unwrap_or(0))
    });
}

pub fn profile(segment: &Segment) -> Vec<Vec<usize>> {
    let mut user1 = Vec::new();
    let mut user2 = Vec::new();
    let mut cities = Vec::new();
    let mut mountains = Vec::new();
    let mut villages = Vec::new();
    let mut osmrest = Vec::new();
    let mut gpx = Vec::new();
    for k in 0..segment.points.len() {
        let wi = &segment.points[k];
        if !is_close_to_track(&wi) {
            continue;
        }
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
    sort_by_elevation(&mut mountains, &segment.points);
    sort_by_population(&mut cities, &segment.points);
    let cities_and_mountains = merge_flip_flop(&cities, &mountains);
    sort_by_population(&mut villages, &segment.points);
    vec![gpx, user1, cities_and_mountains, villages, osmrest]
}

pub fn map(segment: &Segment) -> Vec<Vec<usize>> {
    let profile_indices = profile(segment);
    let gpx = &profile_indices.get(0).unwrap();
    let user1 = &profile_indices.get(1).unwrap();
    let mountains_and_cities = &profile_indices.get(2).unwrap();
    let villages = &profile_indices.get(3).unwrap();
    let osmrest = &profile_indices.get(3).unwrap();
    let mut offtrack_cities = Vec::new();
    for k in 0..segment.points.len() {
        if mountains_and_cities.contains(&k) {
            continue;
        }
        let iw = &segment.points[k];
        match iw.kind() {
            InputType::OSM { kind: osm } => match osm {
                OSM::City => {
                    log::trace!("offtrack city:{}", iw.name().unwrap());
                    offtrack_cities.push(k);
                }
                _ => {}
            },
            _ => {}
        }
    }
    sort_by_distance_to_track(&mut offtrack_cities, &segment.points);
    offtrack_cities.truncate(10);
    //sort_by_population(&mut offtrack_cities, &segment.points);
    let villages_and_far_cities = merge_flip_flop(&offtrack_cities, &villages);
    for k in &villages_and_far_cities {
        log::trace!("ret-offtrack city:{}", segment.points[*k].name().unwrap());
    }
    vec![
        (*gpx).clone(),
        (*user1).clone(),
        (*mountains_and_cities).clone(),
        villages_and_far_cities,
        (*osmrest).clone(),
    ]
}
