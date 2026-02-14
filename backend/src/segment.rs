use std::collections::HashSet;

use crate::bbox::BoundingBox;
use crate::inputpoint::InputPoint;
use crate::math::IntegerSize2D;
use crate::parameters::Parameters;
use crate::point_collection::{Kind, Kinds, RenderFunction, RenderResult, SharedPacketProvider};
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
        SegmentData {
            segment: segment.clone(),
            track,
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

    pub fn preload(&self, size: &IntegerSize2D) {
        let osmkinds =
            HashSet::from([Kind::Cities, Kind::Villages, Kind::Mountains, Kind::Hamlets]);
        let mut parameters = Parameters::default();
        parameters.profile_options.max_area_ratio = parameters
            .profile_options
            .max_area_ratio
            .max(self.parameters.profile_options.max_area_ratio);
        parameters.map_options.max_area_ratio = parameters
            .map_options
            .max_area_ratio
            .max(self.parameters.map_options.max_area_ratio);
        parameters.profile_options.elevation_indicators.clear();

        {
            let lock = self.packet_provider.read().unwrap();
            if lock.hit(&RenderFunction::Map, &self.range(), &parameters, size)
                && lock.hit(&RenderFunction::Profile, &self.range(), &parameters, size)
            {
                log::trace!("preload hit");
                return;
            }
        }
        log::trace!("preload build");

        let collection = {
            let lock = self.packet_provider.read().unwrap();
            let mut coll = lock.collection.clone();
            // remove user steps and controls, we render only osm
            coll.import_other(&Kind::UserStep, Vec::new(), &self.track);
            coll.import_other(&Kind::Controls, Vec::new(), &self.track);
            coll.range_cut(&self.range());
            coll.kinds_cut(&osmkinds);
            coll
        };

        let mut lock = self.packet_provider.write().unwrap();
        let profile_packets = collection.profile();
        let result_profile = profile::profile(&self, size, &profile_packets);
        lock.register_profile_result(&parameters, &self.range(), size, &result_profile);

        let map_packets = collection.map();
        let result_map = svgmap::map(&self, size, &map_packets);
        lock.register_map_result(&parameters, &self.range(), size, &result_map);
    }

    pub fn render_profile(&self, size: &IntegerSize2D, kinds: &Kinds) -> RenderResult {
        log::info!("render profile:{} kinds:{:?}", self.id(), kinds);
        let ret = {
            let lock = self.packet_provider.read().unwrap();
            let mut collection = lock.load(
                &RenderFunction::Profile,
                &self.range(),
                &self.parameters,
                size,
            );
            collection.kinds_cut(kinds);
            profile::profile(&self, size, &collection.profile())
        };
        if self.parameters.debug {
            let filename = std::format!("/tmp/profile-{}.svg", self.id());
            std::fs::write(filename, &ret.svg).expect("Unable to write file");
        }
        ret
    }

    pub fn render_map(&self, size: &IntegerSize2D, kinds: &Kinds) -> RenderResult {
        log::info!("render map:{}", self.id());
        let ret = {
            let lock = self.packet_provider.read().unwrap();
            let mut collection =
                lock.load(&RenderFunction::Map, &self.range(), &self.parameters, size);
            collection.kinds_cut(kinds);
            svgmap::map(&self, size, &collection.map())
        };
        if self.parameters.debug {
            let filename = std::format!("/tmp/map-{}.svg", self.id());
            std::fs::write(filename, &ret.svg).expect("Unable to write file");
        }
        ret
    }
}
