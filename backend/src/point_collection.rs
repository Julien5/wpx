use std::collections::{BTreeMap, BTreeSet, HashSet};

use crate::{
    inputpoint::InputPoint,
    label_placement::features::PointFeature,
    math::IntegerSize2D,
    parameters::{Parameters, RenderFunction, UserStepsOptions},
    track::Track,
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
    pub parameters: RenderInputParameters,
}

impl RenderResult {
    pub fn rendered_input_points(&self) -> Vec<InputPoint> {
        self.rendered
            .iter()
            .filter(|f| f.input_point.is_some())
            .map(|f| f.input_point().unwrap().clone())
            .collect()
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
        let rmap = map.rendered_projections();
        let rprofile = profile.rendered_projections();
        let mut good = BTreeSet::new();
        for (id1, rendered1) in &rmap {
            if let Some(rendered2) = rprofile.get(id1) {
                if *rendered1 == *rendered2 {
                    good.insert(id1);
                }
            }
        }
        for (id, _) in &rmap {
            if !good.contains(id) {
                log::trace!("intersection discarded from map:{}", id);
            }
        }
        for (id, _) in &rprofile {
            if !good.contains(id) {
                log::trace!("intersection discarded from profile:{}", id);
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

        // add the features that where not associated to input points
        // => time labels (9h, etc. in the profile)
        for m in &map.rendered {
            if m.input_point.is_none() {
                common_features_map.push(m.clone());
            }
        }
        for m in &profile.rendered {
            if m.input_point.is_none() {
                common_features_profile.push(m.clone());
            }
        }

        let m = RenderResult {
            svg: String::new(),
            rendered: common_features_map,
            parameters: map.parameters.clone(),
        };
        let p = RenderResult {
            svg: String::new(),
            rendered: common_features_profile,
            parameters: profile.parameters.clone(),
        };
        (m, p)
    }
}

#[derive(Clone, Default)]
pub struct RenderInputParameters {
    pub function: RenderFunction,
    pub parameters: Parameters,
    pub drange: std::ops::Range<f64>,
    pub range: std::ops::Range<usize>,
    pub screen_size: IntegerSize2D,
    pub usersteps: Vec<InputPoint>,
}

impl std::fmt::Debug for RenderInputParameters {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RenderInputParameters")
            .field("function", &self.function)
            .field("from", &self.drange.start)
            .field("end", &self.drange.end)
            .field("width", &self.screen_size.width)
            .field("height", &self.screen_size.width)
            .field("|usersteps|", &self.usersteps.len())
            .finish()
    }
}

fn only_usersteps_parameter_may_differ(p1: &Parameters, p2: &Parameters) -> bool {
    let mut c1 = p1.clone();
    let mut c2 = p2.clone();
    c1.user_steps_options = UserStepsOptions::default();
    c2.user_steps_options = UserStepsOptions::default();
    return c1 == c2;
}

impl RenderInputParameters {
    pub fn make_map_parameters(
        parameters: &Parameters,
        size: &IntegerSize2D,
        track: &Track,
        start: f64,
        end: f64,
        usersteps: &Vec<InputPoint>,
    ) -> Self {
        Self {
            function: RenderFunction::Map,
            parameters: parameters.clone(),
            drange: std::ops::Range {
                start: start,
                end: end,
            },
            range: track.subrange(start, end),
            screen_size: size.clone(),
            usersteps: usersteps.clone(),
        }
    }

    pub fn make_profile_parameters(
        parameters: &Parameters,
        size: &IntegerSize2D,
        track: &Track,
        start: f64,
        end: f64,
        usersteps: &Vec<InputPoint>,
    ) -> Self {
        Self {
            function: RenderFunction::Profile,
            parameters: parameters.clone(),
            drange: std::ops::Range {
                start: start,
                end: end,
            },
            range: track.subrange(start, end),
            screen_size: size.clone(),
            usersteps: usersteps.clone(),
        }
    }

    pub fn mismatch(&self, other: &Self) -> String {
        if self.function != other.function {
            return format!(
                "function mismatch ({:?} != {:?})",
                self.function, other.function
            );
        }
        if self.screen_size != other.screen_size {
            return format!(
                "screen size width ({:?} != {:?})",
                self.screen_size, other.screen_size
            );
        }
        if !only_usersteps_parameter_may_differ(&self.parameters, &other.parameters) {
            return format!(
                "parameter mismatch (other than user steps) {:?} != {:?}",
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
struct CachedResults {
    results: Vec<RenderResult>,
}

// TODO: limit the cache size ?
impl CachedResults {
    pub fn new() -> Self {
        Self {
            results: Vec::new(),
        }
    }
    pub fn push(&mut self, result: RenderResult) {
        self.results.push(result);
    }
    pub fn hit(&self, parameters: &RenderInputParameters) -> Option<RenderResult> {
        for result in &self.results {
            let mismatch = result.parameters.mismatch(parameters);
            if mismatch.is_empty() {
                log::info!("cache hit: {:?}", parameters);
                return Some(result.clone());
            } else {
                /*
                    log::trace!(
                        "cache mismatch with parameters: {:?} ({})",
                        parameters,
                        mismatch
                );
                    */
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
    pub fn register_result(&mut self, result: &RenderResult) {
        assert!(self.results.hit(&result.parameters).is_none());
        self.results.push(result.clone());
    }

    pub fn hit(&self, p: &RenderInputParameters) -> bool {
        self.results.hit(&p).is_some()
    }

    pub fn load(&self, p: &RenderInputParameters) -> RenderResult {
        match self.results.hit(&p) {
            Some(result) => result,
            None => {
                panic!("cache mismatch for parameters: {:?}", p);
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

    pub fn get_vector(&self, kind: &Kind) -> Vec<InputPoint> {
        let ret = match self.map.get(&kind) {
            Some(vector) => vector.clone(),
            None => Vec::new(),
        };
        ret
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
