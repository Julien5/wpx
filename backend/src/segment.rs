use crate::bbox::BoundingBox;
use crate::inputpoint::InputPoint;
use crate::math::IntegerSize2D;
use crate::parameters::{self, Parameters};
use crate::point_collection::{
    Kind, Kinds, PacketProvider, PointCollection, RenderInputParameters, RenderResult,
};
use crate::speed::TimeParameters;
use crate::track::SharedTrack;
use crate::waypoint::{waypoint_for_segment, Waypoint};
use crate::{profile, svgmap};

#[derive(Clone)]
pub struct Segment {
    pub id: i32,
    pub start: f64,
    pub end: f64,
}

pub struct SegmentData<'a> {
    pub segment: Segment,
    pub track: SharedTrack,
    pub parameters: Parameters,
    pub time_parameters: TimeParameters,
    pub packet_provider: &'a PacketProvider,
}

pub struct SegmentStatistics {
    pub length: f64,
    pub elevation_gain: f64,
    pub distance_start: f64,
    pub distance_end: f64,
    pub start_time: String,
    pub end_time: String,
    pub waypoints: Vec<Waypoint>,
    pub controls: Vec<Waypoint>,
}

impl<'a> SegmentData<'a> {
    pub fn new(
        segment: &Segment,
        track: SharedTrack,
        packet_provider: &'a PacketProvider,
        parameters: Parameters,
        time_parameters: TimeParameters,
    ) -> SegmentData<'a> {
        if segment.start > track.total_distance() {
            panic!(
                "range does not intersect with the track ({}>{})",
                segment.start,
                track.total_distance()
            );
        }
        SegmentData {
            segment: segment.clone(),
            track,
            packet_provider: packet_provider,
            parameters: parameters,
            time_parameters,
        }
    }

    pub fn statistics(&self) -> SegmentStatistics {
        let track = &self.track;
        let range = self.range();
        let mut waypoints = waypoint_for_segment(&self.gpxwaypoints(), &self);
        waypoints.sort_by_key(|w| w.track_index.unwrap());
        let mut controls = waypoint_for_segment(&self.controls(), &self);
        controls.sort_by_key(|w| w.track_index.unwrap());
        let distance_start = track.distance(range.start);
        let distance_end = track.distance(range.end - 1);
        SegmentStatistics {
            length: self.segment.end - self.segment.start,
            elevation_gain: track.elevation_gain_on_range(&range),
            distance_start,
            distance_end,
            start_time: parameters::time_to_iso8601(&self.time_parameters.time(distance_start)),
            end_time: parameters::time_to_iso8601(&self.time_parameters.time(distance_end)),
            waypoints,
            controls,
        }
    }

    fn kind_on_segment(&self, kind: &Kind) -> Vec<InputPoint> {
        let c = self.packet_provider.collection.get_vector(kind);
        self.points_on_segment(&c)
    }

    fn points_on_segment(&self, points: &Vec<InputPoint>) -> Vec<InputPoint> {
        // faster than range_cut (which iterates on all kinds, including OSM)
        let mut ret = Vec::new();
        for p in points {
            for proj in &p.track_projections {
                let d = proj.distance_on_track_to_projection;
                if self.start() <= d && d <= self.end() {
                    ret.push(p.clone());
                    break;
                }
            }
        }
        ret
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

    pub fn gpxwaypoints(&self) -> Vec<InputPoint> {
        self.kind_on_segment(&Kind::GPXWaypoints)
    }

    pub fn controls(&self) -> Vec<InputPoint> {
        self.kind_on_segment(&Kind::Controls)
    }

    pub fn background_points(&self) -> Vec<Vec<InputPoint>> {
        vec![self.controls(), self.gpxwaypoints()]
    }

    pub fn usersteps(&self) -> Vec<InputPoint> {
        self.kind_on_segment(&Kind::CutOff)
    }

    pub fn potential_controls(&self) -> Vec<InputPoint> {
        let mut clone = self.packet_provider.collection.clone();
        clone.map.iter_mut().for_each(|(_key, points)| {
            points.retain(|point| point.is_in_distance_range(self.start(), self.end()))
        });
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

    fn profile_render_parameters(
        &self,
        kinds: &Kinds,
        size: &IntegerSize2D,
    ) -> RenderInputParameters {
        RenderInputParameters::make_profile_parameters(
            kinds,
            &self.parameters,
            &self.time_parameters,
            size,
            &self.track,
            self.start(),
            self.end(),
            &self.background_points(),
            &self.usersteps(),
        )
    }

    fn map_render_parameters(&self, kinds: &Kinds, size: &IntegerSize2D) -> RenderInputParameters {
        RenderInputParameters::make_map_parameters(
            kinds,
            &self.parameters,
            &self.time_parameters,
            size,
            &self.track,
            self.start(),
            self.end(),
            &self.background_points(),
            &self.usersteps(),
        )
    }

    pub fn render_profile(&self, size: &IntegerSize2D, kinds: &Kinds) -> RenderResult {
        log::info!("render profile:{} kinds:{:?}", self.id(), kinds);
        let ret = {
            let render_parameters = self.profile_render_parameters(kinds, size);
            profile::render_profile(
                &self.track,
                &render_parameters,
                &self
                    .packet_provider
                    .collection
                    .profile(&self.segment, kinds),
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
            svgmap::render_map(
                &self.track,
                &map_parameters,
                &self.packet_provider.collection.map(&self.segment, kinds),
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
            let map_parameters = self.map_render_parameters(kinds, map_size);
            let profile_parameters = self.profile_render_parameters(kinds, profile_size);

            let rp = profile::render_profile(
                &self.track,
                &profile_parameters,
                &self
                    .packet_provider
                    .collection
                    .profile(&self.segment, kinds),
                self.debug_graphic_dir(&format!(
                    "joinprofile-{}x{}",
                    profile_size.width, profile_size.height
                )),
            );

            let profile_collection = PointCollection::from_result(&rp);

            let rm = svgmap::render_map(
                &self.track,
                &map_parameters,
                &profile_collection.map(&self.segment, kinds),
                self.debug_graphic_dir(&format!("joinmap-{}x{}", map_size.width, map_size.height)),
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

    use tokio_util::sync::CancellationToken;

    use crate::{
        backend_data::BackendData,
        controls, event,
        gpsdata::GpxData,
        make_points,
        math::IntegerSize2D,
        osm::{self, DownloadSideData},
        parameters::{self, Parameters, ProfileIndication, RenderFunction},
        point_collection::{
            allkinds, controls_speed_data, Kind, PacketProvider, PointCollection, RenderResult,
        },
        profile,
        segment::{Segment, SegmentData},
        speed, svgmap,
        testhelpers::{load_backend_data, load_backend_data_with_parameters, load_file},
        track::Track,
    };

    static START_TIME: &'static str = "1985-04-12T09:00:00";
    const WITH_OSM: bool = true;

    fn basename(path: &str) -> String {
        use std::path::Path;
        Path::new(path)
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or("")
            .to_string()
    }

    fn make_segment(start: f64, length: f64) -> Segment {
        Segment {
            id: 0,
            start: start,
            end: start + length,
        }
    }

    async fn graph_test(
        src: &str,
        reffilename: &str,
        function: &RenderFunction,
        start: f64,
        length: f64,
        size: &IntegerSize2D,
        with_osm: bool,
    ) -> bool {
        let _ = env_logger::try_init();
        let (track, gpxdata) = load_file(src);
        let mut parameters = Parameters::default();
        parameters.start_time = START_TIME.to_string();
        parameters.user_steps_options.step_distance = None;
        parameters.user_steps_options.step_elevation_gain = Some(250f64);
        parameters.profile_options.elevation_indicators = vec![ProfileIndication::NumericSlope];

        let allowed_speeds = speed::allowed_speeds(track.total_distance());
        parameters.speed = match allowed_speeds.iter().find(|spec| spec.contains("ACP")) {
            Some(spec) => spec.clone(),
            None => speed::format_kmh(15.0),
        };

        let backend_data = load_backend_data_with_parameters(src, parameters, with_osm).await;
        //let kinds = onekind(Kind::Cities);
        let kinds = allkinds();
        let fsegment = make_segment(start, length);
        let segment = backend_data.make_segment_data(&fsegment);
        let map_parameters = segment.map_render_parameters(&kinds, size);
        let profile_parameters = segment.profile_render_parameters(&kinds, size);

        let mut collection = segment.packet_provider.collection.clone();

        let result = match function {
            &RenderFunction::Profile => profile::render_profile(
                &segment.track,
                &profile_parameters,
                &collection.profile(&fsegment, &kinds),
                None,
            ),
            &RenderFunction::Map => svgmap::render_map(
                &segment.track,
                &map_parameters,
                &collection.map(&fsegment, &kinds),
                None,
            ),
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
            WITH_OSM,
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
            WITH_OSM,
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
            WITH_OSM,
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
            WITH_OSM,
        )
        .await;
        assert!(ok);
    }

    #[tokio::test]
    async fn graph_profile_black() {
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
            WITH_OSM,
        )
        .await;
        assert!(ok);
    }

    #[tokio::test]
    async fn graph_profile_nudel() {
        let _ = env_logger::try_init();
        let start = 0_000f64;
        let length = 380979.9055239884;
        let size = IntegerSize2D::new(1099, 255);
        let ok = graph_test(
            "data/ref/nudel.gpx",
            "data/ref/singleprofile-nudel.svg",
            &RenderFunction::Profile,
            start,
            length,
            &size,
            !WITH_OSM,
        )
        .await;
        assert!(ok);
    }

    #[tokio::test]
    async fn graph_map_pbp() {
        let _ = env_logger::try_init();
        let start = 0f64;
        let length = 1220_000f64;
        let size = IntegerSize2D::new(1600, 1000);
        let ok = graph_test(
            "data/PBP-simple.gpx",
            "data/ref/singlemap-pbp2023.svg",
            &RenderFunction::Map,
            start,
            length,
            &size,
            WITH_OSM,
        )
        .await;
        assert!(ok);
    }

    #[tokio::test]
    async fn graph_profile_pbp() {
        let _ = env_logger::try_init();
        let start = 0f64;
        let length = 1220_000f64;
        let size = IntegerSize2D::new(3000, 300);
        let ok = graph_test(
            "data/PBP-simple.gpx",
            "data/ref/singleprofile-pbpsimple.svg",
            &RenderFunction::Profile,
            start,
            length,
            &size,
            WITH_OSM,
        )
        .await;
        assert!(ok);
    }
}
