use std::collections::{BTreeMap, HashSet};

use crate::{
    inputpoint::InputPoint, math::IntegerSize2D, parameters::Parameters,
    track_projection::is_close_to_track,
};

fn sort_by_elevation(mountains: &mut Vec<InputPoint>) {
    mountains.sort_by_key(|w| std::cmp::Reverse(w.ele().unwrap_or(0f64).floor() as i32));
}

fn _sort_by_distance_to_track(mountains: &mut Vec<InputPoint>) {
    mountains.sort_by_key(|w| w.distance_to_track().floor() as i32);
}

fn sort_by_population(cities: &mut Vec<InputPoint>) {
    cities.sort_by_key(|w| std::cmp::Reverse(w.population().unwrap_or(0)));
}

#[derive(Clone, Debug)]
pub struct RenderResult {
    pub svg: String,
    pub rendered: Vec<InputPoint>,
}

#[derive(Clone, Debug)]
struct RenderInputParameters {
    parameters: Parameters,
    range: std::ops::Range<usize>,
    screen_size: IntegerSize2D,
}

impl RenderInputParameters {
    pub fn missmatch(&self, other: &Self, function: &RenderFunction) -> String {
        if self.screen_size.width < other.screen_size.width {
            return format!(
                "screen size width ({}<{})",
                self.screen_size.width, other.screen_size.width
            );
        }
        if self.screen_size.height < other.screen_size.height {
            return format!(
                "screen size height ({}<{})",
                self.screen_size.height, other.screen_size.height
            );
        }
        match function {
            RenderFunction::Map => {
                if self.parameters.map_options.max_area_ratio
                    < other.parameters.map_options.max_area_ratio
                {
                    return format!(
                        "map area ratio ({}<{})",
                        self.parameters.map_options.max_area_ratio,
                        other.parameters.map_options.max_area_ratio
                    );
                }
            }
            RenderFunction::Profile => {
                if self.parameters.profile_options.max_area_ratio
                    < other.parameters.profile_options.max_area_ratio
                {
                    return format!(
                        "profile area ratio ({}<{})",
                        self.parameters.profile_options.max_area_ratio,
                        other.parameters.profile_options.max_area_ratio
                    );
                }
            }
        }
        if self.range.start > other.range.start {
            return format!("range start ({}>{})", self.range.start, other.range.start);
        }
        if self.range.end < other.range.end {
            return format!("range end ({}<{})", self.range.end, other.range.end);
        }
        String::new()
    }
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum RenderFunction {
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

            let missmatch = result.parameters.missmatch(parameters, function);
            if missmatch.is_empty() {
                log::trace!("cache hit for function: {:?}", function);
                return Some(result.output.clone());
            } else {
                log::trace!(
                    "cache missmatch for function: {:?} and parameters: {}",
                    function,
                    missmatch
                );
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
    pub collection: PointCollection,
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
    pub fn register_result(
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
        };
        let mut output = result.clone();
        output.rendered.retain(|w| w.kind() != Kind::UserStep);
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

    pub fn hit(
        &self,
        function: &RenderFunction,
        range: &std::ops::Range<usize>,
        parameters: &Parameters,
        size: &IntegerSize2D,
    ) -> bool {
        let p = RenderInputParameters {
            range: range.clone(),
            parameters: parameters.clone(),
            screen_size: size.clone(),
        };
        self.results.hit(function, &p).is_some()
    }

    pub fn load(
        &self,
        function: &RenderFunction,
        range: &std::ops::Range<usize>,
        parameters: &Parameters,
        size: &IntegerSize2D,
    ) -> PointCollection {
        let p = RenderInputParameters {
            range: range.clone(),
            parameters: parameters.clone(),
            screen_size: size.clone(),
        };
        match self.results.hit(function, &p) {
            Some(result) => {
                log::trace!("packet provider profile cache hit");
                let mut ret = self.collection.clone();
                // TODO: we should avoid to copy all osm points
                ret.import_osm(&result.rendered);
                ret.range_cut(range);
                ret
            }
            None => {
                log::trace!(
                    "packet provider profile cache miss for function: {:?}",
                    function
                );
                assert!(false);
                PointCollection::new()
            }
        }
    }
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Hash)]
pub enum Kind {
    Cities,
    Controls,
    GPXWaypoints,
    Hamlets,
    Mountains,
    Villages,
    UserStep,
}

pub fn is_osm(kind: &Kind) -> bool {
    match kind {
        Kind::Controls | Kind::GPXWaypoints | Kind::UserStep => false,
        _ => true,
    }
}

pub type Kinds = HashSet<Kind>;
pub fn allkinds() -> Kinds {
    HashSet::from([
        Kind::UserStep,
        Kind::GPXWaypoints,
        Kind::Cities,
        Kind::Villages,
        Kind::Controls,
        Kind::Mountains,
        Kind::Hamlets,
    ])
}

#[derive(Clone)]
pub struct PointCollection {
    pub map: BTreeMap<Kind, Vec<InputPoint>>,
}

pub type Packets = Vec<Vec<InputPoint>>;

impl PointCollection {
    pub fn new() -> Self {
        PointCollection {
            map: BTreeMap::new(),
        }
    }

    fn push(&mut self, point: InputPoint) {
        let otype = point.kind();
        self.map.entry(otype).or_default().push(point);
    }

    fn set_vector(&mut self, points: Vec<InputPoint>) {
        if points.is_empty() {
            return;
        }
        let otype = points.first().unwrap().kind();
        self.map.insert(otype, points);
    }

    pub fn get_vector(&self, otype: &Kind) -> Vec<InputPoint> {
        match self.map.get(&otype) {
            Some(vector) => vector.clone(),
            None => Vec::new(),
        }
    }

    pub fn potential_controls(&self) -> Vec<InputPoint> {
        let mut ret = Vec::new();
        ret.extend_from_slice(&self.get_vector(&Kind::Cities));
        ret.extend_from_slice(&self.get_vector(&Kind::Villages));
        ret
    }

    pub fn import_osm(&mut self, points: &Vec<InputPoint>) {
        let empty = Vec::new();
        self.map.insert(Kind::Cities, empty.clone());
        self.map.insert(Kind::Hamlets, empty.clone());
        self.map.insert(Kind::Mountains, empty.clone());
        self.map.insert(Kind::Villages, empty.clone());

        for k in 0..points.len() {
            let wi = points[k].clone();
            // insert also offtrack cities
            if wi.kind() == Kind::Cities || is_close_to_track(&wi) {
                self.push(wi);
            }
        }
        sort_by_elevation(&mut self.map.get_mut(&Kind::Mountains).unwrap());
        sort_by_population(&mut self.map.get_mut(&Kind::Cities).unwrap());
        sort_by_population(&mut self.map.get_mut(&Kind::Villages).unwrap());
        sort_by_population(&mut self.map.get_mut(&Kind::Hamlets).unwrap());
    }

    pub fn import_other(&mut self, kind: &Kind, points: Vec<InputPoint>) {
        self.map.insert(kind.clone(), Vec::new());
        if !points.is_empty() {
            assert!(points.first().unwrap().kind() == *kind);
        }
        self.set_vector(points);
    }

    fn ontrack_cities(&self) -> Vec<InputPoint> {
        let mut cities = self.get_vector(&Kind::Cities);
        cities.retain(|w| is_close_to_track(&w));
        sort_by_population(&mut cities);
        cities
    }

    fn offtrack_cities(&self) -> Vec<InputPoint> {
        let mut cities = self.get_vector(&Kind::Cities);
        cities.retain(|w| !is_close_to_track(&w));
        //sort_by_distance_to_track(&mut cities);
        sort_by_population(&mut cities);
        cities
    }

    pub fn range_cut(&mut self, range: &std::ops::Range<usize>) {
        self.map
            .iter_mut()
            .for_each(|(_key, points)| points.retain(|point| point.is_in_range(range)));
    }

    pub fn kinds_cut(&mut self, kinds: &Kinds) {
        self.map
            .iter_mut()
            .for_each(|(_key, points)| points.retain(|point| kinds.contains(&point.kind())));
    }

    pub fn profile(&self) -> Packets {
        let clone = self.clone();
        vec![
            clone.get_vector(&Kind::UserStep),
            clone.get_vector(&Kind::Controls),
            //clone.get_vector(&Kind::GPXWaypoints),
            clone.ontrack_cities(),
            clone.get_vector(&Kind::Villages),
            clone.get_vector(&Kind::Mountains),
            clone.get_vector(&Kind::Hamlets),
        ]
    }

    pub fn map(&self) -> Packets {
        let clone = self.clone();
        vec![
            clone.get_vector(&Kind::UserStep),
            clone.get_vector(&Kind::Controls),
            //clone.get_vector(&Kind::GPXWaypoints),
            clone.ontrack_cities(),
            clone.get_vector(&Kind::Villages),
            clone.get_vector(&Kind::Mountains),
            clone.offtrack_cities(),
            clone.get_vector(&Kind::Hamlets),
        ]
    }
}
