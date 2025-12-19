use std::collections::BTreeSet;

use crate::bbox::BoundingBox;
use crate::inputpoint::{InputPoint, InputType, SharedPointMaps};
use crate::math::IntegerSize2D;
use crate::parameters::{Parameters, ProfileIndication, UserStepsOptions};
use crate::profile::ProfileRenderResult;
use crate::track::Track;
use crate::{bboxes, make_points, profile, svgmap};

#[derive(Clone)]
pub struct Segment {
    pub id: i32,
    pub start: f64,
    pub end: f64,
    pub track: std::sync::Arc<Track>,
    pub boxes: BTreeSet<BoundingBox>,
    pub pointmaps: SharedPointMaps,
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
        start: f64,
        end: f64,
        track: std::sync::Arc<Track>,
        inputpoints: &SharedPointMaps,
        parameters: &Parameters,
    ) -> Segment {
        let boxes = track.subboxes(start, end);
        Segment {
            id,
            start,
            end,
            track,
            boxes,
            pointmaps: inputpoints.clone(),
            //pointmaps: SharedPointMaps::new(InputPointMaps::new().into()),
            parameters: parameters.clone(),
        }
    }

    pub fn osmpoints(&self) -> Vec<InputPoint> {
        let bbox = bboxes::bounding_box(&self.boxes);
        let range = self.range();
        let lock = self.pointmaps.read().unwrap();
        let map = lock.maps.get(&InputType::OSM);
        if map.is_none() {
            return Vec::new();
        }
        // todo: we need a bounding box in the input parameters
        map.unwrap()
            .points_in(&bbox)
            .filter(|w| w.track_projections.is_empty() || w.is_in_range(&range))
            .map(|w| w.clone())
            .collect()
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
        // FIXME
        //self.points.insert(InputType::UserStep, new_points);
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

    pub fn render_map(&self, size: &IntegerSize2D) -> String {
        log::info!("render map:{}", self.id);
        let ret = svgmap::map(&self, size);
        if self.parameters.debug {
            let filename = std::format!("/tmp/map-{}.svg", self.id);
            std::fs::write(filename, &ret).expect("Unable to write file");
        }
        ret
    }
}
