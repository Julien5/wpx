use crate::{
    inputpoint::{InputPoint, InputType, OSMType},
    locate,
    segment::Segment,
    track_projection::is_close_to_track,
};

fn merge_flip_flop(_a: &Vec<InputPoint>, _b: &Vec<InputPoint>) -> Vec<InputPoint> {
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

fn sort_by_elevation(mountains: &mut Vec<InputPoint>) {
    mountains.sort_by_key(|w| std::cmp::Reverse(w.ele().unwrap_or(0f64).floor() as i32));
}

fn sort_by_distance_to_track(mountains: &mut Vec<InputPoint>) {
    mountains.sort_by_key(|w| w.distance_to_track().floor() as i32);
}

fn sort_by_population(cities: &mut Vec<InputPoint>) {
    cities.sort_by_key(|w| std::cmp::Reverse(w.population().unwrap_or(0)));
}

pub fn profile(segment: &Segment) -> Vec<Vec<InputPoint>> {
    let mut user1 = Vec::new();
    let mut user2 = Vec::new();
    let mut cities = Vec::new();
    let mut mountains = Vec::new();
    let mut villages = Vec::new();
    let mut osmrest = Vec::new();
    let trackrange = segment.range();
    match segment
        .pointmaps
        .read()
        .unwrap()
        .maps
        .get(&InputType::UserStep)
    {
        Some(map) => {
            let points = map.as_vector();
            let mut indices: Vec<_> = (0..points.len()).collect();
            indices.sort_by_key(|i| points[*i].round_track_index());
            indices.retain(|i| trackrange.contains(&points[*i].round_track_index().unwrap()));
            for k in indices {
                let wi = points[k].clone();
                assert!(is_close_to_track(&wi));
                let d = wi.distance_to_track();
                assert_eq!(wi.kind(), InputType::UserStep);
                assert_eq!(d, 0f64);
                if user1.len() < user2.len() {
                    user1.push(wi);
                } else {
                    user2.push(wi);
                }
            }
        }
        _ => {}
    }

    let gpx: Vec<_> = segment
        .pointmaps
        .read()
        .unwrap()
        .maps
        .get(&InputType::GPX)
        .unwrap()
        .as_vector();

    let osmpoints = segment.osmpoints();
    for k in 0..osmpoints.len() {
        let wi = osmpoints[k].clone();
        if !is_close_to_track(&wi) {
            continue;
        }
        match wi.osmkind().unwrap() {
            OSMType::City => {
                cities.push(wi);
            }
            OSMType::MountainPass | OSMType::Peak => {
                mountains.push(wi);
            }
            OSMType::Village => {
                villages.push(wi);
            }
            _ => {
                osmrest.push(wi);
            }
        }
    }
    // sort (peaks and passes) by elevation
    sort_by_elevation(&mut mountains);
    sort_by_population(&mut cities);
    let cities_and_mountains = merge_flip_flop(&cities, &mountains);
    sort_by_population(&mut villages);
    vec![gpx, user1, cities_and_mountains, user2, villages, osmrest]
}

pub fn map(segment: &Segment) -> Vec<Vec<InputPoint>> {
    let profile_points = profile(segment);
    let gpx = &profile_points.get(0).unwrap();
    let user1 = &profile_points.get(1).unwrap();
    let cities_and_mountains = &profile_points.get(2).unwrap();
    let user2 = &profile_points.get(3).unwrap();
    let villages = &profile_points.get(4).unwrap();
    let osmrest = &profile_points.get(5).unwrap();
    let mut offtrack_cities = Vec::new();
    let osmpoints = segment.osmpoints();
    for w in osmpoints {
        if cities_and_mountains.contains(&w) {
            continue;
        }
        match w.osmkind().unwrap() {
            OSMType::City => {
                offtrack_cities.push(w);
            }
            _ => {}
        }
    }
    for point in &mut offtrack_cities {
        let proj = locate::compute_track_projection(&segment.track, &segment.track.tree, &point);
        if point.track_projections.is_empty() {
            point.track_projections.insert(proj);
        }
    }
    sort_by_distance_to_track(&mut offtrack_cities);
    //sort_by_population(&mut offtrack_cities);
    let villages_and_far_cities = merge_flip_flop(&offtrack_cities, &villages);
    vec![
        (*gpx).clone(),
        (*user1).clone(),
        (*cities_and_mountains).clone(),
        (*user2).clone(),
        villages_and_far_cities,
        (*osmrest).clone(),
    ]
}
