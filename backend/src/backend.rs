#![allow(non_snake_case)]

use crate::backend_data::BackendData;
use crate::error;
use crate::error::TrackError;
use crate::event;
use crate::gpsdata;
use crate::gpsdata::GpxData;
use crate::math::IntegerSize2D;
use crate::osm;
use crate::osm::DownloadSideData;
use crate::parameters::current_time_as_string;
use crate::parameters::Parameters;
use crate::parameters::RenderFunction;
use crate::parameters::RenderInput;
use crate::parameters::RenderOutput;
use crate::parameters::TrackPart;
use crate::point_collection::Kind;
use crate::point_collection::Kinds;
use crate::point_collection::PacketProvider;
use crate::speed;
use crate::track::Track;
use crate::trackfile;
use crate::trackfile::SmallParameters;
use crate::trackfile::TrackFile;
use crate::waypoint::Waypoint;
use std::sync::RwLock;

pub type Segment = crate::segment::Segment;
pub type SegmentStatistics = crate::segment::SegmentStatistics;
pub use crate::event::Sender;
pub type SenderHandler = crate::event::SenderHandler;
pub type SenderHandlerLock = crate::event::SenderHandlerLock;

use tokio_util::sync::CancellationToken;

pub struct Backend {
    backend_data: RwLock<Option<BackendData>>,
    gpxdata: RwLock<Option<GpxData>>,
    osm_cancel_token: RwLock<Option<CancellationToken>>,
    pub sender: SenderHandlerLock,
}

impl Backend {
    pub fn make() -> Backend {
        Backend {
            backend_data: RwLock::new(None),
            gpxdata: RwLock::new(None),
            osm_cancel_token: RwLock::new(None),
            sender: RwLock::new(None),
        }
    }
    pub fn loaded(&self) -> bool {
        self.backend_data.read().unwrap().is_some()
    }
    pub fn unload(&mut self) {
        *self.backend_data.write().unwrap() = None;
    }
    pub fn set_sink(&mut self, sink: SenderHandler) {
        self.sender = RwLock::new(Some(sink));
    }
    pub fn send(&self, data: &str) {
        log::trace!("event:{}", data);
        // if there is no sender (sender is an Option)
        // => do nothing
        if self.sender.read().unwrap().is_none() {
            return;
        }
        event::send_worker(&self.sender, data);
    }

    pub async fn cancel_osm(&self) {
        self.send("osm:cancel");
        {
            // .take() moves the value out and leaves None in its place
            if let Some(token) = self.osm_cancel_token.write().unwrap().take() {
                token.cancel();
            } else {
                log::error!("cannot cancel osm data (probably not running)");
            }
        }
    }

    async fn load_osm(&self, try_download: bool) -> Result<usize, TrackError> {
        {
            let lock = self.osm_cancel_token.read().unwrap();
            match *lock {
                Some(_) => {
                    return Err(TrackError::OSMDownloadAlreadyRunning);
                }
                None => {}
            }
        }

        let token = CancellationToken::new();
        {
            let mut lock = self.osm_cancel_token.write().unwrap();
            *lock = Some(token.clone());
        }
        let side = DownloadSideData {
            logger: &self.sender,
            cancel_token: &token,
        };
        let track = self
            .backend_data
            .read()
            .unwrap()
            .as_ref()
            .unwrap()
            .track
            .clone();
        let result = osm::download_for_track(&track, &side, try_download).await;
        {
            let mut lock = self.osm_cancel_token.write().unwrap();
            *lock = None;
        }

        let (osmpoints, missing_box_count) = match result {
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
        self.send("osm:sort");
        self.backend_data
            .write()
            .unwrap()
            .as_mut()
            .unwrap()
            .load_osm(osmpoints);
        self.send("osm:done");
        Ok(missing_box_count)
    }

    pub async fn load_osm_with_download(&self) -> Result<usize, TrackError> {
        let try_dowload = true;
        self.load_osm(try_dowload).await
    }

    pub async fn load_osm_without_download(&self) -> Result<usize, TrackError> {
        let try_dowload = true;
        self.load_osm(!try_dowload).await
    }

    pub fn load_controls(&self) -> Result<usize, TrackError> {
        self.backend_data
            .write()
            .unwrap()
            .as_mut()
            .unwrap()
            .load_controls()
    }

    pub fn make_control_at_waypoint(&self, waypoint: &Waypoint, on: bool) {
        self.backend_data
            .write()
            .unwrap()
            .as_mut()
            .unwrap()
            .make_control_at_waypoint(waypoint, on)
    }

    pub fn allowed_speeds(&self) -> Vec<String> {
        let distance = self
            .backend_data
            .read()
            .unwrap()
            .as_ref()
            .unwrap()
            .track
            .total_distance();
        speed::allowed_speeds(distance)
    }

    pub fn get_parameters(&self) -> Parameters {
        self.backend_data
            .read()
            .unwrap()
            .as_ref()
            .unwrap()
            .parameters
            .clone()
    }

    pub fn set_control_time(&self, waypoint: &Waypoint, time: &Option<String>) -> bool {
        self.backend_data
            .write()
            .unwrap()
            .as_mut()
            .unwrap()
            .set_control_time(waypoint, time)
    }

    pub fn load_content(&mut self, content: &Vec<u8>) -> Result<(), TrackError> {
        self.load_contents(&vec![content.clone()])
    }

    pub fn load_contents(&mut self, contents: &Vec<Vec<u8>>) -> Result<(), TrackError> {
        self.send("gpx:read");
        let track_parts = self.load_track_parts(contents)?;
        self.load_ordered(&track_parts)
    }

    pub fn load_track_parts(&self, contents: &Vec<Vec<u8>>) -> Result<Vec<TrackPart>, TrackError> {
        self.send("gpx:read");
        let gpxdata = gpsdata::GpxData::read_contents(contents)?;
        let parts = gpxdata.track_parts();
        {
            let mut locked = self.gpxdata.write().unwrap();
            *locked = Some(gpxdata);
        }
        Ok(parts)
    }

    pub fn load_ordered(&mut self, parts: &Vec<TrackPart>) -> Result<(), TrackError> {
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
        let mut point_collection = PacketProvider::new();
        point_collection
            .collection
            .import_other(&Kind::GPXWaypoints, gpxdata.waypoints);

        let data = BackendData {
            track,
            parameters,
            packet_provider: point_collection,
        };
        *self.backend_data.write().unwrap() = Some(data);

        // point collection doest not include OSM and user steps
        // => OSM are handled in load_osm
        // => user steps and handled in set_parameters.
        self.set_parameters(&self.get_parameters());
        self.send("gpx:done");
        Ok(())
    }

    pub async fn create_trackfile(&self) -> Result<TrackFile, TrackError> {
        log::trace!("[create_trackfile]{}", self.loaded());
        assert!(self.loaded());
        let name = {
            let lock = self.backend_data.read().unwrap();
            let track = &lock.as_ref().unwrap().track;
            debug_assert!(!track.parts.is_empty());
            track.parts.first().as_ref().unwrap().name.clone()
        };
        log::trace!("[create_trackfile]{}", self.loaded());
        let track = self.trackSegment();
        let stats = self.segment_statistics(&track);
        log::trace!("[create_trackfile]{}", self.loaded());
        let trackFile = match trackfile::SmallParameters::create(&name, &stats).await {
            Ok(trackfile) => trackfile,
            Err(e) => {
                log::error!("write user data failed: {:?}", e);
                return Err(TrackError::IOError.into());
            }
        };
        log::trace!("[create_trackfile]{}", self.loaded());
        let _ = self.save_gpxdata(&trackFile).await;
        let _ = self.save_small_parameters(&trackFile).await;
        log::trace!("[create_trackfile]{}", self.loaded());
        assert!(self.loaded());
        Ok(trackFile)
    }

    pub async fn save_gpxdata(&self, trackfile: &TrackFile) -> Result<(), TrackError> {
        let data = self
            .backend_data
            .read()
            .unwrap()
            .as_ref()
            .unwrap()
            .track_dataset();
        match trackfile::write_trackdata(&trackfile, &data).await {
            Ok(()) => Ok(()),
            Err(e) => {
                log::error!("write user data failed: {:?}", e);
                Err(TrackError::IOError.into())
            }
        }
    }

    pub async fn trackfiles(&self) -> Result<Vec<TrackFile>, TrackError> {
        trackfile::SmallParameters::list()
            .await
            .map_err(|_| TrackError::IOError.into())
    }

    pub async fn remove_trackfile(&self, trackfile: &TrackFile) -> Result<(), TrackError> {
        SmallParameters::remove(trackfile)
            .await
            .map_err(|_| TrackError::IOError.into())
    }

    pub async fn update_trackfile_name(
        &self,
        trackfile: &TrackFile,
        name: &str,
    ) -> Result<TrackFile, TrackError> {
        let mut small_parameters = self
            .backend_data
            .read()
            .unwrap()
            .as_ref()
            .unwrap()
            .small_parameters_with_trackfile(trackfile);
        small_parameters
            .update_name(name)
            .await
            .map_err(|_| TrackError::IOError.into())
    }

    pub async fn read_trackfile(&self, trackfile: &TrackFile) -> Result<(), TrackError> {
        let mut gpxdata = match trackfile::read_trackdata(trackfile).await {
            Some(data) => data,
            None => return Err(TrackError::IOError.into()),
        };
        let track_data = Track::from_tracks(&gpxdata.tracks)?;
        let track = std::sync::Arc::new(track_data);
        for p in &mut gpxdata.waypoints {
            track.project_point(p);
        }
        let smalldata = match trackfile::SmallParameters::read(trackfile).await {
            Some(data) => data,
            None => return Err(TrackError::IOError.into()),
        };
        let mut packet_provider = PacketProvider::new();
        packet_provider
            .collection
            .import_other(&Kind::GPXWaypoints, gpxdata.waypoints);
        packet_provider
            .collection
            .import_other(&Kind::Controls, smalldata.controls);

        let data = BackendData {
            track,
            parameters: smalldata.parameters.clone(),
            packet_provider,
        };
        *self.backend_data.write().unwrap() = Some(data);
        Ok(())
    }

    pub async fn save_small_parameters(&self, trackfile: &TrackFile) -> Result<(), TrackError> {
        let mut small_parameters = self
            .backend_data
            .read()
            .unwrap()
            .as_ref()
            .unwrap()
            .small_parameters_with_trackfile(trackfile);
        small_parameters.trackfile.last_modified = current_time_as_string();
        small_parameters.trackfile.start_time = small_parameters.parameters.start_time.clone();
        match small_parameters.write().await {
            Ok(()) => Ok(()),
            Err(e) => {
                log::error!("write user data failed: {:?}", e);
                Err(TrackError::IOError.into())
            }
        }
    }

    pub fn load_filename(&mut self, filename: &str) -> Result<(), TrackError> {
        let mut f = std::fs::File::open(filename).unwrap();
        let mut buffer = Vec::new();
        // read the whole file
        use std::io::prelude::*;
        f.read_to_end(&mut buffer).unwrap();
        self.load_content(&buffer)
    }

    pub fn track(&self) -> Track {
        (*self.backend_data.read().unwrap().as_ref().unwrap().track).clone()
    }
    pub fn get_waypoints(&self, segment: &Segment, kinds: &Kinds) -> Vec<Waypoint> {
        self.backend_data
            .read()
            .unwrap()
            .as_ref()
            .unwrap()
            .get_waypoints(segment, kinds)
    }
    pub fn render_segment_map_profile(
        &self,
        segment: &Segment,
        map_size: &IntegerSize2D,
        profile_size: &IntegerSize2D,
        kinds: Kinds,
    ) -> Vec<RenderOutput> {
        self.backend_data
            .read()
            .unwrap()
            .as_ref()
            .unwrap()
            .render_segment_map_profile(segment, map_size, profile_size, kinds)
    }
    pub fn render_segment_simple(
        &self,
        segment: &Segment,
        size: &IntegerSize2D,
        kinds: Kinds,
        function: RenderFunction,
    ) -> String {
        self.backend_data
            .read()
            .unwrap()
            .as_ref()
            .unwrap()
            .render_segment_simple(segment, size, kinds, function)
    }
    pub fn render_segment(
        &self,
        segment: &Segment,
        render_inputs: &Vec<RenderInput>,
    ) -> Vec<RenderOutput> {
        self.backend_data
            .read()
            .unwrap()
            .as_ref()
            .unwrap()
            .render_segment(segment, render_inputs)
    }
    pub fn segments(&self) -> Vec<Segment> {
        self.backend_data
            .read()
            .unwrap()
            .as_ref()
            .unwrap()
            .segments()
    }
    pub fn trackSegment(&self) -> Segment {
        self.backend_data
            .read()
            .unwrap()
            .as_ref()
            .unwrap()
            .trackSegment()
    }
    pub fn set_parameters(&self, parameters: &Parameters) {
        self.backend_data
            .write()
            .unwrap()
            .as_mut()
            .unwrap()
            .set_parameters(parameters)
    }

    pub fn set_start_time(&mut self, rfc3339: String) {
        self.backend_data
            .write()
            .unwrap()
            .as_mut()
            .unwrap()
            .set_start_time(rfc3339);
    }

    pub fn set_segment_length(&mut self, length: f64) {
        self.backend_data
            .write()
            .unwrap()
            .as_mut()
            .unwrap()
            .set_segment_length(length);
    }

    pub fn statistics(&self) -> SegmentStatistics {
        self.backend_data
            .read()
            .unwrap()
            .as_ref()
            .unwrap()
            .statistics()
    }

    pub fn segment_statistics(&self, segment: &Segment) -> SegmentStatistics {
        self.backend_data
            .read()
            .unwrap()
            .as_ref()
            .unwrap()
            .segment_statistics(segment)
    }

    pub fn generateZip(&self, kinds: &Kinds) -> Result<Vec<u8>, TrackError> {
        self.backend_data
            .read()
            .unwrap()
            .as_ref()
            .unwrap()
            .generateZip(kinds)
    }
    pub fn generatePdf(&self, kinds: &Kinds) -> Result<Vec<u8>, TrackError> {
        self.backend_data
            .read()
            .unwrap()
            .as_ref()
            .unwrap()
            .generatePdf(kinds)
    }

    pub async fn init_pdf_fonts() -> Result<(), TrackError> {
        crate::pdf::init_fonts().await
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        backend::Backend,
        math::IntegerSize2D,
        parameters::{ProfileIndication, RenderFunction},
        point_collection::{self},
    };
    static START_TIME: &'static str = "1985-04-12T06:05:00.00Z";
    static BLACK_FOREST: &'static str = "data/blackforest.gpx";

    async fn load_test_data(filename: &str) -> Backend {
        let mut backend = Backend::make();
        backend.load_filename(filename).expect("fail");
        backend.load_osm_without_download().await.unwrap();
        backend.load_controls().unwrap();
        backend
    }

    #[tokio::test]
    async fn svg_profile() {
        let _ = env_logger::try_init();
        let backend = load_test_data(BLACK_FOREST).await;

        let mut parameters = backend.get_parameters();
        parameters.start_time = START_TIME.to_string();
        parameters.user_steps_options.step_distance = Some((10_000) as f64);
        parameters.profile_options.elevation_indicators = vec![ProfileIndication::NumericSlope];
        parameters.debug = true;

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
    async fn svg_large_map() {
        let _ = env_logger::try_init();
        let backend = load_test_data(BLACK_FOREST).await;
        let mut parameters = backend.get_parameters();
        parameters.start_time = START_TIME.to_string();
        parameters.user_steps_options.step_distance = Some((10_000) as f64);
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
        let backend = load_test_data(BLACK_FOREST).await;
        let mut parameters = backend.get_parameters();
        parameters.start_time = START_TIME.to_string();
        parameters.user_steps_options.step_distance = Some((10_000) as f64);
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
        let mut track_parts = backend.load_track_parts(&vec![bytes]).unwrap();
        let result = backend.load_ordered(&track_parts);
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
        let result = backend.load_ordered(&track_parts);
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

    #[tokio::test]
    async fn persist() {
        let _ = env_logger::try_init();
        // see filesystem.rs
        // DATA_DIR=data/ref/persist/share1
        unsafe {
            std::env::set_var("DATA_DIR", "data/ref/persist/share1");
        }
        let backend = Backend::make();
        for trackfile in backend.trackfiles().await.unwrap() {
            let _ = backend.read_trackfile(&trackfile).await;
            let svg = backend.render_segment_simple(
                &backend.trackSegment(),
                &IntegerSize2D::new(2000, 1000),
                point_collection::allkinds(),
                RenderFunction::Profile,
            );
            assert!(!svg.is_empty());
        }
    }
}
