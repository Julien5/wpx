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
use crate::parameters::Parameters;
use crate::parameters::RenderFunction;
use crate::parameters::RenderOutput;
use crate::parameters::TrackPart;
use crate::point_collection::Kind;
use crate::point_collection::Kinds;
use crate::point_collection::PacketProvider;
use crate::track::Track;
use crate::waypoint::Waypoint;

pub type Segment = crate::segment::Segment;
pub type SegmentStatistics = crate::segment::SegmentStatistics;
pub use crate::event::Sender;
pub type SenderHandler = crate::event::SenderHandler;
pub type SenderHandlerLock = crate::event::SenderHandlerLock;

use tokio_util::sync::CancellationToken;

pub struct Backend {
    backend_data: Option<BackendData>,
    gpxdata: std::sync::RwLock<Option<GpxData>>,
    osm_cancel_token: std::sync::RwLock<Option<CancellationToken>>,
    pub sender: SenderHandlerLock,
}

impl Backend {
    pub fn make() -> Backend {
        Backend {
            backend_data: None,
            gpxdata: std::sync::RwLock::new(None),
            osm_cancel_token: std::sync::RwLock::new(None),
            sender: std::sync::RwLock::new(None),
        }
    }
    pub fn loaded(&self) -> bool {
        self.backend_data.is_some()
    }
    pub fn unload(&mut self) {
        self.backend_data = None;
    }
    pub fn set_sink(&mut self, sink: SenderHandler) {
        self.sender = std::sync::RwLock::new(Some(sink));
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

    pub async fn load_osm(&mut self) -> Result<(), TrackError> {
        {
            let lock = self.osm_cancel_token.read().unwrap();
            match *lock {
                Some(_) => {
                    return Err(TrackError::OSMDownloadRunning);
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
        let track = &self.backend_data.as_ref().unwrap().track;
        let result = osm::download_for_track(track, &side).await;
        {
            let mut lock = self.osm_cancel_token.write().unwrap();
            *lock = None;
        }

        let osmpoints = match result {
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
        self.backend_data.as_mut().unwrap().load_osm(osmpoints);
        Ok(())
    }

    pub fn load_controls(&self) -> Result<usize, TrackError> {
        Ok(0usize)
    }

    pub fn make_control_at_waypoint(&self, waypoint: &Waypoint, on: bool) {}

    pub fn allowed_speeds(&self) -> Vec<String> {
        //speed::allowed_speeds(self.d.track.total_distance())
        Vec::new()
    }

    pub fn get_parameters(&self) -> Parameters {
        self.backend_data.as_ref().unwrap().parameters.clone()
    }

    pub fn set_control_time(&self, waypoint: &Waypoint, time: &Option<String>) -> bool {
        true
    }

    pub fn load_content(&mut self, content: &Vec<u8>) -> Result<(), TrackError> {
        Ok(())
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
        self.backend_data = Some(data);

        // this updates the collection, too
        self.send("gpx:done");
        Ok(())
    }

    pub async fn persist_gpxdata(&self) -> Result<(), TrackError> {
        Ok(())
    }

    pub async fn has_persist(&self) -> bool {
        true
    }

    pub async fn load_persist(&mut self) -> Result<(), TrackError> {
        Ok(())
    }

    pub fn load_contents(&mut self, contents: &Vec<Vec<u8>>) -> Result<(), TrackError> {
        Ok(())
    }

    pub fn load_filename(&mut self, filename: &str) -> Result<(), TrackError> {
        Ok(())
    }

    pub fn track(&self) -> Track {
        (*self.backend_data.as_ref().unwrap().track).clone()
    }
    pub fn get_waypoints(&self, segment: &Segment, kinds: &Kinds) -> Vec<Waypoint> {
        Vec::new()
    }
    pub fn render_segment_map_profile(
        &self,
        segment: &Segment,
        map_size: &IntegerSize2D,
        profile_size: &IntegerSize2D,
        kinds: Kinds,
    ) -> Vec<RenderOutput> {
        self.backend_data
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
            .as_ref()
            .unwrap()
            .render_segment_simple(segment, size, kinds, function)
    }
    pub fn trackSegment(&self) -> Segment {
        self.backend_data.as_ref().unwrap().trackSegment()
    }
    pub fn set_parameters(&mut self, parameters: &Parameters) {
        self.backend_data
            .as_mut()
            .unwrap()
            .set_parameters(parameters)
    }
    pub fn statistics(&self) -> SegmentStatistics {
        self.backend_data.as_ref().unwrap().statistics()
    }
    pub async fn generateZip(&self, kinds: &Kinds) -> Vec<u8> {
        self.backend_data.as_ref().unwrap().generateZip(kinds).await
    }
    pub async fn generatePdf(&self, kinds: &Kinds) -> Vec<u8> {
        self.backend_data.as_ref().unwrap().generatePdf(kinds).await
    }
}
