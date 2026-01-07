use crate::bbox::BoundingBox;
use crate::bboxes::BoundingBoxes;
use crate::inputpoint::{InputPoint, InputType, SharedPointMaps};
use crate::math::IntegerSize2D;
use crate::parameters::Parameters;
use crate::profile::ProfileRenderResult;
use crate::track::SharedTrack;
use crate::{bboxes, profile, svgmap};

#[derive(Clone)]
pub struct Segment {
    pub id: i32,
    pub start: f64,
    pub end: f64,
}

pub struct SegmentData {
    pub segment: Segment,
    pub track: SharedTrack,
    pub boxes: BoundingBoxes,
    _pointmaps: SharedPointMaps,
    pub parameters: Parameters,
}

pub struct SegmentStatistics {
    pub length: f64,
    pub elevation_gain: f64,
    pub distance_start: f64,
    pub distance_end: f64,
}

impl SegmentData {
    pub fn new(
        segment: &Segment,
        track: SharedTrack,
        inputpoints: SharedPointMaps,
        parameters: Parameters,
    ) -> SegmentData {
        let boxes = track.subboxes(segment.start, segment.end);
        SegmentData {
            segment: segment.clone(),
            track,
            boxes,
            _pointmaps: inputpoints.clone(),
            //pointmaps: SharedPointMaps::new(InputPointMaps::new().into()),
            parameters: parameters.clone(),
        }
    }

    pub fn id(&self) -> i32 {
        self.segment.id
    }

    pub fn start(&self) -> f64 {
        self.segment.start
    }

    pub fn end(&self) -> f64 {
        self.segment.end
    }

    pub fn points(&self, kind: &InputType) -> Vec<InputPoint> {
        let bbox = bboxes::bounding_box(&self.boxes);
        let range = self.range();
        let lock = self._pointmaps.read().unwrap();
        let map = lock.maps.get(kind);
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

    pub fn osmpoints(&self) -> Vec<InputPoint> {
        self.points(&InputType::OSM)
    }

    pub fn range(&self) -> std::ops::Range<usize> {
        self.track.subrange(self.segment.start, self.segment.end)
    }

    pub fn map_box(&self) -> BoundingBox {
        svgmap::euclidean_bounding_box(
            &self.track,
            &self.range(),
            &self.parameters.map_options.size2d(),
        )
    }

    pub fn render_profile(&self, size: &IntegerSize2D) -> ProfileRenderResult {
        log::info!("render profile:{}", self.id());
        let ret = profile::profile(&self, size);
        if self.parameters.debug {
            let filename = std::format!("/tmp/profile-{}.svg", self.id());
            std::fs::write(filename, &ret.svg).expect("Unable to write file");
        }
        ret
    }

    pub fn render_map(&self, size: &IntegerSize2D) -> String {
        log::info!("render map:{}", self.id());
        let ret = svgmap::map(&self, size);
        if self.parameters.debug {
            let filename = std::format!("/tmp/map-{}.svg", self.id());
            std::fs::write(filename, &ret).expect("Unable to write file");
        }
        ret
    }
}
