use clap::ValueEnum;
use std::collections::{BTreeMap, HashSet};

use crate::{
    inputpoint::InputPoint,
    label_placement::features::PointFeature,
    math::IntegerSize2D,
    parameters::{Parameters, RenderFunction, UserStepsOptions},
    track::Track,
    track_projection::{is_close_to_track, TrackProjection, TrackProjections},
};

fn sort_by_elevation(mountains: &mut Vec<InputPoint>) {
    mountains.sort_by_key(|w| std::cmp::Reverse(w.ele().unwrap_or(0f64).floor() as i32));
}

fn sort_by_population(cities: &mut Vec<InputPoint>) {
    cities.sort_by_key(|w| std::cmp::Reverse(w.population().unwrap_or(0)));
}

#[allow(dead_code)]
fn sort_by_distance(cities: &mut Vec<InputPoint>) {
    cities.retain(|w| !w.track_projections.is_empty());
    cities.sort_by(|a, b| {
        a.distance_to_track()
            .partial_cmp(&b.distance_to_track())
            .unwrap()
    });
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

    pub fn packets_for_map(&self) -> Packets {
        let mut map: BTreeMap<usize, Vec<_>> = BTreeMap::new();
        let mut n1 = 0;
        for f in &self.rendered {
            if f.input_point.is_none() {
                continue;
            }
            map.entry(f.hardness).or_default().push(f.clone());
            n1 += 1;
        }
        let mut hardnesses: Vec<_> = map.keys().collect();
        // sort in descending order
        hardnesses.sort_by(|a, b| b.cmp(a));
        let mut ret = Vec::new();
        let mut n2 = 0;
        for hardness in hardnesses {
            let mut packet = Packet {
                hardness: *hardness,
                points: map
                    .get(&hardness)
                    .unwrap()
                    .into_iter()
                    .map(|f| f.input_point().unwrap().clone())
                    .collect(),
            };
            let mut seen = HashSet::new();
            packet.points.retain(|point| {
                if point.kind() == Kind::CutOff {
                    return false;
                }
                if is_osm(&point.kind()) {
                    return seen.insert(point.map_id());
                }
                true
            });
            n2 += packet.points.len();
            ret.push(packet);
        }
        debug_assert!(n2 <= n1);
        debug_assert!(n1 == 0 || n2 > 0);
        ret
    }

    pub fn rendered_input_points_for_table(&self) -> Vec<InputPoint> {
        let mut ret = self.rendered_input_points();
        ret.retain(|point| {
            if point.kind() == Kind::CutOff {
                return false;
            }
            true
        });
        ret
    }
}

#[derive(Clone, Default)]
pub struct RenderInputParameters {
    pub function: RenderFunction,
    pub kinds: Kinds,
    pub parameters: Parameters,
    pub drange: std::ops::Range<f64>,
    pub range: std::ops::Range<usize>,
    pub screen_size: IntegerSize2D,
    pub other_parameters_hash: Option<String>,
    pub background_points: Vec<Vec<InputPoint>>,
    pub usersteps: Vec<InputPoint>,
}

impl RenderInputParameters {
    pub fn hash(&self) -> String {
        let mut parameters = self.parameters.clone();
        // Parameters must be taken into account (because of start time and speed),
        // but the cache may be re-used for different user steps parameters because
        // user steps are rendered in the foreground.
        parameters.user_steps_options = UserStepsOptions::default();
        format!(
            "F={:?}-S={:?}-K={:?}-Rd={:?}-P={:?}-O={:?}-B={:?}",
            self.function,
            self.screen_size,
            self.kinds,
            self.drange,
            parameters,
            self.other_parameters_hash,
            self.background_points
        )
    }
}

impl std::fmt::Debug for RenderInputParameters {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RenderInputParameters")
            .field("function", &self.function)
            .field("kinds", &self.kinds)
            .field("from", &self.drange.start)
            .field("end", &self.drange.end)
            .field("width", &self.screen_size.width)
            .field("height", &self.screen_size.width)
            .field("other", &self.other_parameters_hash.is_some())
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
        kinds: &Kinds,
        parameters: &Parameters,
        size: &IntegerSize2D,
        track: &Track,
        start: f64,
        end: f64,
        background_points: &Vec<Vec<InputPoint>>,
        usersteps: &Vec<InputPoint>,
    ) -> Self {
        Self {
            function: RenderFunction::Map,
            kinds: kinds.clone(),
            parameters: parameters.clone(),
            drange: std::ops::Range {
                start: start,
                end: end,
            },
            range: track.subrange(start, end),
            screen_size: size.clone(),
            other_parameters_hash: None,
            background_points: background_points.clone(),
            usersteps: usersteps.clone(),
        }
    }

    pub fn make_profile_parameters(
        kinds: &Kinds,
        parameters: &Parameters,
        size: &IntegerSize2D,
        track: &Track,
        start: f64,
        end: f64,
        background_points: &Vec<Vec<InputPoint>>,
        usersteps: &Vec<InputPoint>,
    ) -> Self {
        Self {
            function: RenderFunction::Profile,
            kinds: kinds.clone(),
            parameters: parameters.clone(),
            drange: std::ops::Range {
                start: start,
                end: end,
            },
            range: track.subrange(start, end),
            screen_size: size.clone(),
            other_parameters_hash: None,
            background_points: background_points.clone(),
            usersteps: usersteps.clone(),
        }
    }

    pub fn mismatch(&self, other: &Self) -> String {
        if self.hash() == other.hash() {
            return String::new();
        }
        if self.kinds != other.kinds {
            return format!("kinds mismatch ({:?} != {:?})", self.kinds, other.kinds);
        }
        if self.function != other.function {
            return format!(
                "function mismatch ({:?} != {:?})",
                self.function, other.function
            );
        }
        if self.other_parameters_hash != other.other_parameters_hash {
            return format!(
                "other parameter mismatch ({:?} != {:?})",
                self.other_parameters_hash, other.other_parameters_hash
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
        if self.background_points != other.background_points {
            return format!(
                "background points mismatch ({:?} != {:?})",
                self.background_points, other.background_points
            );
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

    pub fn hit(&self, p: &RenderInputParameters) -> Option<RenderResult> {
        self.results.hit(&p)
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

#[derive(ValueEnum, Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Hash)]
#[value(rename_all = "PascalCase")]
pub enum Kind {
    Cities,
    Controls,
    #[value(name = "GPXWaypoints")]
    GPXWaypoints,
    Hamlets,
    Mountains,
    Villages,
    CutOff,
}

impl std::fmt::Display for Kind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

pub fn is_osm(kind: &Kind) -> bool {
    match kind {
        Kind::Controls | Kind::GPXWaypoints | Kind::CutOff => false,
        _ => true,
    }
}

pub type Kinds = HashSet<Kind>;

pub fn allkinds() -> Kinds {
    HashSet::from([
        Kind::CutOff,
        Kind::GPXWaypoints,
        Kind::Cities,
        Kind::Villages,
        Kind::Controls,
        Kind::Mountains,
        Kind::Hamlets,
    ])
}
pub fn onekind(kind: Kind) -> Kinds {
    HashSet::from([kind])
}

#[derive(Clone)]
pub struct PointCollection {
    pub map: BTreeMap<Kind, Vec<InputPoint>>,
}

pub struct Packet {
    pub points: Vec<InputPoint>,
    pub hardness: usize,
}

impl Packet {
    pub fn make_forced_packet(points: Vec<InputPoint>) -> Self {
        Self {
            hardness: 11,
            points,
        }
    }
}
pub type Packets = Vec<Packet>;

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
        ret.extend_from_slice(&self.get_vector(&Kind::Hamlets));
        ret.extend_from_slice(&self.get_vector(&Kind::Mountains));
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

    pub fn offtrack_cities(&self) -> Vec<InputPoint> {
        let mut cities = self.get_vector(&Kind::Cities);
        cities.retain(|w| !is_close_to_track(&w));
        //sort_by_distance(&mut cities);
        sort_by_population(&mut cities);
        cities.truncate(8);
        cities
    }

    fn controls(&self) -> Vec<InputPoint> {
        let controls = self.get_vector(&Kind::Controls);
        let mut ret = Vec::new();
        for w in controls {
            ret.push(w.clone());
        }
        ret
    }

    fn gpxwaypoints(&self) -> Vec<InputPoint> {
        let controls = self.get_vector(&Kind::Controls);
        let waypoints = self.get_vector(&Kind::GPXWaypoints);
        let mut ret = Vec::new();
        // filter out waypoints that are rendered as controls,
        // "projection-aware":
        // a waypoint may have two projections P1 and P2.
        // If only P1 is rendered as control, push P2.
        for w in waypoints {
            let origin_id = w.gpxwaypoint_id();
            let matching_control_projections: TrackProjections = controls
                .iter()
                .filter(|c| c.control_waypoint_origin_id() == origin_id)
                .map(|c| c.track_projections.clone())
                .flatten()
                .collect();
            for proj in &w.track_projections {
                if !projections_contains_fuzzy(&matching_control_projections, &proj) {
                    ret.push(w.clone_with_proj(proj));
                }
            }
        }
        ret
    }

    pub fn range_cut(&mut self, range: &std::ops::Range<usize>) {
        self.map
            .iter_mut()
            .for_each(|(_key, points)| points.retain(|point| point.is_in_range(range)));
    }

    pub fn kinds_cut(&mut self, kinds: &Kinds) {
        self.map.retain(|kind, _points| kinds.contains(kind));
    }

    pub fn osm_packet(points: Vec<InputPoint>) -> Packet {
        // Quite brutal but seems to give reasonable results.
        // Packet with 1 point => hardness = 9
        //      with 10 points => hardness = 0
        let hardness = 10 - points.len().min(10);
        Packet { hardness, points }
    }

    pub fn profile(&self) -> Packets {
        let clone = self.clone();
        vec![
            Packet::make_forced_packet(clone.controls()),
            Packet::make_forced_packet(clone.gpxwaypoints()),
            Packet {
                hardness: 0,
                points: clone.get_vector(&Kind::CutOff),
            },
            Self::osm_packet(clone.ontrack_cities()),
            Self::osm_packet(clone.get_vector(&Kind::Villages)),
            Self::osm_packet(clone.get_vector(&Kind::Mountains)),
            //clone.get_vector(&Kind::Hamlets),
        ]
    }

    pub fn map(&self) -> Packets {
        let clone = self.clone();
        vec![
            Packet::make_forced_packet(clone.controls()),
            Packet::make_forced_packet(clone.gpxwaypoints()),
            Packet {
                hardness: 0,
                points: clone.get_vector(&Kind::CutOff),
            },
            Self::osm_packet(clone.ontrack_cities()),
            Self::osm_packet(clone.get_vector(&Kind::Villages)),
            Self::osm_packet(clone.get_vector(&Kind::Mountains)),
            Self::osm_packet(clone.offtrack_cities()),
            //clone.get_vector(&Kind::Hamlets),
        ]
    }
}

fn projections_contains_fuzzy(
    control_projections: &TrackProjections,
    waypoint_projection: &TrackProjection,
) -> bool {
    for proj in control_projections {
        // Consider the following situation:
        //  C1,C2----+
        //           |
        //           | W
        // ------------------
        // C1 is a control. Segment  1 goes to C1. Segment  2 start from C1.
        // C2 is a control. Segment 10 goes to C1. Segment 11 start from C2.
        // W is the waypoint, it is associated to C2, but not to C1.
        // Considering only d_geo would associate W to C1 and C2.
        // => we must consider also the distance along the track, but it can
        // be "a bit large", depending on the distance along the track between
        // W and C1 or C2.
        let d_geo = proj
            .euclidean
            .point2d()
            .distance_to(&waypoint_projection.euclidean.point2d());

        let d_ontrack = (proj.distance_on_track_to_projection
            - waypoint_projection.distance_on_track_to_projection)
            .abs();

        if d_geo < 300.0 && d_ontrack < 1000.0 {
            return true;
        }
    }
    false
}
