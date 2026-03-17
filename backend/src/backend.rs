#![allow(non_snake_case)]

use std::collections::BTreeMap;

use crate::controls;
use crate::error;
use crate::error::TrackError;
use crate::event;
use crate::gpsdata;
use crate::gpsdata::GpxData;
use crate::gpxexport;
use crate::inputpoint::*;
use crate::make_points;
use crate::math::IntegerSize2D;
use crate::osm;
use crate::parameters;
use crate::parameters::karl_order;
use crate::parameters::ControlSource;
use crate::parameters::Parameters;
use crate::parameters::ProfileIndication;
use crate::parameters::RenderFunction;
use crate::parameters::RenderInput;
use crate::parameters::RenderOutput;
use crate::parameters::TrackPart;
use crate::parameters::UserStepsOptions;
use crate::pdf;
use crate::point_collection::Kind;
use crate::point_collection::Kinds;
use crate::point_collection::PacketProvider;
use crate::point_collection::SharedPacketProvider;
use crate::render;
use crate::segment::SegmentData;
use crate::track::SharedTrack;
use crate::track::Track;
use crate::track_projection::is_close_to_track;
use crate::waypoint;
use crate::waypoint::Waypoint;
use crate::waypoint::WaypointInfo;
use crate::waypoint::Waypoints;
use crate::wheel;

pub type Segment = crate::segment::Segment;
pub type SegmentStatistics = crate::segment::SegmentStatistics;
pub use crate::event::Sender;
use crate::zipexport;
pub type SenderHandler = crate::event::SenderHandler;
pub type SenderHandlerLock = crate::event::SenderHandlerLock;

pub struct BackendData {
    pub parameters: Parameters,
    pub track: SharedTrack,
    pub packet_provider: SharedPacketProvider,
}

pub struct Backend {
    backend_data: Option<BackendData>,
    gpxdata: std::sync::RwLock<Option<GpxData>>,
    pub sender: SenderHandlerLock,
}

impl Backend {
    pub fn make() -> Backend {
        Backend {
            backend_data: None,
            gpxdata: std::sync::RwLock::new(None),
            sender: std::sync::RwLock::new(None),
        }
    }
    pub fn loaded(&self) -> bool {
        self.backend_data.is_some()
    }
    pub fn set_sink(&mut self, sink: SenderHandler) {
        self.sender = std::sync::RwLock::new(Some(sink));
    }
    pub fn send(&self, data: &str) {
        log::trace!("event:{}", data);
        if self.sender.read().unwrap().is_none() {
            return;
        }
        event::send_worker(&self.sender, data);
    }

    pub async fn load_osm(&self) -> Result<(), TrackError> {
        log::trace!("download osm data");
        self.send("download osm data");

        /*let tick = tokio::time::Duration::from_millis(1000);
        for i in 0..5 {
            self.send(&format!("download {}", i));
            tokio::time::sleep(tick).await;
        }*/

        let mut osmpoints = match osm::download_for_track(&self.d().track, &self.sender).await {
            Ok(p) => {
                if std::path::Path::new(&"/tmp/force_error").exists() {
                    return Err(TrackError::OSMDownloadFailed);
                }
                p
            }
            Err(e) => {
                log::error!("OSM download failed {}", e);
                return Err(error::TrackError::from(e));
            }
        };

        self.d().track.project_map(&mut osmpoints);

        self.send("sort points");
        {
            // TODO: osmpoints are sorted per tile.
            // we loose the sorting. Performance loss is okay, but this probably needs cleanup.
            let mut locked = self.d().packet_provider.write().unwrap();
            locked.collection.import_osm(&osmpoints.as_vector());
        }

        Ok(())
    }

    pub async fn load_controls(&self, source: ControlSource) -> Result<usize, TrackError> {
        let waypoints = self
            .d()
            .packet_provider
            .read()
            .unwrap()
            .collection
            .get_vector(&Kind::GPXWaypoints);
        let mut controls = match source {
            ControlSource::Segments => {
                controls::infer_controls_from_gpx_segments(&self.d().track, &waypoints)
            }
            ControlSource::Waypoints => {
                controls::make_controls_with_waypoints(&self.d().track, &waypoints)
            }
            ControlSource::OSM => {
                assert!(false);
                let segment = self.make_segment_data(&self.trackSegment());
                controls::make_with_osm(
                    &segment,
                    self.d().packet_provider.clone(),
                    70_000.0,
                    &Kind::Controls,
                )
            }
        };

        // the case of mutiple projections for a single point is not handled correctly
        // in export_points (the waypoint creation assume a single projections).
        for c in &mut controls {
            debug_assert!(!c.track_projections.is_empty());
            if c.track_projections.is_empty() {
                self.d().track.project_point(c);
            }
        }

        let len = controls.len();

        // update provider
        {
            let mut locked = self.d().packet_provider.write().unwrap();
            locked.collection.import_other(&Kind::Controls, controls);
        }

        Ok(len)
    }

    pub async fn load_content(&mut self, content: &Vec<u8>) -> Result<(), TrackError> {
        self.load_contents(&vec![content.clone()]).await
    }

    pub async fn load_track_parts(
        &self,
        contents: &Vec<Vec<u8>>,
    ) -> Result<Vec<TrackPart>, TrackError> {
        self.send("read gpx");
        let gpxdata = gpsdata::GpxData::read_contents(contents)?;
        let rparts = gpxdata.track_parts();
        let parts = karl_order(&rparts);
        {
            let mut locked = self.gpxdata.write().unwrap();
            *locked = Some(gpxdata);
        }
        Ok(parts)
    }

    pub async fn load_ordered(&mut self, parts: &Vec<TrackPart>) -> Result<(), TrackError> {
        assert!(self.gpxdata.read().unwrap().is_some());
        let mut gpxdata = {
            let mut locked = self.gpxdata.write().unwrap();
            let indices: Vec<_> = parts.iter().map(|part| part.part_index.clone()).collect();
            locked.as_mut().unwrap().reorder(&indices)
        };
        let track_data = Track::from_tracks(&gpxdata.tracks)?;
        let track = std::sync::Arc::new(track_data);
        for p in &mut gpxdata.waypoints {
            track.project_point(p);
        }

        let parameters = Parameters::default();
        let point_collection = SharedPacketProvider::new(PacketProvider::new().into());
        point_collection
            .write()
            .unwrap()
            .collection
            .import_other(&Kind::GPXWaypoints, gpxdata.waypoints);

        self.send("compute elevation");
        let data = BackendData {
            track,
            parameters,
            packet_provider: point_collection,
        };
        self.send("update waypoints");
        self.backend_data = Some(data);

        // this updates the collection, too
        self.set_user_step_options(&self.get_parameters().user_steps_options);
        self.send("content loaded");
        Ok(())
    }

    pub async fn load_contents(&mut self, contents: &Vec<Vec<u8>>) -> Result<(), TrackError> {
        self.send("read gpx");
        let track_parts = self.load_track_parts(contents).await?;
        self.load_ordered(&track_parts).await
    }
    pub async fn load_filename(&mut self, filename: &str) -> Result<(), TrackError> {
        let mut f = std::fs::File::open(filename).unwrap();
        let mut buffer = Vec::new();
        // read the whole file
        use std::io::prelude::*;
        f.read_to_end(&mut buffer).unwrap();
        self.load_content(&buffer).await
    }
}

// methods that access BackendData (should not be used in bridge)
impl Backend {
    pub fn d(&self) -> &BackendData {
        self.backend_data.as_ref().unwrap()
    }
    fn dmut(&mut self) -> &mut BackendData {
        self.backend_data.as_mut().unwrap()
    }

    pub fn make_segment_data(&self, segment: &Segment) -> SegmentData {
        SegmentData::new(
            segment,
            self.d().track.clone(),
            self.d().packet_provider.clone(),
            self.d().parameters.clone(),
        )
    }

    pub fn get_parameters(&self) -> Parameters {
        self.d().parameters.clone()
    }

    pub fn set_parameters(&mut self, parameters: &Parameters) {
        self.dmut().parameters = parameters.clone();
        if self.d().parameters.segment_overlap > self.d().parameters.segment_length {
            assert!(false);
        }

        // update user steps
        {
            let mut locked = self.d().packet_provider.write().unwrap();
            let usersteps =
                make_points::user_points(&self.d().track, &self.d().parameters.user_steps_options);
            locked.collection.import_other(&Kind::UserStep, usersteps);
        }
    }

    pub fn get_points(&self, segment: &Segment, kinds: Kinds) -> Vec<InputPoint> {
        let mut points = Vec::new();
        let range = self.d().track.subrange(segment.start, segment.end);
        if kinds.is_empty() {
            return Vec::new();
        }

        for kind in &kinds {
            let kpoints = self
                .d()
                .packet_provider
                .read()
                .unwrap()
                .collection
                .get_vector(kind);
            let mut copy = kpoints.clone();
            copy.retain(|w| {
                if w.kind() == Kind::Controls && kinds.contains(&Kind::GPXWaypoints) {
                    // When a control point has been created using a GPX waypoint,
                    // and the waypoint is also going to be in the returned list (not cleanly
                    // done), then discard this control, show only the GPX waypoint.
                    // Otherwise, the informations are shown twice (e.g. in tables).
                    // TODO: ensure that this gpx waypoint is really going to the caller.
                    if !w.control_waypoint_origin_id().is_empty() {
                        return false;
                    }
                }
                is_close_to_track(&w)
                    && range.contains(&w.track_projections.first().unwrap().track_index)
            });
            points.extend_from_slice(&copy);
        }
        log::info!(
            "segment: {} [{:.1}:{:.1}] export {} waypoints",
            segment.id,
            segment.start / 1000f64,
            segment.end / 1000f64,
            points.len()
        );
        points
    }

    pub fn export_points(&self, points: &Vec<InputPoint>) -> Waypoints {
        // TODO: handle multiple projections.
        let mut ret = Waypoints::new();
        let projections = InputPoint::flatten_projections(&points);
        for (index, projection) in projections {
            ret.push(points[index].waypoint(&projection));
        }
        debug_assert!(points.len() <= ret.len());
        WaypointInfo::make_waypoint_infos(&mut ret, &self.d().track, &self.d().parameters);
        ret
    }

    pub fn get_waypoints(&self, segment: &Segment, kinds: Kinds) -> Vec<Waypoint> {
        self.export_points(&self.get_points(&segment, kinds))
    }

    pub async fn generatePdf(&self) -> Vec<u8> {
        let typbytes = render::make_typst_document(self);
        let ret = pdf::compile(&typbytes, self.get_parameters().debug).await;
        log::info!("generated {} pdf bytes", ret.len());
        ret
    }
    pub fn generateGpx(&self) -> Vec<u8> {
        let mut gpxpoints = Vec::new();
        let v = self
            .d()
            .packet_provider
            .read()
            .unwrap()
            .collection
            .get_vector(&Kind::UserStep);
        v.iter().for_each(|p| {
            assert!(!p.track_projections.is_empty());
        });
        gpxpoints.extend_from_slice(&v);
        let waypoints = self.export_points(&gpxpoints);
        gpxexport::generate(&self.d().track, &waypoints)
    }
    pub async fn generateZip(&self) -> Vec<u8> {
        let gpx = self.generateGpx();
        let pdf = self.generatePdf().await;
        zipexport::generate(&gpx, &pdf)
    }

    pub fn set_user_step_options(&mut self, options: &UserStepsOptions) {
        self.dmut().parameters.user_steps_options = options.clone();
        // update user steps
        {
            let mut locked = self.d().packet_provider.write().unwrap();
            let usersteps =
                make_points::user_points(&self.d().track, &self.d().parameters.user_steps_options);
            locked.collection.import_other(&Kind::UserStep, usersteps);
        }
    }

    pub fn set_profile_indications(&mut self, indications: &Vec<ProfileIndication>) {
        self.dmut().parameters.profile_options.elevation_indicators = indications.clone();
    }

    pub fn set_userstep_gpx_name_format(&mut self, format: &String) {
        self.dmut().parameters.user_steps_options.gpx_name_format = format.clone();
    }

    pub fn set_control_gpx_name_format(&mut self, format: &String) {
        self.dmut().parameters.control_gpx_name_format = format.clone();
    }

    pub fn setStartTime(&mut self, rfc3339: String) {
        self.dmut().parameters.start_time = rfc3339;
    }
    pub fn setSpeed(&mut self, s: f64) {
        self.dmut().parameters.speed = s;
    }
    pub fn setSegmentLength(&mut self, length: f64) {
        self.dmut().parameters.segment_length = length;
    }

    pub fn segments(&self) -> Vec<Segment> {
        let mut ret = Vec::new();

        let mut start = 0f64;
        let mut k = 0usize;
        loop {
            let end = start + self.d().parameters.segment_length;
            ret.push(Segment {
                id: k as i32,
                start,
                end,
            });
            if end > self.d().track.total_distance() {
                break;
            }
            start += self.d().parameters.segment_length - self.d().parameters.segment_overlap;
            k = k + 1;
        }
        ret
    }

    pub fn trackSegment(&self) -> Segment {
        let start = 0f64;
        let end = self.d().track.total_distance();
        Segment { id: -1, start, end }
    }

    pub fn render_segment_simple(
        &self,
        segment: &Segment,
        size: &IntegerSize2D,
        kinds: Kinds,
        function: RenderFunction,
    ) -> String {
        let input = RenderInput {
            kinds,
            function,
            size: (size.width, size.height),
        };
        self.render_segment(segment, &vec![input]).remove(0).svg
    }

    pub fn render_segment(
        &self,
        segment: &Segment,
        render_inputs: &Vec<RenderInput>,
    ) -> Vec<RenderOutput> {
        if render_inputs.len() == 2 {
            let sizes: BTreeMap<_, _> = render_inputs
                .iter()
                .map(|input| (input.function.clone(), input.size))
                .collect();
            let kinds = render_inputs.first().unwrap().kinds.clone();
            match (
                sizes.get(&RenderFunction::Map),
                sizes.get(&RenderFunction::Profile),
            ) {
                (Some(msize), Some(psize)) => {
                    let map_size = IntegerSize2D::new(msize.0, msize.1);
                    let profile_size = IntegerSize2D::new(psize.0, psize.1);
                    return self.render_segment_map_profile(
                        segment,
                        &map_size,
                        &profile_size,
                        kinds,
                    );
                }
                _ => {}
            }
        }

        let data = self.make_segment_data(segment);
        let mut ret = Vec::new();
        for render_input in render_inputs {
            let size = IntegerSize2D::new(render_input.size.0, render_input.size.1);
            data.preload(&render_input.function, &render_input.kinds, &size);
            let render_result = match render_input.function {
                RenderFunction::Profile => data.render_profile(&size, &render_input.kinds),
                RenderFunction::Map => data.render_map(&size, &render_input.kinds),
                RenderFunction::Wheel => {
                    let time_parameters = wheel::model::TimeParameters {
                        start: parameters::parse_time(&self.d().parameters.start_time),
                        speed: self.d().parameters.speed,
                        total_distance: self.d().track.total_distance(),
                    };
                    let mut model = wheel::model::WheelModel::new(&time_parameters);
                    model.add_points(&data, &render_input.kinds);
                    wheel::render(&size, &model)
                }
                RenderFunction::WheelPages => {
                    let time_parameters = wheel::model::TimeParameters {
                        start: parameters::parse_time(&self.d().parameters.start_time),
                        speed: self.d().parameters.speed,
                        total_distance: self.d().track.total_distance(),
                    };
                    let mut model = wheel::model::WheelModel::new(&time_parameters);
                    model.add_points(&data, &render_input.kinds);
                    model.add_pages(&self.segments());
                    wheel::render(&size, &model)
                }
                RenderFunction::Unknown => {
                    panic!("The render function is not set. Bye.");
                }
            };
            log::info!(
                "done - render_segment_what:{} {:?}",
                segment.id,
                render_input.function
            );
            let points = render_result.rendered_input_points_for_table();
            ret.push(RenderOutput {
                svg: render_result.svg,
                render_input: render_input.clone(),
                error: None,
                waypoints: waypoint::table(&data, &points),
            });
        }
        ret
    }

    pub fn render_segment_map_profile(
        &self,
        segment: &Segment,
        map_size: &IntegerSize2D,
        profile_size: &IntegerSize2D,
        kinds: Kinds,
    ) -> Vec<RenderOutput> {
        log::info!(
            "start - render_segment_profile_map:{} map_size:{}x{} profile_size:{}x{}",
            segment.id,
            map_size.width,
            map_size.height,
            profile_size.width,
            profile_size.height
        );
        let data = self.make_segment_data(segment);
        data.preload_map_profile(&kinds, map_size, profile_size);
        let (result_map, result_profile) = data.render_map_profile(map_size, profile_size, &kinds);
        let mut ret = Vec::new();
        ret.push((RenderFunction::Map, map_size, result_map));
        ret.push((RenderFunction::Profile, profile_size, result_profile));
        ret.iter()
            .map(|(function, size, result)| {
                debug_assert_eq!(result.parameters.function, function.clone());
                let points = result.rendered_input_points_for_table();
                RenderOutput {
                    svg: result.svg.clone(),
                    render_input: RenderInput {
                        kinds: kinds.clone(),
                        function: function.clone(),
                        size: (size.width, size.height),
                    },
                    error: None,
                    waypoints: waypoint::table(&data, &points),
                }
            })
            .collect()
    }

    pub fn segment_statistics(&self, segment: &Segment) -> SegmentStatistics {
        let range = self.d().track.subrange(segment.start, segment.end);
        assert!(range.end > 0);
        SegmentStatistics {
            length: self.d().track.distance(range.end - 1) - self.d().track.distance(range.start),
            elevation_gain: self.d().track.elevation_gain_on_range(&range),
            distance_start: self.d().track.distance(range.start),
            distance_end: self.d().track.distance(range.end - 1),
        }
    }

    pub fn statistics(&self) -> SegmentStatistics {
        let range = 0..self.d().track.len();
        assert!(range.end > 0);
        SegmentStatistics {
            length: self.d().track.distance(range.end - 1) - self.d().track.distance(range.start),
            elevation_gain: self.d().track.elevation_gain_on_range(&range),
            distance_start: self.d().track.distance(range.start),
            distance_end: self.d().track.distance(range.end - 1),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        backend::Backend,
        math::IntegerSize2D,
        parameters::{self, ControlSource, ProfileIndication, RenderFunction},
        point_collection::{self, Kind},
        wheel,
    };
    static START_TIME: &'static str = "1985-04-12T06:05:00.00Z";

    async fn load_test_data() -> Backend {
        let mut backend = Backend::make();
        backend
            .load_filename("data/blackforest.gpx")
            .await
            .expect("fail");
        backend.load_osm().await.unwrap();
        backend
            .load_controls(ControlSource::Waypoints)
            .await
            .unwrap();
        backend
    }

    #[tokio::test]
    async fn svg_profile() {
        let _ = env_logger::try_init();
        let mut backend = load_test_data().await;

        let mut parameters = backend.get_parameters();
        parameters.start_time = START_TIME.to_string();
        parameters.user_steps_options.step_distance = Some((10_000) as f64);
        parameters.profile_options.max_area_ratio = 0.20f64;
        parameters.profile_options.elevation_indicators = vec![ProfileIndication::NumericSlope];

        backend.set_parameters(&parameters);

        let segments = backend.segments();
        let mut ok_count = 0;
        let profile_size = IntegerSize2D::new(1420, 400);
        for segment in &segments {
            let result = backend.render_segment_simple(
                &segment,
                &profile_size,
                point_collection::allkinds(),
                RenderFunction::Profile,
            );

            let reffilename = std::format!("data/ref/profile-{}.svg", segment.id);
            println!("test {}", reffilename);
            let reference_svg = if std::fs::exists(&reffilename).unwrap() {
                std::fs::read_to_string(&reffilename).unwrap()
            } else {
                String::new()
            };
            if reference_svg == result {
                ok_count += 1;
            }
            let tmpfilename = std::format!("/tmp/profile-{}.svg", segment.id);
            std::fs::write(&tmpfilename, result.clone()).unwrap();
            if reference_svg != result {
                println!("test failed: {} {}", tmpfilename, reffilename);
            }
        }
        assert!(ok_count == segments.len());
    }

    #[tokio::test]
    async fn svg_segment_wheel() {
        let _ = env_logger::try_init();
        let mut backend = load_test_data().await;
        let mut parameters = backend.get_parameters();
        parameters.start_time = START_TIME.to_string();
        parameters.user_steps_options.step_distance = Some((3_000) as f64);
        parameters.segment_length = 55000f64;
        parameters.segment_overlap = 5000f64;
        backend.set_parameters(&parameters);
        let reffilename = std::format!("data/ref/segment-wheel.svg");
        let data = if std::fs::exists(&reffilename).unwrap() {
            std::fs::read_to_string(&reffilename).unwrap()
        } else {
            String::new()
        };
        let segment = backend.trackSegment();
        let sgdata = backend.make_segment_data(&segment);
        let segments = backend.segments();
        let time_parameters = wheel::model::TimeParameters {
            start: parameters::parse_time(&parameters.start_time),
            speed: parameters.speed,
            total_distance: backend.d().track.total_distance(),
        };
        let mut model = wheel::model::WheelModel::new(&time_parameters);
        model.add_pages(&segments);
        model.add_points(&sgdata, &point_collection::allkinds());
        let result = wheel::render(&IntegerSize2D::new(400, 400), &model);

        let tmpfilename = std::format!("/tmp/segment-wheel.svg");
        std::fs::write(&tmpfilename, result.svg.clone()).unwrap();
        if data != result.svg {
            println!("test failed: {} {}", tmpfilename, reffilename);
            assert!(false);
        }
    }

    #[tokio::test]
    async fn test_get_waypoints() {
        let _ = env_logger::try_init();
        let backend = load_test_data().await;
        let fseg = backend.trackSegment();
        let seg = backend.make_segment_data(&fseg);
        let controls = seg.controls();
        let len = controls.len();
        assert!(len > 0);
        let kinds = std::collections::HashSet::from([Kind::Controls]);
        let waypoints = backend.get_waypoints(&fseg, kinds);
        assert!(!waypoints.is_empty());
        for waypoint in waypoints {
            log::info!("gpx name={}", waypoint.info.unwrap().gpx_name);
        }
    }

    #[tokio::test]
    async fn svg_large_map() {
        let _ = env_logger::try_init();
        let mut backend = load_test_data().await;
        let mut parameters = backend.get_parameters();
        parameters.start_time = START_TIME.to_string();
        parameters.user_steps_options.step_distance = Some((10_000) as f64);
        parameters.map_options.max_area_ratio = 0.15f64;
        backend.set_parameters(&parameters);

        let segment = &backend.trackSegment();
        let map_size = IntegerSize2D::new(800, 800);
        let result = backend.render_segment_simple(
            &segment,
            &map_size,
            point_collection::allkinds(),
            RenderFunction::Map,
        );
        let reffilename = std::format!("data/ref/largemap.svg");
        println!("test {}", reffilename);
        let refdata = if std::fs::exists(&reffilename).unwrap() {
            std::fs::read_to_string(&reffilename).unwrap()
        } else {
            String::new()
        };
        let tmpfilename = std::format!("/tmp/largemap.svg");
        std::fs::write(&tmpfilename, result.clone()).unwrap();
        if refdata != result {
            println!("test failed: {} {}", tmpfilename, reffilename);
            assert!(false);
        }
    }

    #[tokio::test]
    async fn svg_map() {
        let _ = env_logger::try_init();
        let mut backend = load_test_data().await;
        let mut parameters = backend.get_parameters();
        parameters.start_time = START_TIME.to_string();
        parameters.user_steps_options.step_distance = Some((10_000) as f64);
        parameters.map_options.max_area_ratio = 0.15f64;
        backend.set_parameters(&parameters);

        let segments = backend.segments();
        let map_size = IntegerSize2D::new(400, 400);

        let mut ok_count = 0;
        for (_idx, segment) in segments.iter().enumerate() {
            let result = backend.render_segment_simple(
                &segment,
                &map_size,
                point_collection::allkinds(),
                RenderFunction::Map,
            );

            let reffilename = std::format!("data/ref/map-{}.svg", segment.id);
            println!("test {}", reffilename);
            let refdata = if std::fs::exists(&reffilename).unwrap() {
                std::fs::read_to_string(&reffilename).unwrap()
            } else {
                String::new()
            };
            if refdata == result {
                ok_count += 1;
            }
            let tmpfilename = std::format!("/tmp/map-{}.svg", segment.id);
            std::fs::write(&tmpfilename, result.clone()).unwrap();
            if refdata != result {
                println!("test failed: {} {}", tmpfilename, reffilename);
            }
        }
        assert!(ok_count == segments.len());
    }

    #[tokio::test]
    async fn gpx() {
        let _ = env_logger::try_init();
        let mut backend = load_test_data().await;
        let mut parameters = backend.get_parameters();
        parameters.start_time = START_TIME.to_string();
        parameters.user_steps_options.step_distance = Some((10_000) as f64);
        parameters.map_options.max_area_ratio = 0.15f64;
        backend.set_parameters(&parameters);
        let svg = backend.generateGpx();
        let reffilename = std::format!("data/ref/route.gpx");
        println!("test {}", reffilename);
        let data = if std::fs::exists(&reffilename).unwrap() {
            std::fs::read(&reffilename).unwrap()
        } else {
            Vec::new()
        };
        let tmpfilename = std::format!("/tmp/route.gpx");
        std::fs::write(&tmpfilename, svg.clone()).unwrap();
        if data != svg {
            println!("test failed: {} {}", tmpfilename, reffilename);
            assert!(false);
        }
    }

    #[tokio::test]
    async fn reorder() {
        let _ = env_logger::try_init();
        let bytes = {
            let mut f = std::fs::File::open("data/ref/karl-400.gpx").unwrap();
            let mut buffer = Vec::new();
            // read the whole file
            use std::io::prelude::*;
            f.read_to_end(&mut buffer).unwrap();
            buffer
        };
        let mut backend = Backend::make();
        let mut track_parts = backend.load_track_parts(&vec![bytes]).await.unwrap();
        let result = backend.load_ordered(&track_parts).await;
        assert!(result.is_ok());
        assert!(backend.loaded());
        let s1 = backend.statistics();
        log::trace!(
            "dstart={:.1} dend={:.1} km={:.1}",
            s1.distance_start,
            s1.distance_end,
            s1.length / 1000f64
        );

        track_parts.insert(0, track_parts.last().unwrap().clone());
        track_parts.remove(track_parts.len() - 1);
        let result = backend.load_ordered(&track_parts).await;
        assert!(result.is_ok());
        assert!(backend.loaded());
        let s2 = backend.statistics();
        log::trace!(
            "dstart={:.1} dend={:.1} km={:.1}",
            s2.distance_start,
            s2.distance_end,
            s2.length / 1000f64
        );
        let d = (s1.length - s2.length).abs();
        log::trace!("d={}", d);
        // there is a 65m distance between the end of K4-K5 and the beginning of K5-Ziel.
        assert!(d < 100f64);
    }
}
