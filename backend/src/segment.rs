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

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use crate::{
        controls, event,
        gpsdata::GpxData,
        math::IntegerSize2D,
        osm,
        parameters::Parameters,
        point_collection::{Kind, PacketProvider, PointCollection, SharedPacketProvider},
        segment::{Segment, SegmentData},
        svgmap,
        track::Track,
    };

    fn read(filename: &str) -> GpxData {
        use crate::gpsdata;
        let mut f = std::fs::File::open(filename).unwrap();
        let mut content = Vec::new();
        // read the whole file
        use std::io::prelude::*;
        f.read_to_end(&mut content).unwrap();
        gpsdata::read_content(&content).unwrap()
    }

    static START_TIME: &'static str = "1985-04-12T06:05:00.00Z";

    #[tokio::test]
    async fn svg_map_single() {
        let _ = env_logger::try_init();
        let filename = "data/blackforest.gpx";
        let gpxdata = read(filename);
        let track = Arc::new(Track::from_tracks(&gpxdata.tracks).unwrap());

        let b: event::SenderHandler = Box::new(event::ConsoleEventSender {});
        let logger = std::sync::RwLock::new(Some(b));
        let mut osmpoints = osm::download_for_track(&track, &logger).await.unwrap();
        track.project_map(&mut osmpoints);

        let mut waypoints = gpxdata.waypoints.clone();
        for w in &mut waypoints {
            track.project_point(w);
        }

        let mut collection = PointCollection::new();
        collection.import_osm(&osmpoints.as_vector());
        let mut controls = controls::make_controls_with_waypoints(&track, &waypoints);
        for c in &mut controls {
            track.project_point(c);
        }
        collection.import_other(&Kind::GPXWaypoints, waypoints, &track);
        collection.import_other(&Kind::Controls, controls, &track);

        let fsegment = Segment {
            id: 0,
            start: 000_000f64,
            end: 110_000f64,
        };
        let provider = SharedPacketProvider::new(PacketProvider::new().into());
        let mut parameters = Parameters::default();
        parameters.start_time = START_TIME.to_string();
        parameters.map_options.max_area_ratio = 0.15f64;

        let segment = SegmentData::new(&fsegment, track, provider, parameters);
        let size = IntegerSize2D::new(400, 400);
        //let packets = vec![collection.get_vector(&Kind::Villages)];
        collection.range_cut(&segment.range());
        let packets = collection.map();
        let result_map = svgmap::map(&segment, &size, &packets);

        let reffilename = std::format!("data/ref/singlemap.svg");
        println!("test {}", reffilename);
        let data = if std::fs::exists(&reffilename).unwrap() {
            std::fs::read_to_string(&reffilename).unwrap()
        } else {
            String::new()
        };
        let tmpfilename = std::format!("/tmp/singlemap.svg");
        std::fs::write(&tmpfilename, &result_map.svg).expect("Unable to write file");
        if data != result_map.svg {
            println!("test failed: {} {}", tmpfilename, reffilename);
            assert!(false);
        }
    }
}
