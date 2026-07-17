use crate::{
    backend::Segment,
    inputpoint::InputPointData::OSM,
    mercator::DateTime,
    speed::{self, TimeParameters},
};
use clap::ValueEnum;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashSet};

use crate::{
    inputpoint::InputPoint,
    label_placement::features::PointFeature,
    math::IntegerSize2D,
    parameters::{Parameters, RenderFunction},
    track::Track,
    track_projection::{TrackProjection, TrackProjections},
};

fn sort_by_elevation(mountains: &mut Vec<InputPoint>) {
    mountains.sort_by_key(|w| {
        let elevation = match &w.data {
            OSM(d) => d.elevation(),
            _ => 0f64,
        };
        std::cmp::Reverse(elevation.floor() as i32)
    });
}

fn sort_by_population(cities: &mut Vec<InputPoint>) {
    cities.sort_by_key(|w| {
        let population = match &w.data {
            OSM(d) => d.population(),
            _ => 0,
        };
        std::cmp::Reverse(population)
    });
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
                    let good = seen.insert(point.map_id());
                    return good;
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
    pub time_parameters: TimeParameters,
    pub drange: std::ops::Range<f64>,
    pub range: std::ops::Range<usize>,
    pub screen_size: IntegerSize2D,
    pub background_points: Vec<Vec<InputPoint>>,
    pub usersteps: Vec<InputPoint>,
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
            .field("|usersteps|", &self.usersteps.len())
            .finish()
    }
}

impl RenderInputParameters {
    pub fn make_map_parameters(
        kinds: &Kinds,
        parameters: &Parameters,
        time_parameters: &TimeParameters,
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
            time_parameters: time_parameters.clone(),
            drange: std::ops::Range {
                start: start,
                end: end,
            },
            range: track.subrange(start, end),
            screen_size: size.clone(),
            background_points: background_points.clone(),
            usersteps: usersteps.clone(),
        }
    }

    pub fn make_profile_parameters(
        kinds: &Kinds,
        parameters: &Parameters,
        time_parameters: &TimeParameters,
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
            time_parameters: time_parameters.clone(),
            drange: std::ops::Range {
                start: start,
                end: end,
            },
            range: track.subrange(start, end),
            screen_size: size.clone(),
            background_points: background_points.clone(),
            usersteps: usersteps.clone(),
        }
    }
}

pub struct PacketProvider {
    pub collection: PointCollection,
}

impl PacketProvider {
    pub fn new() -> Self {
        Self {
            collection: PointCollection::new(),
        }
    }
}

#[derive(ValueEnum, Debug, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Clone, Hash)]
#[value(rename_all = "PascalCase")]
#[serde(rename_all = "snake_case")]
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

pub type Kinds = Vec<Kind>;

pub fn allkinds() -> Kinds {
    Vec::from([
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
    Vec::from([kind])
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

    fn set_vector(&mut self, mut points: Vec<InputPoint>) {
        if points.is_empty() {
            return;
        }
        let otype = points.first().unwrap().kind();
        for (index, point) in points.iter_mut().enumerate() {
            point.index = Some(index);
        }
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
            if wi.kind() == Kind::Cities || wi.is_close_to_track() {
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
            debug_assert!(points.first().unwrap().kind() == *kind);
        }
        self.set_vector(points);
    }

    fn ontrack_cities(&self) -> Vec<InputPoint> {
        let mut cities = self.get_vector(&Kind::Cities);
        cities.retain(|w| w.is_close_to_track());
        sort_by_population(&mut cities);
        cities
    }

    pub fn offtrack_cities(&self) -> Vec<InputPoint> {
        let mut cities = self.get_vector(&Kind::Cities);
        cities.retain(|w| !w.is_close_to_track());
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
        remove_control_waypoints(&waypoints, &controls)
    }

    pub fn range_cut(&mut self, segment: &Segment) {
        self.map.iter_mut().for_each(|(_key, points)| {
            points.retain(|point| point.is_on_segment(segment.start, segment.end))
        });
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

    pub fn from_result(profile: &RenderResult) -> Self {
        let mut map: BTreeMap<Kind, Vec<InputPoint>> = BTreeMap::new();
        for point in profile.rendered_input_points() {
            map.entry(point.kind()).or_default().push(point.clone());
        }
        Self { map }
    }

    pub fn profile(&self, segment: &Segment, kinds: &Kinds) -> Packets {
        let mut clone = self.clone();
        clone.range_cut(segment);
        clone.kinds_cut(kinds);
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
            // Self::osm_packet(clone.get_vector(&Kind::Hamlets)),
        ]
    }

    pub fn map(&self, segment: &Segment, kinds: &Kinds) -> Packets {
        let mut clone = self.clone();
        clone.range_cut(segment);
        clone.kinds_cut(kinds);
        vec![
            Packet::make_forced_packet(clone.controls()),
            Packet::make_forced_packet(clone.gpxwaypoints()),
            /* exclude cutoff points from the map
             * Packet {
             *    hardness: 0,
             *    points: clone.get_vector(&Kind::CutOff),
             * },
             */
            Self::osm_packet(clone.ontrack_cities()),
            Self::osm_packet(clone.get_vector(&Kind::Villages)),
            Self::osm_packet(clone.get_vector(&Kind::Mountains)),
            Self::osm_packet(clone.offtrack_cities()),
            // Self::osm_packet(clone.get_vector(&Kind::Hamlets)),
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

pub fn remove_control_waypoints(
    waypoints: &Vec<InputPoint>,
    controls: &Vec<InputPoint>,
) -> Vec<InputPoint> {
    let mut ret = Vec::new();
    // filter out waypoints that are rendered as controls,
    // "projection-aware":
    // a waypoint may have two projections P1 and P2.
    // If only P1 is rendered as control, push P2.
    for w in waypoints {
        let index = w.index();
        let matching_control_projections: TrackProjections = controls
            .iter()
            .filter(|c| c.control_waypoint_origin_index() == index)
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

fn control_speed_data(start_time: &DateTime, control: &InputPoint) -> speed::InterpolationPoint {
    let distance = control
        .track_projections
        .first()
        .unwrap()
        .distance_on_track_to_projection;
    let cdata = control.data.as_control().unwrap();
    let time = cdata.cutoff_time.clone();
    let is_end = cdata.is_end();
    let duration = match time {
        Some(t) => Some(t - start_time),
        None => None,
    };
    speed::InterpolationPoint {
        distance,
        duration,
        is_end,
    }
}

pub fn controls_speed_data(
    start_time: &DateTime,
    controls: &Vec<InputPoint>,
) -> Vec<speed::InterpolationPoint> {
    let mut ret: Vec<_> = controls
        .iter()
        .map(|c| control_speed_data(start_time, c))
        .collect();
    ret.sort_by(|a, b| a.distance.partial_cmp(&b.distance).unwrap());
    ret
}
