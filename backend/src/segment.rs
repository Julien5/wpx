use geo::LineLocatePoint;

use crate::bbox::BoundingBox;
use crate::inputpoint::{InputPoint, InputPointMaps, TrackProjection};
use crate::math::{IntegerSize2D, Point2D};
use crate::mercator::MercatorPoint;
use crate::parameters::{Parameters, ProfileIndication};
use crate::profile::ProfileRenderResult;
use crate::track::{self, Track};
use crate::{bboxes, locate, profile, svgmap};

#[derive(Clone)]
pub struct Segment {
    pub id: i32,
    pub range: std::ops::Range<usize>,
    pub track_tree: locate::IndexedPointsTree,
    pub track: std::sync::Arc<Track>,
    pub points: Vec<InputPoint>,
    pub parameters: Parameters,
}

pub struct SegmentStatistics {
    pub length: f64,
    pub elevation_gain: f64,
    pub distance_start: f64,
    pub distance_end: f64,
}

impl Segment {
    pub fn new(
        id: i32,
        range: std::ops::Range<usize>,
        track_tree: locate::IndexedPointsTree,
        track: std::sync::Arc<Track>,
        inputpoints: &InputPointMaps,
        parameters: &Parameters,
    ) -> Segment {
        let map_box =
            svgmap::euclidean_bounding_box(&track, &range, &parameters.map_options.size2d());
        let points = Self::copy_segment_points(inputpoints, &map_box, &track, &track_tree);
        Segment {
            id,
            range: range.clone(),
            track_tree: track_tree,
            track,
            points,
            parameters: parameters.clone(),
        }
    }

    pub fn set_profile_indication(&mut self, p: &ProfileIndication) {
        self.parameters.profile_options.elevation_indicators.clear();
        self.parameters
            .profile_options
            .elevation_indicators
            .insert(p.clone());
    }

    pub fn map_box(&self) -> BoundingBox {
        svgmap::euclidean_bounding_box(
            &self.track,
            &self.range,
            &self.parameters.map_options.size2d(),
        )
    }

    pub fn render_profile(&self) -> ProfileRenderResult {
        log::info!("render profile:{}", self.id);
        let ret = profile::profile(&self);
        if self.parameters.debug {
            let filename = std::format!("/tmp/profile-{}.svg", self.id);
            std::fs::write(filename, &ret.svg).expect("Unable to write file");
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
        let dbefore = p.euclidean.d2(&track.euclidian[index - 1]);
        let dafter = p.euclidean.d2(&track.euclidian[index - 1]);
        if dbefore < dafter {
            (index - 1, *index)
        } else {
            (*index, *index + 1)
        }
    }

    fn compute_track_projection(
        track: &track::Track,
        tracktree: &locate::IndexedPointsTree,
        point: &mut InputPoint,
    ) {
        let index = tracktree.nearest_neighbor(&point.euclidean).unwrap();
        let (index1, index2) = Self::two_closest_index(track, &index, point);
        let p1 = &track.euclidian[index1];
        let p2 = &track.euclidian[index2];
        let linestring: geo::LineString = vec![p1.xy(), p2.xy()].into();
        let index_floating_part = linestring
            .line_locate_point(&geo::point!(point.euclidean.xy()))
            .unwrap();
        assert!(0.0 <= index_floating_part && index_floating_part <= 1f64);
        let floating_index = index1 as f64 + index_floating_part;
        let t1 = &track.euclidian[index1];
        let t2 = &track.euclidian[index2];
        let a1 = (t1.0, t1.1, track.elevation(index1));
        let a2 = (t2.0, t2.1, track.elevation(index2));
        let m = Self::middle_point(&a1, &a2, index_floating_part);
        let euclidean = MercatorPoint::from_point2d(&Point2D::new(m.0, m.1));
        let elevation = m.2;
        let track_distance = euclidean.d2(&point.euclidean).sqrt();

        // check
        let di = point.euclidean.d2(&track.euclidian[index]);
        let df = point.euclidean.d2(&euclidean);
        debug_assert!(df <= di);

        point.track_projection = Some(TrackProjection {
            track_floating_index: floating_index,
            track_index: index1,
            euclidean,
            elevation,
            track_distance,
        });
    }

    fn copy_segment_points(
        inputpoints: &InputPointMaps,
        map_box: &BoundingBox,
        track: &track::Track,
        tracktree: &locate::IndexedPointsTree,
    ) -> Vec<InputPoint> {
        let mut ret = Vec::new();
        let mut bbox = map_box.clone();
        bbox.enlarge(&5000f64);
        let bboxs = bboxes::split(&bbox, &bboxes::BBOXWIDTH);
        for (_inputtype, map) in &inputpoints.maps {
            for (_index, bbox) in &bboxs {
                let _points = map.get(&bbox);
                if _points.is_none() {
                    continue;
                }
                let points = _points.unwrap();
                for p in points {
                    let mut c = p.clone();
                    Self::compute_track_projection(track, tracktree, &mut c);
                    ret.push(c);
                }
            }
        }
        ret
    }

    pub fn render_map(&self, size: &IntegerSize2D) -> String {
        log::trace!("render map:{}", self.id);
        let ret = svgmap::map(&self, size);
        if self.parameters.debug {
            let filename = std::format!("/tmp/map-{}.svg", self.id);
            std::fs::write(filename, &ret).expect("Unable to write file");
        }
        ret
    }
}
