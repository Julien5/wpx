use std::collections::BTreeMap;

use crate::bbox::BoundingBox;
use crate::inputpoint::{InputPoint, InputPointMaps, InputType};
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
                if p.track_projection.is_none() {
                    p.track_projection =
                        Some(locate::compute_track_projection(track, tracktree, p));
                }
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
