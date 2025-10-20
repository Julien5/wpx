use geo::LineLocatePoint;

use crate::inputpoint::{InputPoint, InputPointMap, InputType, TrackProjection};
use crate::mercator::{EuclideanBoundingBox, MercatorPoint};
use crate::parameters::Parameters;
use crate::track::{self, Track};
use crate::{bboxes, gpsdata, locate, profile, svgmap};

#[derive(Clone)]
pub struct Segment {
    pub id: usize,
    pub range: std::ops::Range<usize>,
    pub profile_bbox: gpsdata::ProfileBoundingBox,
    pub map_bbox: EuclideanBoundingBox,
    pub track_tree: locate::Locate,
    pub track: std::sync::Arc<Track>,
    pub points: Vec<InputPoint>,
}

pub struct SegmentStatistics {
    pub length: f64,
    pub elevation_gain: f64,
    pub distance_start: f64,
    pub distance_end: f64,
}

impl Segment {
    pub fn new(
        id: usize,
        range: std::ops::Range<usize>,
        bbox: &gpsdata::ProfileBoundingBox,
        mbbox: &EuclideanBoundingBox,
        track_tree: locate::Locate,
        track: std::sync::Arc<Track>,
        inputpoints: &InputPointMap,
    ) -> Segment {
        let points = Self::copy_segment_points(inputpoints, mbbox, &track, &track_tree);
        Segment {
            id,
            range: range.clone(),
            profile_bbox: bbox.clone(),
            map_bbox: mbbox.clone(),
            track_tree: track_tree,
            track,
            points,
        }
    }

    pub fn render_profile(&self, (width, height): (i32, i32), parameters: &Parameters) -> String {
        log::info!("render profile:{}", self.id);
        let points = self.profile_points();
        let ret = profile::profile(
            &self.track,
            &points,
            &self,
            &parameters.profile_options,
            width,
            height,
            parameters.debug,
        );
        if parameters.debug {
            let filename = std::format!("/tmp/profile-{}.svg", self.id);
            std::fs::write(filename, &ret).expect("Unable to write file");
        }
        ret
    }

    fn middle_point(a: &(f64, f64, f64), b: &(f64, f64, f64), alpha: f64) -> (f64, f64, f64) {
        let ab = (b.0 - a.0, b.1 - a.1, b.2 - a.2);
        (a.0 + alpha * ab.0, a.1 + alpha * ab.1, a.2 + alpha * ab.2)
    }

    fn two_closest_index(track: &track::Track, index: &usize, p: &InputPoint) -> (usize, usize) {
        let tracklen = track.euclidian.len();
        if *index == 0 {
            return (0, 1);
        }
        if *index == tracklen - 1 {
            return (index - 1, *index);
        }
        let dbefore = p.euclidian.d2(&track.euclidian[index - 1]);
        let dafter = p.euclidian.d2(&track.euclidian[index - 1]);
        if dbefore < dafter {
            (index - 1, *index)
        } else {
            (*index, *index + 1)
        }
    }

    fn compute_relation(track: &track::Track, tracktree: &locate::Locate, point: &mut InputPoint) {
        let index = tracktree.nearest_neighbor(&point.euclidian).unwrap();
        let (index1, index2) = Self::two_closest_index(track, &index, point);
        let p1 = &track.euclidian[index1];
        let p2 = &track.euclidian[index2];
        let linestring: geo::LineString = vec![p1.xy(), p2.xy()].into();
        let findex = linestring
            .line_locate_point(&geo::point!(point.euclidian.xy()))
            .unwrap();
        assert!(findex <= 1f64);
        let rindex = index1 as f64 + findex;
        let t1 = &track.euclidian[index1];
        let t2 = &track.euclidian[index2];
        let a1 = (t1.0, t1.1, track.elevation(index1));
        let a2 = (t2.0, t2.1, track.elevation(index2));
        let m = Self::middle_point(&a1, &a2, findex);
        let euclidean = MercatorPoint::from_xy(&(m.0, m.1));
        let elevation = m.2;
        let track_distance = euclidean.d2(&point.euclidian).sqrt();

        // check
        let di = point.euclidian.d2(&track.euclidian[index]);
        let df = point.euclidian.d2(&euclidean);
        debug_assert!(df <= di);

        point.track_projection = Some(TrackProjection {
            track_index: rindex,
            euclidean,
            elevation,
            track_distance,
        });
    }

    fn copy_segment_points(
        inputpoints: &InputPointMap,
        _bbox: &EuclideanBoundingBox,
        track: &track::Track,
        tracktree: &locate::Locate,
    ) -> Vec<InputPoint> {
        let mut ret = Vec::new();
        let mut bbox = _bbox.clone();
        // a bit too much enlarging, probably.
        bbox.enlarge(&5000f64);
        let bboxs = bboxes::split(&bbox, &bboxes::BBOXWIDTH);
        for (_index, bbox) in bboxs {
            let _points = inputpoints.get(&bbox);
            if _points.is_none() {
                continue;
            }
            let points = _points.unwrap();
            for p in points {
                let mut c = p.clone();
                Self::compute_relation(track, tracktree, &mut c);
                ret.push(c);
            }
        }
        ret
    }

    fn distance_to_track(&self, p: &InputPoint) -> f64 {
        match &p.track_projection {
            Some(proj) => proj.track_distance,
            None => {
                assert!(false);
                f64::MAX
            }
        }
    }

    fn important(p: &InputPoint) -> bool {
        let pop = match p.population() {
            Some(n) => n,
            None => {
                if p.kind() == InputType::City {
                    1000
                } else {
                    0
                }
            }
        };
        let dist = p.distance_to_track();
        if pop > 100000 && dist < 5000f64 {
            return true;
        }
        if pop > 10000 && dist < 1000f64 {
            return true;
        }
        if pop >= 500 && dist < 500f64 {
            return true;
        }
        /*if dist < 2000f64 {
            log::trace!(
                "too far for the profile:{:?} {:?} {:?} d={:.1}",
                p.kind(),
                p.population(),
                p.name(),
                dist
            );
        }*/
        false
    }

    pub fn profile_points(&self) -> Vec<InputPoint> {
        let mut ret = self.points.clone();
        ret.retain(|p| {
            let distance = self.distance_to_track(&p);
            match p.kind() {
                InputType::MountainPass | InputType::Peak => {
                    return distance < 250f64;
                }
                InputType::Hamlet => {
                    return false;
                }
                InputType::GPX => {
                    return distance < 250f64;
                }
                InputType::City | InputType::Village => {
                    return Self::important(p);
                }
            }
        });
        for w in &mut ret {
            w.label_placement_order = Self::placement_order_profile(&w);
        }
        ret
    }

    pub fn render_map(&self, (width, height): (i32, i32), debug: bool) -> String {
        log::info!("render map:{}", self.id);
        let points = self.map_points();
        let ret = svgmap::map(&self.track, &points, &self, width, height, debug);
        if debug {
            let filename = std::format!("/tmp/map-{}.svg", self.id);
            std::fs::write(filename, &ret).expect("Unable to write file");
        }
        ret
    }

    fn placement_order_profile(point: &InputPoint) -> i32 {
        let delta = point.distance_to_track();
        let kind = point.kind();
        let mut ret = 1;
        if kind == InputType::City && delta < 1000f64 {
            return ret;
        }
        if (kind == InputType::MountainPass || kind == InputType::Peak) && delta < 500f64 {
            return ret;
        }
        ret += 1;
        if kind == InputType::Village && delta < 1000f64 {
            return ret;
        }
        ret += 1;
        if kind == InputType::City && delta < 10000f64 {
            return ret;
        }
        ret += 1;
        if kind == InputType::Village && delta < 200f64 {
            return ret;
        }
        ret += 1;
        ret
    }

    fn placement_order_map(point: &InputPoint) -> i32 {
        if point.name().is_none() {
            return i32::MAX - 10;
        }
        match point.kind() {
            InputType::Hamlet => {
                return 5;
            }
            InputType::Village => {
                return 4;
            }
            InputType::MountainPass | InputType::Peak => {
                return 3;
            }
            InputType::GPX => {
                return 2;
            }
            InputType::City => {
                return 1;
            }
        }
    }

    fn map_points(&self) -> Vec<InputPoint> {
        let profile = self.profile_points();
        // at most 15 points on the map.
        let nextra = if profile.len() >= 15 {
            0
        } else {
            (15 - profile.len()).max(0)
        };
        if nextra == 0 {
            return profile.clone();
        }
        let mut extra = self.points.clone();
        extra.retain(|p| {
            if profile.contains(&p) {
                return false;
            }
            if !self.map_bbox.contains(&p.euclidian.xy()) {
                return false;
            }
            true
        });
        extra.sort_by_key(|p| Self::placement_order_map(&p));
        extra.truncate(nextra);
        let mut ret = profile.clone();
        ret.extend_from_slice(&extra);
        for w in &mut ret {
            if profile.contains(&w) {
                w.label_placement_order = Self::placement_order_profile(&w);
            } else {
                w.label_placement_order = Self::placement_order_map(&w) + 5;
            }
        }
        ret
    }
}
