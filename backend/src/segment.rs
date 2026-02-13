use crate::bbox::BoundingBox;
use crate::inputpoint::InputPoint;
use crate::math::IntegerSize2D;
use crate::parameters::Parameters;
use crate::point_collection::{Kind, Kinds, RenderResult, SharedPacketProvider};
use crate::tile::Tiles;
use crate::track::SharedTrack;
use crate::{profile, svgmap};

#[derive(Clone)]
pub struct Segment {
    pub id: i32,
    pub start: f64,
    pub end: f64,
}

pub struct SegmentData {
    pub segment: Segment,
    pub track: SharedTrack,
    pub boxes: Tiles,
    pub parameters: Parameters,
    pub packet_provider: SharedPacketProvider,
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
        packet_provider: SharedPacketProvider,
        parameters: Parameters,
    ) -> SegmentData {
        let boxes = track.tiles(segment.start, segment.end);
        SegmentData {
            segment: segment.clone(),
            track,
            boxes,
            //pointmaps: SharedPointMaps::new(InputPointMaps::new().into()),
            packet_provider: packet_provider,
            parameters: parameters,
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

    pub fn controls(&self) -> Vec<InputPoint> {
        let lock = self.packet_provider.read();
        let mut clone = lock.unwrap().collection.clone();
        clone.range_cut(&self.range());
        clone.get_vector(&Kind::Controls)
    }

    pub fn usersteps(&self) -> Vec<InputPoint> {
        let lock = self.packet_provider.read();
        let mut clone = lock.unwrap().collection.clone();
        clone.range_cut(&self.range());
        clone.get_vector(&Kind::UserStep)
    }

    pub fn potential_controls(&self) -> Vec<InputPoint> {
        let lock = self.packet_provider.read();
        let mut clone = lock.unwrap().collection.clone();
        clone.range_cut(&self.range());
        clone.potential_controls()
    }

    pub fn range(&self) -> std::ops::Range<usize> {
        self.track.subrange(self.segment.start, self.segment.end)
    }

    pub fn map_box(&self) -> BoundingBox {
        svgmap::euclidean_bounding_box(&self.track, &self.range())
    }

    pub fn render_profile(&self, size: &IntegerSize2D, kinds: &Kinds) -> RenderResult {
        log::info!("render profile:{}", self.id());
        let ret = profile::profile(&self, size, kinds);
        if self.parameters.debug {
            let filename = std::format!("/tmp/profile-{}.svg", self.id());
            std::fs::write(filename, &ret.svg).expect("Unable to write file");
        }
        ret
    }

    pub fn render_map(&self, size: &IntegerSize2D, kinds: &Kinds) -> RenderResult {
        log::info!("render map:{}", self.id());
        let ret = svgmap::map(&self, size, kinds);
        if self.parameters.debug {
            let filename = std::format!("/tmp/map-{}.svg", self.id());
            std::fs::write(filename, &ret.svg).expect("Unable to write file");
        }
        ret
    }

    pub fn profile_packets(&self, screen_size: &IntegerSize2D) -> Vec<Vec<InputPoint>> {
        self.packet_provider
            .read()
            .unwrap()
            .profile(&self.range(), &self.parameters, &screen_size)
    }
    pub fn map_packets(&self, screen_size: &IntegerSize2D) -> Vec<Vec<InputPoint>> {
        self.packet_provider
            .read()
            .unwrap()
            .map(&self.range(), &self.parameters, &screen_size)
    }
}
