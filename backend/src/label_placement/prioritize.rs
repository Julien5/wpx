use crate::{
    inputpoint::{InputPoint, InputType, OSMType},
    make_points::is_close_to_track,
    segment::Segment,
};

fn merge_flip_flop<'a>(_a: &Vec<&'a InputPoint>, _b: &Vec<&'a InputPoint>) -> Vec<&'a InputPoint> {
    let mut a = _a.clone();
    let mut b = _b.clone();
    let mut ret = Vec::new();
    while !a.is_empty() || !b.is_empty() {
        match ret.len() % 2 {
            0 => {
                if !a.is_empty() {
                    ret.push(*a.first().unwrap());
                    a.remove(0);
                } else {
                    ret.push(*b.first().unwrap());
                    b.remove(0);
                }
            }
            1 => {
                if !b.is_empty() {
                    ret.push(*b.first().unwrap());
                    b.remove(0);
                } else {
                    ret.push(*a.first().unwrap());
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

fn sort_by_elevation(mountains: &mut Vec<&InputPoint>) {
    mountains.sort_by_key(|w| std::cmp::Reverse(w.ele().unwrap_or(0f64).floor() as i32));
}

fn sort_by_distance_to_track(mountains: &mut Vec<&InputPoint>) {
    mountains.sort_by_key(|w| w.track_projection.as_ref().unwrap().track_distance.floor() as i32);
}

fn sort_by_population(cities: &mut Vec<&InputPoint>) {
    cities.sort_by_key(|w| std::cmp::Reverse(w.population().unwrap_or(0)));
}

pub fn profile(segment: &Segment) -> Vec<Vec<&InputPoint>> {
    let mut user1 = Vec::new();
    let mut user2 = Vec::new();
    let mut cities = Vec::new();
    let mut mountains = Vec::new();
    let mut villages = Vec::new();
    let mut osmrest = Vec::new();

    match segment.points.get(&InputType::UserStep) {
        Some(points) => {
            for wi in points {
                if wi.name().unwrap_or("".to_string()).ends_with("0") {
                    user1.push(wi);
                } else {
                    user2.push(wi);
                }
            }
        }
        _ => {}
    }

    let gpx: Vec<_> = segment
        .points
        .get(&InputType::GPX)
        .unwrap()
        .iter()
        .map(|w| w)
        .collect();
    let osmpoints = segment.osmpoints();
    for k in 0..osmpoints.len() {
        let wi = &osmpoints[k];
        if !is_close_to_track(wi) {
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
    vec![gpx, user1, cities_and_mountains, villages, osmrest]
}

pub fn map(segment: &Segment) -> Vec<Vec<&InputPoint>> {
    let profile_points = profile(segment);
    let gpx = &profile_points.get(0).unwrap();
    let user1 = &profile_points.get(1).unwrap();
    let mountains_and_cities = &profile_points.get(2).unwrap();
    let villages = &profile_points.get(3).unwrap();
    let osmrest = &profile_points.get(4).unwrap();
    let mut offtrack_cities = Vec::new();
    let osmpoints = segment.osmpoints();
    for w in osmpoints {
        if mountains_and_cities.contains(&w) {
            continue;
        }
        let mut in_profile = false;
        for packet in &profile_points {
            if in_profile {
                break;
            }
            if packet.contains(&w) {
                in_profile = true;
                break;
            }
        }
        if in_profile {
            continue;
        }
        match w.osmkind().unwrap() {
            OSMType::City => {
                log::trace!("offtrack city:{}", w.name().unwrap());
                offtrack_cities.push(w);
            }
            _ => {}
        }
    }
    sort_by_distance_to_track(&mut offtrack_cities);
    //offtrack_cities.truncate(300);
    //sort_by_population(&mut offtrack_cities, &segment.points);
    let villages_and_far_cities = merge_flip_flop(&offtrack_cities, &villages);
    for w in &villages_and_far_cities {
        log::trace!("ret-offtrack city:{}", w.name().unwrap());
    }
    vec![
        (*gpx).clone(),
        (*user1).clone(),
        (*mountains_and_cities).clone(),
        villages_and_far_cities,
        (*osmrest).clone(),
    ]
}
