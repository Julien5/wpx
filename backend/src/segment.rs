use crate::gpsdata::distance_wgs84;
use crate::inputpoint::{InputPoint, InputPointMap, InputType};
use crate::mercator::EuclideanBoundingBox;
use crate::track::Track;
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
        let points = Self::copy_segment_points(inputpoints, mbbox, &track_tree);
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
    pub fn render_profile(&self, (width, height): (i32, i32), debug: bool) -> String {
        log::info!("render profile:{}", self.id);
        let points = self.profile_points();
        let ret = profile::profile(&self.track, &points, &self, width, height, debug);
        if debug {
            let filename = std::format!("/tmp/profile-{}.svg", self.id);
            std::fs::write(filename, &ret).expect("Unable to write file");
        }
        ret
    }
    fn copy_segment_points(
        inputpoints: &InputPointMap,
        _bbox: &EuclideanBoundingBox,
        tracktree: &locate::Locate,
    ) -> Vec<InputPoint> {
        let mut ret = Vec::new();
        let mut bbox = _bbox.clone();
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
                let index = tracktree.nearest_neighbor(&c.euclidian);
                assert!(index.is_some());
                c.track_index = index;
                assert!(c.track_index.is_some());
                ret.push(c);
            }
        }
        ret
    }
    pub fn profile_points(&self) -> Vec<InputPoint> {
        let mut ret = self.points.clone();
        ret.retain(|p| {
            assert!(p.track_index.is_some());
            // use only if there are no other points shown
            if p.kind() == InputType::Hamlet {
                return false;
            }
            if p.kind() == InputType::GPX {
                return true;
            }
            let mut distance = f64::MAX;
            match p.track_index {
                Some(index) => {
                    let ptrack = &self.track.wgs84[index];
                    distance = distance_wgs84(ptrack, &p.wgs84);
                }
                None => {
                    assert!(false);
                }
            }
            if p.population().is_some() {
                let pop = p.population().unwrap();
                if pop > 100000 && distance < 5000f64 {
                    return true;
                }
                if pop > 10000 && distance < 1000f64 {
                    return true;
                }
                if pop > 1000 && distance < 500f64 {
                    return true;
                }
            }
            if distance < 200f64 {
                return true;
            }
            // log::trace!("too far:{:?} d={:.1}", p.name(), distance);
            false
        });
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
    fn map_points(&self) -> Vec<InputPoint> {
        let mut ret = self.profile_points();
        for p in &self.points {
            if ret.contains(&p) {
                continue;
            }
            assert!(p.track_index.is_some());
            if p.kind() == InputType::City {
                ret.push(p.clone());
                continue;
            }
        }

        ret
    }
}
