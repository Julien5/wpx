use std::collections::{BTreeMap, BTreeSet, HashSet};

use crate::{
    inputpoint::InputPoint,
    label_placement::features::PointFeature,
    math::IntegerSize2D,
    parameters::{Parameters, RenderFunction, UserStepsOptions},
    track_projection::is_close_to_track,
};

fn sort_by_elevation(mountains: &mut Vec<InputPoint>) {
    mountains.sort_by_key(|w| std::cmp::Reverse(w.ele().unwrap_or(0f64).floor() as i32));
}

fn sort_by_population(cities: &mut Vec<InputPoint>) {
    cities.sort_by_key(|w| std::cmp::Reverse(w.population().unwrap_or(0)));
}

#[derive(Clone, Debug, Default)]
pub struct RenderResult {
    pub svg: String,
    pub rendered: Vec<PointFeature>,
}

impl RenderResult {
    pub fn rendered_input_points(&self) -> Vec<InputPoint> {
        self.rendered
            .iter()
            .filter(|f| f.input_point.is_some())
            .map(|f| f.input_point().unwrap().clone())
            .collect()
    }

    pub fn debug(&self) {
        for p in &self.rendered {
            log::trace!("rendered: {}", p.id());
        }
    }

    fn features_with_point_id(&self, id: &String) -> Vec<PointFeature> {
        self.rendered
            .iter()
            .filter(|f| f.input_point.is_some())
            .filter(|f| f.input_point().unwrap().id() == *id)
            .map(|f| f.clone())
            .collect()
    }

    // input_point.id() -> set of rendered projections
    fn rendered_projections(&self) -> BTreeMap<String, BTreeSet<usize>> {
        let mut rendered_projections: BTreeMap<String, BTreeSet<usize>> = BTreeMap::new();

        for feature in &self.rendered {
            if let Some(point) = &feature.input_point {
                rendered_projections
                    .entry(point.0.id())
                    .or_default()
                    .extend(&point.1);
            }
        }
        rendered_projections
    }

    pub fn intersection(map: &Self, profile: &Self) -> (Self, Self) {
        let r1 = map.rendered_projections();
        let r2 = profile.rendered_projections();
        let mut good = BTreeSet::new();
        for (id1, rendered1) in r1 {
            if let Some(rendered2) = r2.get(&id1) {
                if rendered1 == *rendered2 {
                    good.insert(id1);
                }
            }
        }
        log::trace!("number of common input points:{}", good.len());
        let mut common_features_map = Vec::new();
        let mut common_features_profile = Vec::new();
        for id in good {
            let maps = map.features_with_point_id(&id);
            common_features_map.extend_from_slice(&maps);
            let profiles = profile.features_with_point_id(&id);
            common_features_profile.extend_from_slice(&profiles);
        }
        let m = RenderResult {
            svg: String::new(),
            rendered: common_features_map,
        };
        let p = RenderResult {
            svg: String::new(),
            rendered: common_features_profile,
        };
        (m, p)
    }
}

#[derive(Clone, Debug)]
struct RenderInputParameters {
    parameters: Parameters,
    range: std::ops::Range<usize>,
    screen_size: IntegerSize2D,
    controls: Vec<InputPoint>,
}

fn only_usersteps_parameter_may_differ(p1: &Parameters, p2: &Parameters) -> bool {
    let mut c1 = p1.clone();
    let mut c2 = p2.clone();
    c1.user_steps_options = UserStepsOptions::default();
    c2.user_steps_options = UserStepsOptions::default();
    return c1 == c2;
}

impl RenderInputParameters {
    pub fn mismatch(&self, other: &Self, _function: &RenderFunction) -> String {
        if self.screen_size.width != other.screen_size.width {
            return format!(
                "screen size width ({} != {})",
                self.screen_size.width, other.screen_size.width
            );
        }
        if self.screen_size.height != other.screen_size.height {
            return format!(
                "screen size height ({} != {})",
                self.screen_size.height, other.screen_size.height
            );
        }
        if self.controls != other.controls {
            return format!("controls have changed");
        }
        if !only_usersteps_parameter_may_differ(&self.parameters, &other.parameters) {
            return format!(
                "parameter mismatch (more than user steps) {:?} != {:?}",
                self.parameters, other.parameters
            );
        }
        if self.range != other.range {
            return format!("range mismatch {:?} != {:?}", self.range, other.range);
        }
        String::new()
    }
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

            let mismatch = result.parameters.mismatch(parameters, function);
            if mismatch.is_empty() {
                //log::info!("cache hit for function: {:?}", function);
                return Some(result.output.clone());
            } else {
                /*  log::info!(
                    "cache mismatch for function: {:?} and parameters: {}",
                    function,
                    mismatch
                );*/
            }
        }
        None
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
        // note: controls are the current controls in the collection,
        // not those rendered (result.rendered)
        let p = RenderInputParameters {
            range: range.clone(),
            screen_size: size.clone(),
            parameters: parameters.clone(),
            controls: self.collection.get_vector(&Kind::Controls).clone(),
        };
        let cresult = CachedResult {
            function: function.clone(),
            parameters: p.clone(),
            output: result.clone(),
        };
        if self.results.hit(function, &p).is_none() {
            self.results.push(cresult);
        }
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
            controls: self.collection.get_vector(&Kind::Controls).clone(),
        };
        self.results.hit(function, &p).is_some()
    }

    pub fn load(
        &self,
        function: &RenderFunction,
        range: &std::ops::Range<usize>,
        parameters: &Parameters,
        size: &IntegerSize2D,
    ) -> RenderResult {
        let p = RenderInputParameters {
            range: range.clone(),
            parameters: parameters.clone(),
            screen_size: size.clone(),
            controls: self.collection.get_vector(&Kind::Controls).clone(),
        };
        match self.results.hit(function, &p) {
            Some(result) => result,
            None => {
                panic!(
                    "packet provider cache miss for function: {:?}. This should not happen. Bye.",
                    function
                );
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

    pub fn debug(&self) {
        let r = self.get_vector(&Kind::UserStep);
        log::debug!("collection contains {} usersteps", r.len());
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

        for index in 0..points.len() {
            let wi = points[index].clone();
            if !is_osm(&wi.kind()) {
                continue;
            }
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

    pub fn intersect(&mut self, other: &Self) {
        let mut map = BTreeMap::new();
        for k in self.map.keys() {
            if !other.map.contains_key(&k) {
                continue;
            }
            let v1 = self.map.get(&k).unwrap();
            let v2 = other.map.get(&k).unwrap();
            let intersection: Vec<_> = v1
                .into_iter()
                .filter(|x| v2.contains(x))
                .map(|v| v.clone())
                .collect();
            log::trace!(
                "kind={:?} v1={} v2={} intersection={}",
                k,
                v1.len(),
                v2.len(),
                intersection.len()
            );
            map.insert(k.clone(), intersection);
        }
        self.map = map;
        sort_by_elevation(&mut self.map.get_mut(&Kind::Mountains).unwrap());
        sort_by_population(&mut self.map.get_mut(&Kind::Cities).unwrap());
        sort_by_population(&mut self.map.get_mut(&Kind::Villages).unwrap());
        sort_by_population(&mut self.map.get_mut(&Kind::Hamlets).unwrap());
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
        assert!(!clone.get_vector(&Kind::UserStep).is_empty());
        vec![
            clone.get_vector(&Kind::UserStep),
            clone.get_vector(&Kind::Controls),
            //clone.get_vector(&Kind::GPXWaypoints),
            clone.ontrack_cities(),
            clone.get_vector(&Kind::Villages),
            clone.get_vector(&Kind::Mountains),
            //clone.get_vector(&Kind::Hamlets),
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
            //clone.get_vector(&Kind::Hamlets),
        ]
    }
}
