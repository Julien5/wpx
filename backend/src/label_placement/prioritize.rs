use crate::{
    inputpoint::{InputPoint, InputType, OSMType},
    segment::SegmentData,
    track_projection::is_close_to_track,
};

fn merge_flip_flop<T: Clone>(a: &[T], b: &[T]) -> Vec<T> {
    // gemini
    let mut result = Vec::with_capacity(a.len() + b.len());
    let max_len = std::cmp::max(a.len(), b.len());

    for i in 0..max_len {
        if let Some(val_a) = a.get(i) {
            result.push(val_a.clone());
        }
        if let Some(val_b) = b.get(i) {
            result.push(val_b.clone());
        }
    }

    result
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

type Points = Vec<InputPoint>;

struct ProfilePoints {
    pub user1: Points,
    pub user2: Points,
    pub cities: Points,
    pub controls: Points,
    pub mountains: Points,
    pub villages: Points,
    pub osmrest: Points,
}

impl ProfilePoints {
    fn new() -> Self {
        let empty = Points::new();
        ProfilePoints {
            user1: empty.clone(),
            user2: empty.clone(),
            cities: empty.clone(),
            controls: empty.clone(),
            mountains: empty.clone(),
            villages: empty.clone(),
            osmrest: empty.clone(),
        }
    }
    fn import(&mut self, segment: &SegmentData) {
        let controls = segment.points(&InputType::Control);
        let gpx = segment.points(&InputType::GPX);
        self.controls = if controls.is_empty() {
            gpx.clone()
        } else {
            controls.clone()
        };
        let trackrange = segment.range();
        {
            let points = segment.points(&InputType::UserStep);
            let mut indices: Vec<_> = (0..points.len()).collect();
            indices.retain(|i| points[*i].is_in_range(&trackrange));
            for k in indices {
                let wi = points[k].clone();
                assert!(is_close_to_track(&wi));
                let d = wi.distance_to_track();
                assert_eq!(wi.kind(), InputType::UserStep);
                assert_eq!(d, 0f64);
                if self.user1.len() < self.user2.len() {
                    self.user1.push(wi);
                } else {
                    self.user2.push(wi);
                }
            }
        }

        let osmpoints = segment.osmpoints();
        for k in 0..osmpoints.len() {
            let wi = osmpoints[k].clone();
            if !is_close_to_track(&wi) {
                continue;
            }
            match wi.osmkind().unwrap() {
                OSMType::City => {
                    self.cities.push(wi);
                }
                OSMType::MountainPass | OSMType::Peak => {
                    self.mountains.push(wi);
                }
                OSMType::Village => {
                    self.villages.push(wi);
                }
                _ => {
                    self.osmrest.push(wi);
                }
            }
        }
        sort_by_elevation(&mut self.mountains);
        sort_by_population(&mut self.cities);
        sort_by_population(&mut self.villages);
    }
    fn cities_and_mountains(&self) -> Vec<InputPoint> {
        merge_flip_flop(&self.cities, &self.mountains)
    }

    fn export(&mut self) -> Vec<Vec<InputPoint>> {
        // sort (peaks and passes) by elevation
        vec![
            self.controls.clone(),
            self.user1.clone(),
            self.cities_and_mountains(),
            self.user2.clone(),
            self.villages.clone(),
            self.osmrest.clone(),
        ]
    }
}

pub fn profile(segment: &SegmentData) -> Vec<Vec<InputPoint>> {
    let mut profile_points = ProfilePoints::new();
    profile_points.import(segment);
    profile_points.export()
}

pub fn map(segment: &SegmentData) -> Vec<Vec<InputPoint>> {
    let mut profile_points = ProfilePoints::new();
    profile_points.import(segment);
    let mut offtrack_cities = Vec::new();
    let osmpoints = segment.osmpoints();
    for w in osmpoints {
        if is_close_to_track(&w) {
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
        if point.track_projections.is_empty() {
            segment.track.project_point(point);
        }
    }
    sort_by_distance_to_track(&mut offtrack_cities);
    let villages_and_far_cities = merge_flip_flop(&offtrack_cities, &profile_points.villages);
    vec![
        (profile_points.controls).clone(),
        (profile_points.user1).clone(),
        (profile_points.cities_and_mountains()).clone(),
        (profile_points.user2).clone(),
        villages_and_far_cities,
        (profile_points.osmrest).clone(),
    ]
}
