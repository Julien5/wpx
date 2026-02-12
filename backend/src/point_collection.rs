use crate::{
    inputpoint::{InputPoint, InputPointMaps, InputType, OSMType},
    math::IntegerSize2D,
    parameters::Parameters,
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

#[derive(Clone)]
pub struct RenderResult {
    pub svg: String,
    pub rendered: Vec<InputPoint>,
}

#[derive(Clone)]
struct RenderInputParameters {
    controls: Vec<InputPoint>,
    parameters: Parameters,
    range: std::ops::Range<usize>,
    screen_size: IntegerSize2D,
}

impl RenderInputParameters {
    pub fn covers(&self, other: &Self, function: &RenderFunction) -> bool {
        if self.screen_size.width < other.screen_size.width {
            log::trace!("missmatch: width");
            return false;
        }
        if self.screen_size.height < other.screen_size.height {
            log::trace!("missmatch: height");
            return false;
        }
        match function {
            RenderFunction::Map => {
                if self.parameters.map_options.max_area_ratio
                    < other.parameters.map_options.max_area_ratio
                {
                    log::trace!("missmatch: max_area_ratio (map)");
                    return false;
                }
            }
            RenderFunction::Profile => {
                if self.parameters.profile_options.max_area_ratio
                    < other.parameters.profile_options.max_area_ratio
                {
                    log::trace!("missmatch: max_area_ratio (profile)");
                    return false;
                }
            }
        }
        if self.range.start > other.range.start {
            log::trace!("missmatch: range start");
            return false;
        }
        if self.range.end < other.range.end {
            log::trace!("missmatch: range end");
            return false;
        }
        if self.controls != other.controls {
            log::trace!("missmatch: controls");
            return false;
        }
        true
    }
}

#[derive(Clone, PartialEq, Eq)]
enum RenderFunction {
    Map,
    Profile,
}

#[derive(Clone)]
struct CachedResult {
    output: RenderResult,
    function: RenderFunction,
    parameters: RenderInputParameters,
}

struct CachedResults {
    results: Vec<CachedResult>,
}

// in situations where the *same* parameters are used
// we might directly return the svg

// TODO: limit the cache size

impl CachedResults {
    pub fn new() -> Self {
        Self {
            results: Vec::new(),
        }
    }
    pub fn push(&mut self, result: CachedResult) {
        self.results.push(result);
    }
    pub fn hit(
        &self,
        function: &RenderFunction,
        parameters: &RenderInputParameters,
    ) -> Option<RenderResult> {
        for result in &self.results {
            if result.function != *function {
                continue;
            }
            if result.parameters.covers(parameters, function) {
                return Some(result.output.clone());
            }
        }
        None
    }
    pub fn size(&self) -> usize {
        self.results.len()
    }
}

pub type SharedPacketProvider = std::sync::Arc<std::sync::RwLock<PacketProvider>>;

pub struct PacketProvider {
    collection: PointCollection,
    results: CachedResults,
}

impl PacketProvider {
    pub fn new() -> Self {
        Self {
            collection: PointCollection::new(),
            results: CachedResults::new(),
        }
    }
    pub fn register_map_result(
        &mut self,
        parameters: &Parameters,
        range: &std::ops::Range<usize>,
        size: &IntegerSize2D,
        result: &RenderResult,
    ) {
        self.register_result(&RenderFunction::Map, parameters, range, size, result);
    }
    pub fn register_profile_result(
        &mut self,
        parameters: &Parameters,
        range: &std::ops::Range<usize>,
        size: &IntegerSize2D,
        result: &RenderResult,
    ) {
        self.register_result(&RenderFunction::Profile, parameters, range, size, result);
    }
    fn register_result(
        &mut self,
        function: &RenderFunction,
        parameters: &Parameters,
        range: &std::ops::Range<usize>,
        size: &IntegerSize2D,
        result: &RenderResult,
    ) {
        let p = RenderInputParameters {
            range: range.clone(),
            screen_size: size.clone(),
            parameters: parameters.clone(),
            controls: self.collection.controls.clone(),
        };
        let mut output = result.clone();
        output.rendered.retain(|w| w.kind() != InputType::UserStep);
        let result = CachedResult {
            function: function.clone(),
            parameters: p.clone(),
            output,
        };
        if self.results.hit(function, &p).is_none() {
            self.results.push(result);
        }
        log::trace!("cache size: {}", self.results.size());
    }
    pub fn import_osm(&mut self, points: &Vec<InputPoint>, _track: &Track) {
        self.collection.import_osm(points, _track);
    }
    pub fn import_other(&mut self, pointmaps: &InputPointMaps, _track: &Track) {
        self.collection.import_other(pointmaps, _track);
    }
    pub fn map(
        &self,
        range: &std::ops::Range<usize>,
        parameters: &Parameters,
        size: &IntegerSize2D,
    ) -> Vec<Vec<InputPoint>> {
        let p = RenderInputParameters {
            range: range.clone(),
            parameters: parameters.clone(),
            screen_size: size.clone(),
            controls: self.collection.controls.clone(),
        };
        match self.results.hit(&RenderFunction::Map, &p) {
            Some(result) => {
                log::trace!("packet provider map cache hit");
                // HACK: the user steps are "updated in the cache".
                // TODO: Fix this later.
                vec![self.collection.user.clone(), result.rendered]
            }
            None => {
                log::trace!("packet provider map cache miss");
                self.collection.map(range)
            }
        }
    }
    pub fn profile(
        &self,
        range: &std::ops::Range<usize>,
        parameters: &Parameters,
        size: &IntegerSize2D,
    ) -> Vec<Vec<InputPoint>> {
        let p = RenderInputParameters {
            range: range.clone(),
            parameters: parameters.clone(),
            screen_size: size.clone(),
            controls: self.collection.controls.clone(),
        };
        match self.results.hit(&RenderFunction::Profile, &p) {
            Some(result) => {
                log::trace!("packet provider profile cache hit");
                // HACK: the user steps are "updated in the cache".
                // TODO: Fix this later.
                vec![self.collection.user.clone(), result.rendered]
            }
            None => {
                log::trace!("packet provider profile cache miss");
                self.collection.profile(range)
            }
        }
    }
}

type Points = Vec<InputPoint>;

pub struct PointCollection {
    pub user: Points,
    pub cities: Points,
    pub controls: Points,
    pub gpx: Points,
    pub mountains: Points,
    pub villages: Points,
    pub osmrest: Points,
    pub offtrack_cities: Points,
}

impl PointCollection {
    pub fn new() -> Self {
        let empty = Points::new();
        PointCollection {
            user: empty.clone(),
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
        self.offtrack_cities.clear();
        self.cities.clear();
        self.mountains.clear();
        self.villages.clear();
        self.osmrest.clear();
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
            self.user.clear();
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
                self.user.push(wi);
            }
        }
    }

    fn cities_and_mountains(&self) -> Vec<InputPoint> {
        merge_flip_flop(&self.cities, &self.mountains)
    }

    fn filter_for_segment(points: &mut Vec<InputPoint>, range: &std::ops::Range<usize>) {
        points.retain(|point| point.is_in_range(range));
    }

    fn filter_packets_for_segment(
        packets: &mut Vec<Vec<InputPoint>>,
        range: &std::ops::Range<usize>,
    ) {
        packets
            .iter_mut()
            .for_each(|packet| Self::filter_for_segment(packet, range));
    }

    fn export_profile(&self) -> Vec<Vec<InputPoint>> {
        vec![
            self.user.clone(),
            self.controls.clone(),
            self.gpx.clone(),
            self.cities_and_mountains(),
            self.villages.clone(),
            self.osmrest.clone(),
        ]
    }

    pub fn profile(&self, range: &std::ops::Range<usize>) -> Vec<Vec<InputPoint>> {
        let mut ret = self.export_profile();
        Self::filter_packets_for_segment(&mut ret, range);
        ret
    }

    pub fn map(&self, range: &std::ops::Range<usize>) -> Vec<Vec<InputPoint>> {
        let mut villages = self.villages.clone();
        Self::filter_for_segment(&mut villages, range);
        let mut off = self.offtrack_cities.clone();
        Self::filter_for_segment(&mut off, range);
        let villages_and_far_cities = merge_flip_flop(&off, &villages);
        let mut ret = vec![
            self.user.clone(),
            self.controls.clone(),
            self.gpx.clone(),
            self.cities_and_mountains().clone(),
            villages_and_far_cities,
            self.osmrest.clone(),
        ];
        Self::filter_packets_for_segment(&mut ret, range);
        ret
    }
}
