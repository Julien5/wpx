use crate::bbox::BoundingBox;
use crate::inputpoint::InputPoint;
use crate::math::IntegerSize2D;
use crate::parameters::{Parameters, RenderFunction};
use crate::point_collection::{
    Kind, Kinds, RenderInputParameters, RenderResult, SharedPacketProvider,
};
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

    fn debug_graphic_dir(&self, marker: &str) -> Option<String> {
        if self.parameters.debug {
            Some(format!("{}", marker))
        } else {
            None
        }
    }

    fn map_profile_join_parameters(
        &self,
        kinds: &Kinds,
        map_size: &IntegerSize2D,
        profile_size: &IntegerSize2D,
    ) -> (RenderInputParameters, RenderInputParameters) {
        let profile_parameters = self.profile_render_parameters(kinds, profile_size);
        let map_parameters = self.map_render_parameters(kinds, map_size);

        let mut profile_parameters_join = profile_parameters.clone();
        profile_parameters_join.other_parameters_hash = Some(map_parameters.hash());

        let mut map_parameters_join = map_parameters.clone();
        map_parameters_join.other_parameters_hash = Some(profile_parameters.hash());
        (map_parameters_join, profile_parameters_join)
    }

    pub fn preload_map_profile(
        &self,
        kinds: &Kinds,
        map_size: &IntegerSize2D,
        profile_size: &IntegerSize2D,
    ) -> (RenderResult, RenderResult) {
        let (map_parameters_join, profile_parameters_join) =
            self.map_profile_join_parameters(kinds, map_size, profile_size);

        // try join result in cache.
        {
            let lock = self.packet_provider.read().unwrap();
            let (mapr, profiler) = (
                lock.hit(&map_parameters_join),
                lock.hit(&profile_parameters_join),
            );
            match (mapr, profiler) {
                (Some(map), Some(profile)) => {
                    return (map, profile);
                }
                _ => {}
            }
        }

        let profile_result = self.preload(&RenderFunction::Profile, kinds, profile_size);

        let map_result = svgmap::map_background(
            &self.track,
            &map_parameters_join,
            &vec![profile_result.rendered_input_points_for_map()],
            self.debug_graphic_dir(&format!(
                "preload-map-profile-{}x{}",
                map_size.width, map_size.height
            )),
        );

        // TODO: intersect.
        let mut profile_result_join = profile_result.clone();
        profile_result_join.parameters = profile_parameters_join.clone();
        let mut map_result_join = map_result.clone();
        map_result_join.parameters = map_parameters_join.clone();

        let mut lock = self.packet_provider.write().unwrap();
        lock.register_result(&profile_result_join);
        lock.register_result(&map_result_join);
        (map_result_join, profile_result_join)
    }

    pub fn preload(
        &self,
        function: &RenderFunction,
        kinds: &Kinds,
        size: &IntegerSize2D,
    ) -> RenderResult {
        match function {
            RenderFunction::Map => {}
            RenderFunction::Profile => {}
            _ => {
                return RenderResult::default();
            }
        };
        let render_parameters = match function {
            RenderFunction::Map => self.map_render_parameters(kinds, size),
            RenderFunction::Profile => self.profile_render_parameters(kinds, size),
            _ => {
                assert!(false);
                RenderInputParameters::default()
            }
        };

        // make background for that size for both profile and map
        {
            let lock = self.packet_provider.read().unwrap();
            if let Some(result) = lock.hit(&render_parameters) {
                return result;
            }
        }
        let collection = {
            let lock = self.packet_provider.read().unwrap();
            let mut coll = lock.collection.clone();
            coll.range_cut(&self.range());
            coll.kinds_cut(&kinds);
            coll
        };

        let result = match function {
            &RenderFunction::Profile => profile::profile_background(
                &self.track,
                &render_parameters,
                &collection.profile(),
                self.debug_graphic_dir(&format!("preload-profile-{}x{}", size.width, size.height)),
            ),
            &RenderFunction::Map => svgmap::map_background(
                &self.track,
                &render_parameters,
                &collection.map(),
                self.debug_graphic_dir(&format!("preload-map-{}x{}", size.width, size.height)),
            ),
            _ => RenderResult::default(),
        };

        if self.parameters.debug {
            let tmpfilename = format!(
                "/tmp/preload-{:?}-{}x{}.svg",
                function, size.width, size.height
            );
            std::fs::write(&tmpfilename, result.svg.clone()).unwrap();
        }

        assert!(result.parameters.hash() == render_parameters.hash());
        let mut lock = self.packet_provider.write().unwrap();
        lock.register_result(&result);
        result
    }

    fn profile_render_parameters(
        &self,
        kinds: &Kinds,
        size: &IntegerSize2D,
    ) -> RenderInputParameters {
        RenderInputParameters::make_profile_parameters(
            kinds,
            &self.parameters,
            size,
            &self.track,
            self.start(),
            self.end(),
            &self.usersteps(),
        )
    }

    fn map_render_parameters(&self, kinds: &Kinds, size: &IntegerSize2D) -> RenderInputParameters {
        RenderInputParameters::make_map_parameters(
            kinds,
            &self.parameters,
            size,
            &self.track,
            self.start(),
            self.end(),
            &self.usersteps(),
        )
    }

    pub fn render_profile(&self, size: &IntegerSize2D, kinds: &Kinds) -> RenderResult {
        log::info!("render profile:{} kinds:{:?}", self.id(), kinds);
        let ret = {
            let render_parameters = self.profile_render_parameters(kinds, size);
            let lock = self.packet_provider.read().unwrap();
            let last_result = lock.load(&render_parameters);
            profile::profile_foreground(
                &self.track,
                &render_parameters,
                &last_result,
                self.debug_graphic_dir(&format!("render-profile-{}x{}", size.width, size.height)),
            )
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
            let map_parameters = self.map_render_parameters(kinds, size);
            let lock = self.packet_provider.read().unwrap();
            let background = lock.load(&map_parameters);
            svgmap::map_foreground(
                &self.track,
                &map_parameters,
                &background,
                self.debug_graphic_dir(&format!("render-map-{}x{}", size.width, size.height)),
            )
        };
        if self.parameters.debug {
            let filename = std::format!("/tmp/map-{}.svg", self.id());
            std::fs::write(filename, &ret.svg).expect("Unable to write file");
        }
        ret
    }

    pub fn render_map_profile(
        &self,
        map_size: &IntegerSize2D,
        profile_size: &IntegerSize2D,
        kinds: &Kinds,
    ) -> (RenderResult, RenderResult) {
        log::info!("render map_profile:{} kinds:{:?}", self.id(), kinds);
        let (map_ret, profile_ret) = {
            let (map_parameters, profile_parameters) =
                self.map_profile_join_parameters(kinds, map_size, profile_size);
            log::trace!(
                "users steps profile: {}",
                profile_parameters.usersteps.len()
            );
            log::trace!("users steps map: {}", map_parameters.usersteps.len());
            let lock = self.packet_provider.read().unwrap();

            let map_background = lock.load(&map_parameters);
            let profile_background = lock.load(&profile_parameters);

            let (rm, rp) = (
                svgmap::map_foreground(
                    &self.track,
                    &map_parameters,
                    &map_background,
                    self.debug_graphic_dir(&format!(
                        "joinmap-{}x{}",
                        map_size.width, map_size.height
                    )),
                ),
                profile::profile_foreground(
                    &self.track,
                    &profile_parameters,
                    &profile_background,
                    self.debug_graphic_dir(&format!(
                        "joinprofile-{}x{}",
                        profile_size.width, profile_size.height
                    )),
                ),
            );
            (rm, rp)
        };
        if self.parameters.debug {
            let filename = std::format!("/tmp/joinprofile-{}.svg", self.id());
            std::fs::write(filename, &profile_ret.svg).expect("Unable to write file");
            let filename = std::format!("/tmp/joinmap-{}.svg", self.id());
            std::fs::write(filename, &map_ret.svg).expect("Unable to write file");
        }
        (map_ret, profile_ret)
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use crate::{
        controls, event,
        gpsdata::GpxData,
        make_points,
        math::IntegerSize2D,
        osm,
        parameters::{Parameters, ProfileIndication, RenderFunction},
        point_collection::{
            allkinds, onekind, Kind, Kinds, PacketProvider, PointCollection, RenderResult,
            SharedPacketProvider,
        },
        profile,
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
        gpsdata::GpxData::read_content(&content).unwrap()
    }

    static START_TIME: &'static str = "1985-04-12T06:05:00.00Z";

    async fn load_segment(
        filename: &str,
        start: f64,
        length: f64,
        parameters: Parameters,
    ) -> SegmentData {
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
        collection.import_other(&Kind::GPXWaypoints, waypoints);
        collection.import_other(&Kind::Controls, controls);

        let usersteps = make_points::user_points(&track, &parameters.user_steps_options);
        collection.import_other(&Kind::UserStep, usersteps);

        let fsegment = Segment {
            id: 0,
            start: start,
            end: start + length,
        };
        let mut pprovider = PacketProvider::new();
        pprovider.collection = collection;
        let provider = SharedPacketProvider::new(pprovider.into());

        SegmentData::new(&fsegment, track, provider, parameters)
    }

    fn basename(path: &str) -> String {
        use std::path::Path;
        Path::new(path)
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or("")
            .to_string()
    }

    async fn graph_test(
        src: &str,
        reffilename: &str,
        function: &RenderFunction,
        start: f64,
        length: f64,
        size: &IntegerSize2D,
    ) -> bool {
        let _ = env_logger::try_init();
        let mut parameters = Parameters::default();
        parameters.start_time = START_TIME.to_string();
        parameters.map_options.max_area_ratio = 0.15f64;
        parameters.user_steps_options.step_distance = None;
        parameters.user_steps_options.step_elevation_gain = Some(250f64);
        parameters.profile_options.elevation_indicators = vec![ProfileIndication::NumericSlope];
        let segment = load_segment(src, start, length, parameters).await;
        //let kinds = onekind(Kind::Cities);
        let kinds = allkinds();
        let map_parameters = segment.map_render_parameters(&kinds, size);
        let profile_parameters = segment.profile_render_parameters(&kinds, size);

        let mut collection = segment.packet_provider.read().unwrap().collection.clone();
        collection.range_cut(&segment.range());
        collection.kinds_cut(&kinds);
        let result = match function {
            &RenderFunction::Profile => profile::profile_background(
                &segment.track,
                &profile_parameters,
                &collection.profile(),
                None,
            ),
            &RenderFunction::Map => {
                svgmap::map_background(&segment.track, &map_parameters, &collection.map(), None)
            }
            _ => {
                assert!(false);
                RenderResult::default()
            }
        };

        println!("test {}", reffilename);
        let data = if std::fs::exists(&reffilename).unwrap() {
            std::fs::read_to_string(&reffilename).unwrap()
        } else {
            String::new()
        };
        let tmpfilename = std::format!("/tmp/{}", basename(&reffilename));
        std::fs::write(&tmpfilename, &result.svg).expect("Unable to write file");
        if data != result.svg {
            println!("test failed: {} {}", tmpfilename, reffilename);
            return false;
        }
        true
    }

    #[tokio::test]
    async fn graph_winni() {
        let _ = env_logger::try_init();
        let start = 0f64;
        let length = 110_000f64;
        let size = IntegerSize2D::new(1600, 1000);
        let ok = graph_test(
            "data/ref/winni.gpx",
            "data/ref/singlemap-winni.svg",
            &RenderFunction::Map,
            start,
            length,
            &size,
        )
        .await;
        assert!(ok);
    }

    #[tokio::test]
    async fn graph_jerome() {
        let _ = env_logger::try_init();
        let start = 0f64;
        let length = 60_000f64;
        let size = IntegerSize2D::new(1600, 1000);
        let ok = graph_test(
            "data/jerome.gpx",
            "data/ref/singlemap-jerome.svg",
            &RenderFunction::Map,
            start,
            length,
            &size,
        )
        .await;
        assert!(ok);
    }

    #[tokio::test]
    async fn graph_roland() {
        let _ = env_logger::try_init();
        let start = 0f64;
        let length = 200_000f64;
        let size = IntegerSize2D::new(1600, 1000);
        let ok = graph_test(
            "data/ref/roland-nowaypoints.gpx",
            "data/ref/singlemap-roland.svg",
            &RenderFunction::Map,
            start,
            length,
            &size,
        )
        .await;
        assert!(ok);
    }

    #[tokio::test]
    async fn graph_black() {
        let _ = env_logger::try_init();
        let start = 100_000f64;
        let length = 110_000f64;
        let size = IntegerSize2D::new(400, 400);
        let ok = graph_test(
            "data/blackforest.gpx",
            "data/ref/singlemap-black.svg",
            &RenderFunction::Map,
            start,
            length,
            &size,
        )
        .await;
        assert!(ok);
    }

    #[tokio::test]
    async fn graph_profile() {
        let _ = env_logger::try_init();
        let start = 0_000f64;
        let length = 110_000f64;
        let size = IntegerSize2D::new(1420, 400);
        let ok = graph_test(
            "data/blackforest.gpx",
            "data/ref/singleprofile-black.svg",
            &RenderFunction::Profile,
            start,
            length,
            &size,
        )
        .await;
        assert!(ok);
    }

    #[tokio::test]
    async fn graph_pbp() {
        let _ = env_logger::try_init();
        let start = 0f64;
        let length = 1200_000f64;
        let size = IntegerSize2D::new(1600, 1000);
        let ok = graph_test(
            "data/ref/pbp2023.gpx",
            "data/ref/singlemap-pbp2023.svg",
            &RenderFunction::Map,
            start,
            length,
            &size,
        )
        .await;
        assert!(ok);
    }
}
