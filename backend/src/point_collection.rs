use crate::{
    inputpoint::{InputPoint, InputPointMaps, InputType, OSMType},
    segment::SegmentData,
    track::Track,
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

pub struct PointCollection {
    pub user1: Points,
    pub user2: Points,
    pub cities: Points,
    pub controls: Points,
    pub gpx: Points,
    pub mountains: Points,
    pub villages: Points,
    pub osmrest: Points,
    pub offtrack_cities: Points,
}

pub type SharedPointCollection = std::sync::Arc<std::sync::RwLock<PointCollection>>;

impl PointCollection {
    pub fn new() -> Self {
        let empty = Points::new();
        PointCollection {
            user1: empty.clone(),
            user2: empty.clone(),
            cities: empty.clone(),
            controls: empty.clone(),
            gpx: empty.clone(),
            mountains: empty.clone(),
            villages: empty.clone(),
            osmrest: empty.clone(),
            offtrack_cities: empty.clone(),
        }
    }

    pub fn import_osm(&mut self, osmpoints: &Vec<InputPoint>, track: &Track) {
        debug_assert!(self.offtrack_cities.is_empty());
        debug_assert!(self.cities.is_empty());
        debug_assert!(self.mountains.is_empty());
        debug_assert!(self.villages.is_empty());
        debug_assert!(self.osmrest.is_empty());
        for k in 0..osmpoints.len() {
            let wi = osmpoints[k].clone();
            if !is_close_to_track(&wi) {
                match wi.osmkind().unwrap() {
                    OSMType::City => {
                        self.offtrack_cities.push(wi);
                    }
                    _ => {}
                }
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

        for point in &mut self.offtrack_cities {
            if point.track_projections.is_empty() {
                track.project_point(point);
            }
        }
        sort_by_distance_to_track(&mut self.offtrack_cities);
    }

    pub fn import_other(&mut self, pointmaps: &InputPointMaps, _track: &Track) {
        match pointmaps.maps.get(&InputType::Control) {
            Some(map) => self.controls = map.as_vector(),
            _ => {}
        }
        match pointmaps.maps.get(&InputType::GPX) {
            Some(map) => self.gpx = map.as_vector(),
            _ => {}
        }

        {
            self.user1.clear();
            self.user2.clear();
            let points = pointmaps
                .maps
                .get(&InputType::UserStep)
                .unwrap()
                .as_vector();
            let indices: Vec<_> = (0..points.len()).collect();
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
    }

    fn cities_and_mountains(&self) -> Vec<InputPoint> {
        merge_flip_flop(&self.cities, &self.mountains)
    }

    fn filter_for_segment(points: &mut Vec<InputPoint>, segment: &SegmentData) {
        let range = segment.range();
        points.retain(|point| point.is_in_range(&range));
    }

    fn filter_packets_for_segment(packets: &mut Vec<Vec<InputPoint>>, segment: &SegmentData) {
        packets
            .iter_mut()
            .for_each(|packet| Self::filter_for_segment(packet, segment));
    }

    fn export_profile(&self) -> Vec<Vec<InputPoint>> {
        // sort (peaks and passes) by elevation
        vec![
            self.controls.clone(),
            self.gpx.clone(),
            self.cities_and_mountains(),
            self.villages.clone(),
            self.osmrest.clone(),
        ]
    }

    pub fn profile(&self, segment: &SegmentData) -> Vec<Vec<InputPoint>> {
        let mut ret = self.export_profile();
        Self::filter_packets_for_segment(&mut ret, segment);
        ret
    }

    pub fn map(&self, segment: &SegmentData) -> Vec<Vec<InputPoint>> {
        let mut villages = self.villages.clone();
        Self::filter_for_segment(&mut villages, segment);
        let mut off = self.offtrack_cities.clone();
        Self::filter_for_segment(&mut off, segment);
        let villages_and_far_cities = merge_flip_flop(&off, &villages);
        let mut ret = vec![
            self.controls.clone(),
            //self.gpx.clone(),
            self.cities_and_mountains().clone(),
            villages_and_far_cities,
            self.osmrest.clone(),
        ];
        Self::filter_packets_for_segment(&mut ret, segment);
        ret
    }
}
