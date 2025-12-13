use std::collections::BTreeMap;

use geo::LineLocatePoint;

use crate::bbox::BoundingBox;
use crate::inputpoint::{InputPoint, InputPointMaps, InputType, TrackProjection};
use crate::math::{IntegerSize2D, Point2D};
use crate::mercator::MercatorPoint;
use crate::parameters::{Parameters, ProfileIndication, UserStepsOptions};
use crate::profile::ProfileRenderResult;
use crate::track::{self, Track};
use crate::{bboxes, locate, make_points, profile, svgmap};

pub type SegmentPoints = BTreeMap<InputType, Vec<InputPoint>>;

#[derive(Clone)]
pub struct Segment {
    pub id: i32,
    pub start: f64,
    pub end: f64,
    pub track: std::sync::Arc<Track>,
    pub points: SegmentPoints,
    pub parameters: Parameters,
}

pub struct SegmentStatistics {
    pub length: f64,
    pub elevation_gain: f64,
    pub distance_start: f64,
    pub distance_end: f64,
}

impl Segment {
    pub fn osmpoints(&self) -> &Vec<InputPoint> {
        return self.points.get(&InputType::OSM).unwrap();
    }

    pub fn new(
        id: i32,
        start: f64,
        end: f64,
        track_tree: locate::IndexedPointsTree,
        track: std::sync::Arc<Track>,
        inputpoints: &InputPointMaps,
        parameters: &Parameters,
    ) -> Segment {
        let range = track.segment(start, end);
        let map_box =
            svgmap::euclidean_bounding_box(&track, &range, &parameters.map_options.size2d());
        let points = Self::copy_segment_points(inputpoints, &map_box, &track, &track_tree);
        Segment {
            id,
            start,
            end,
            track,
            points,
            parameters: parameters.clone(),
        }
    }

    pub fn range(&self) -> std::ops::Range<usize> {
        self.track.segment(self.start, self.end)
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
            &self.range(),
            &self.parameters.map_options.size2d(),
        )
    }

    // used by bridge
    pub fn set_user_step_options(&mut self, options: &UserStepsOptions) {
        self.parameters.user_steps_options = options.clone();
        let range = self.range();
        let mut new_points =
            make_points::user_points(&self.track, &self.parameters.user_steps_options);
        new_points.retain(|w| {
            let index = w.round_track_index().unwrap();
            range.contains(&index)
        });
        self.points.insert(InputType::UserStep, new_points);
    }

    pub fn get_user_step_options(&mut self) -> UserStepsOptions {
        self.parameters.user_steps_options.clone()
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

    pub fn compute_track_projection(
        track: &track::Track,
        tracktree: &locate::IndexedPointsTree,
        point: &mut InputPoint,
    ) {
        // user steps projection on track is unique...
        if point.kind() == InputType::UserStep {
            assert!(point.track_projection.is_some());
            return;
        }
        // as opposed to GPX and OSM points, which may be on several segments
        assert!(!point.track_projection.is_some());
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
        let new_proj = TrackProjection {
            track_floating_index: floating_index,
            track_index: index1,
            euclidean,
            elevation,
            track_distance,
        };
        point.track_projection = Some(new_proj);
    }

    fn copy_segment_points(
        inputpoints: &InputPointMaps,
        map_box: &BoundingBox,
        track: &track::Track,
        tracktree: &locate::IndexedPointsTree,
    ) -> SegmentPoints {
        let mut ret = SegmentPoints::new();
        let mut bbox = map_box.clone();
        bbox.enlarge(&5000f64);
        let bboxs = bboxes::split(&bbox, &bboxes::BBOXWIDTH);
        for (input_type, map) in &inputpoints.maps {
            let mut points = Vec::new();
            for (_index, bbox) in &bboxs {
                let _points = map.get(&bbox);
                if _points.is_none() {
                    continue;
                }
                points.extend_from_slice(_points.unwrap());
            }
            points.iter_mut().for_each(|mut p| {
                Self::compute_track_projection(track, tracktree, &mut p);
            });
            ret.insert(input_type.clone(), points);
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
