#![allow(non_snake_case)]

use std::collections::BTreeMap;

use crate::error::Error;
use crate::gpsdata;
use crate::gpxexport;
use crate::inputpoint::*;
use crate::locate;
use crate::make_points;
use crate::math::IntegerSize2D;
use crate::osm;
use crate::parameters::Parameters;
use crate::pdf;
use crate::profile;
use crate::render;
use crate::track;
use crate::waypoint::Waypoint;
use crate::waypoint::WaypointInfo;
use crate::waypoint::Waypoints;
use crate::wheel;

pub type Segment = crate::segment::Segment;
pub type SegmentStatistics = crate::segment::SegmentStatistics;

pub struct BackendData {
    pub parameters: Parameters,
    pub track: std::sync::Arc<track::Track>,
    pub inputpoints: InputPointMaps,
}

pub trait Sender {
    fn send(&mut self, data: &String);
}

pub type SenderHandler = Box<dyn Sender + Send + Sync>;
pub type SenderHandlerLock = std::sync::RwLock<Option<SenderHandler>>;

pub struct Backend {
    backend_data: Option<BackendData>,
    pub sender: SenderHandlerLock,
}

#[cfg(not(target_arch = "wasm32"))]
async fn send_worker(handler: &SenderHandlerLock, data: &String) {
    let _ = handler.write().unwrap().as_mut().unwrap().send(&data);
}

#[cfg(target_arch = "wasm32")]
async fn send_worker(handler: &SenderHandlerLock, data: &String) {
    let _ = handler.write().unwrap().as_mut().unwrap().send(&data);
    let tick = std::time::Duration::from_millis(0);
    let _ = wasmtimer::tokio::sleep(tick).await;
}

impl Backend {
    pub fn make() -> Backend {
        Backend {
            backend_data: None,
            sender: std::sync::RwLock::new(None),
        }
    }
    pub fn d(&self) -> &BackendData {
        self.backend_data.as_ref().unwrap()
    }
    pub fn loaded(&self) -> bool {
        self.backend_data.is_some()
    }
    fn dmut(&mut self) -> &mut BackendData {
        self.backend_data.as_mut().unwrap()
    }
    pub fn set_sink(&mut self, sink: SenderHandler) {
        self.sender = std::sync::RwLock::new(Some(sink));
    }
    pub async fn send(&self, data: &String) {
        log::trace!("event:{}", data);
        if self.sender.read().unwrap().is_none() {
            return;
        }
        send_worker(&self.sender, data).await
    }

    pub fn get_parameters(&self) -> Parameters {
        self.d().get_parameters()
    }

    pub fn set_parameters(&mut self, p: &Parameters) {
        self.dmut().set_parameters(p)
    }

    pub fn segment_statistics(&self, segment: &Segment) -> SegmentStatistics {
        self.d().segment_statistics(segment)
    }

    pub fn statistics(&self) -> SegmentStatistics {
        self.d().statistics()
    }

    pub async fn generatePdf(&mut self) -> Vec<u8> {
        self.dmut().generatePdf().await
    }
    pub fn generateGpx(&mut self) -> Vec<u8> {
        self.dmut().generateGpx()
    }
    pub fn segments(&self) -> Vec<Segment> {
        self.d().segments()
    }
    pub fn trackSegment(&self) -> Segment {
        self.d().trackSegment()
    }
    pub fn render_segment_what(
        &mut self,
        segment: &Segment,
        what: &String,
        size: &IntegerSize2D,
        kinds: Kinds,
    ) -> String {
        self.dmut().render_segment_what(segment, what, size, kinds)
    }

    pub fn get_waypoints(&self, segment: &Segment, kinds: Kinds) -> Vec<Waypoint> {
        return self.d().get_waypoints(segment, kinds);
    }

    pub async fn load_content(&mut self, content: &Vec<u8>) -> Result<(), Error> {
        self.send(&"read gpx".to_string()).await;
        let gpxdata = gpsdata::read_content(content)?;
        self.send(&"download osm data".to_string()).await;
        let mut inputpoints = BTreeMap::new();
        let osmpoints = osm::download_for_track(&gpxdata.track).await;
        inputpoints.insert(InputType::OSM, osmpoints);
        inputpoints.insert(InputType::GPX, gpxdata.waypoints);

        let parameters = Parameters::default();
        self.send(&"compute elevation".to_string()).await;
        let data = BackendData {
            track: std::sync::Arc::new(gpxdata.track),
            inputpoints: InputPointMaps { maps: inputpoints },
            parameters,
        };
        self.send(&"update waypoints".to_string()).await;
        self.backend_data = Some(data);
        self.send(&"done".to_string()).await;
        Ok(())
    }

    pub async fn load_filename(&mut self, filename: &str) -> Result<(), Error> {
        let mut f = std::fs::File::open(filename).unwrap();
        let mut buffer = Vec::new();
        // read the whole file
        use std::io::prelude::*;
        f.read_to_end(&mut buffer).unwrap();
        self.load_content(&buffer).await
    }

    pub async fn load_demo(&mut self) -> Result<(), Error> {
        let content = include_bytes!("../data/ref/roland-nowaypoints.gpx");
        self.load_content(&content.to_vec()).await
    }
}

impl BackendData {
    pub fn get_parameters(self: &BackendData) -> Parameters {
        self.parameters.clone()
    }
    fn export_points(&self, points: &Vec<InputPoint>) -> Waypoints {
        let mut ret = Waypoints::new();
        for p in points {
            ret.push(p.waypoint());
        }
        WaypointInfo::make_waypoint_infos(&mut ret, &self.track, &self.parameters);
        ret
    }
    pub fn get_waypoints(&self, segment: &Segment, kinds: Kinds) -> Vec<Waypoint> {
        let mut points = Vec::new();
        let segpoints = &segment.points;
        let range = segment.range();
        for kind in &kinds {
            match segpoints.get(kind) {
                Some(kpoints) => {
                    let mut copy = kpoints.clone();
                    copy.retain(|w| {
                        w.kind() != InputType::OSM
                            && make_points::is_close_to_track(w)
                            && range.contains(&w.round_track_index().unwrap())
                    });
                    points.extend_from_slice(&copy);
                }
                None => {}
            }
        }
        log::trace!("export {} waypoints", points.len());
        self.export_points(&points)
    }

    pub fn set_parameters(self: &mut BackendData, parameters: &Parameters) {
        self.parameters = parameters.clone();
        if self.parameters.segment_overlap > self.parameters.segment_length {
            assert!(false);
        }

        // update user steps
        self.inputpoints
            .maps
            .insert(InputType::UserStep, InputPointMap::new());
        // update user points
        match self.inputpoints.maps.get_mut(&InputType::UserStep) {
            Some(user_steps_map) => {
                user_steps_map.clear();
                user_steps_map.sort_and_insert(&make_points::user_points(
                    &self.track,
                    &self.parameters.user_steps_options,
                ));
            }
            _ => {
                assert!(false);
            }
        }
    }

    pub fn setStartTime(&mut self, rfc3339: String) {
        self.parameters.start_time = rfc3339;
    }
    pub fn setSpeed(&mut self, s: f64) {
        self.parameters.speed = s;
    }
    pub fn setSegmentLength(&mut self, length: f64) {
        self.parameters.segment_length = length;
    }

    pub fn segments(&self) -> Vec<Segment> {
        let mut ret = Vec::new();

        let mut start = 0f64;
        let mut k = 0usize;
        loop {
            let end = start + self.parameters.segment_length;
            let range = self.track.segment(start, end);
            if range.is_empty() {
                break;
            }
            let tracktree = locate::IndexedPointsTree::from_track(&self.track, &range);
            log::trace!("make segment: {:.1} {:.1}", start / 1000f64, end / 1000f64);
            ret.push(Segment::new(
                k as i32,
                start,
                end,
                tracktree,
                self.track.clone(),
                &self.inputpoints,
                &self.parameters,
            ));
            start = start + self.parameters.segment_length - self.parameters.segment_overlap;
            k = k + 1;
        }
        ret
    }

    pub fn trackSegment(&self) -> Segment {
        let start = 0f64;
        let end = self.track.total_distance();
        let range = self.track.segment(start, end);
        let tracktree = locate::IndexedPointsTree::from_track(&self.track, &range);
        let ret = Segment::new(
            0,
            start,
            end,
            tracktree,
            self.track.clone(),
            &self.inputpoints,
            &self.parameters,
        );
        ret
    }

    pub fn render_segment_what(
        &mut self,
        segment: &Segment,
        what: &String,
        size: &IntegerSize2D,
        kinds: Kinds,
    ) -> String {
        log::trace!(
            "start - render_segment_what:{} {} size:{}x{}",
            segment.id,
            what,
            size.width,
            size.height
        );
        let ret = match what.as_str() {
            "profile" => segment.render_profile().svg,
            "ylabels" => self.render_yaxis_labels_overlay(segment),
            "wheel" => {
                let model = wheel::model::WheelModel::make(&segment, kinds);
                wheel::render(size, &model)
            }
            "map" => segment.render_map(size),
            _ => {
                // assert!(false);
                String::new()
            }
        };
        log::trace!("done - render_segment_what:{} {}", segment.id, what);
        ret
    }

    fn render_yaxis_labels_overlay(&mut self, segment: &Segment) -> String {
        log::info!("render_segment_track:{}", segment.id);
        let profile_bbox =
            gpsdata::ProfileBoundingBox::from_track(&segment.track, &segment.start, &segment.end);
        let mut profile =
            profile::ProfileView::init(&profile_bbox, &segment.parameters.profile_options);
        profile.add_yaxis_labels_overlay();
        let ret = profile.render().svg;
        if self.get_parameters().debug {
            let filename = std::format!("/tmp/yaxis-{}.svg", segment.id);
            std::fs::write(filename, &ret).expect("Unable to write file");
        }
        ret
    }

    pub fn segment_statistics(&self, segment: &Segment) -> SegmentStatistics {
        let range = &segment.range();
        assert!(range.end > 0);
        SegmentStatistics {
            length: self.track.distance(range.end - 1) - self.track.distance(range.start),
            elevation_gain: self.track.elevation_gain_on_range(&range),
            distance_start: self.track.distance(range.start),
            distance_end: self.track.distance(range.end - 1),
        }
    }

    pub fn statistics(&self) -> SegmentStatistics {
        let range = 0..self.track.len();
        assert!(range.end > 0);
        SegmentStatistics {
            length: self.track.distance(range.end - 1) - self.track.distance(range.start),
            elevation_gain: self.track.elevation_gain_on_range(&range),
            distance_start: self.track.distance(range.start),
            distance_end: self.track.distance(range.end - 1),
        }
    }
    pub async fn generatePdf(&mut self) -> Vec<u8> {
        let typbytes = render::make_typst_document(self);
        let ret = pdf::compile(&typbytes, self.get_parameters().debug).await;
        log::info!("generated {} bytes", ret.len());
        ret
    }
    pub fn generateGpx(&mut self) -> Vec<u8> {
        let range = 0..self.track.wgs84.len();
        let tracktree = locate::IndexedPointsTree::from_track(&self.track, &range);
        let mut gpxpoints = Vec::new();
        // TODO: we should project the GPX points segment-wise.
        for kind in [InputType::UserStep, InputType::GPX] {
            match self.inputpoints.maps.get(&kind) {
                Some(p) => {
                    let mut v = p.as_vector();
                    v.iter_mut().for_each(|mut p| {
                        if p.track_projection.is_none() {
                            Segment::compute_track_projection(&self.track, &tracktree, &mut p);
                        }
                    });
                    gpxpoints.extend_from_slice(&v);
                }
                _ => {}
            }
        }
        let waypoints = self.export_points(&gpxpoints);
        gpxexport::generate(&self.track, &waypoints)
    }
}

#[cfg(test)]
mod tests {
    use crate::{backend::Backend, inputpoint, math::IntegerSize2D, wheel};
    static START_TIME: &'static str = "1985-04-12T08:05:00.00Z";

    #[tokio::test]
    async fn svg_profile() {
        let _ = env_logger::try_init();
        log::info!("info");
        log::warn!("warn");
        log::trace!("trace");
        let mut backend = Backend::make();
        backend
            .load_filename("data/blackforest.gpx")
            .await
            .expect("fail");

        let mut parameters = backend.get_parameters();
        parameters.start_time = START_TIME.to_string();
        parameters.user_steps_options.step_distance = Some((10_000) as f64);
        parameters.profile_options.size = (1420, 400);
        parameters.profile_options.max_area_ratio = 0.1f64;
        backend.set_parameters(&parameters);

        let segments = backend.segments();
        let mut ok_count = 0;
        for k in 0..segments.len() {
            let segment = &segments[k];
            let rendered_profile = segment.render_profile();
            let reffilename = std::format!("data/ref/profile-{}.svg", segment.id);
            println!("test {}", reffilename);
            let reference_svg = if std::fs::exists(&reffilename).unwrap() {
                std::fs::read_to_string(&reffilename).unwrap()
            } else {
                String::new()
            };
            if reference_svg == rendered_profile.svg {
                ok_count += 1;
            }
            let tmpfilename = std::format!("/tmp/profile-{}.svg", segment.id);
            std::fs::write(&tmpfilename, rendered_profile.svg.clone()).unwrap();
            if reference_svg != rendered_profile.svg {
                println!("test failed: {} {}", tmpfilename, reffilename);
            }
        }
        assert!(ok_count == segments.len());
    }

    #[tokio::test]
    async fn svg_segment_wheel() {
        let _ = env_logger::try_init();
        let mut backend = Backend::make();
        backend
            .load_filename("data/blackforest.gpx")
            .await
            .expect("fail");
        let mut parameters = backend.get_parameters();
        parameters.start_time = START_TIME.to_string();
        parameters.user_steps_options.step_distance = Some((3_000) as f64);
        backend.set_parameters(&parameters);
        let reffilename = std::format!("data/ref/segment-wheel.svg");
        let data = if std::fs::exists(&reffilename).unwrap() {
            std::fs::read_to_string(&reffilename).unwrap()
        } else {
            String::new()
        };
        let model = wheel::model::WheelModel::make(&backend.trackSegment(), inputpoint::allkinds());
        let svg = wheel::render(&IntegerSize2D::new(400, 400), &model);

        let tmpfilename = std::format!("/tmp/segment-wheel.svg");
        std::fs::write(&tmpfilename, svg.clone()).unwrap();
        if data != svg {
            println!("test failed: {} {}", tmpfilename, reffilename);
            assert!(false);
        }
    }

    #[tokio::test]
    async fn svg_map() {
        let mut backend = Backend::make();
        backend
            .load_filename("data/blackforest.gpx")
            .await
            .expect("fail");
        let mut parameters = backend.get_parameters();
        parameters.start_time = START_TIME.to_string();
        parameters.user_steps_options.step_distance = Some((10_000) as f64);
        parameters.map_options.max_area_ratio = 0.15f64;
        backend.set_parameters(&parameters);

        let segments = backend.segments();
        let mut ok_count = 0;
        for segment in &segments {
            let _ = segment.render_profile();
            let svg = segment.render_map(&parameters.map_options.size2d());
            let reffilename = std::format!("data/ref/map-{}.svg", segment.id);
            println!("test {}", reffilename);
            let data = if std::fs::exists(&reffilename).unwrap() {
                std::fs::read_to_string(&reffilename).unwrap()
            } else {
                String::new()
            };
            if data == svg {
                ok_count += 1;
            }
            let tmpfilename = std::format!("/tmp/map-{}.svg", segment.id);
            std::fs::write(&tmpfilename, svg.clone()).unwrap();
            if data != svg {
                println!("test failed: {} {}", tmpfilename, reffilename);
            }
        }
        assert!(ok_count == segments.len());
    }

    #[tokio::test]
    async fn time_iso8601() {
        let mut backend = Backend::make();
        backend
            .load_filename("data/blackforest.gpx")
            .await
            .expect("fail");
        backend
            .dmut()
            .setStartTime(String::from("2007-03-01T13:00:00Z"));
        backend
            .dmut()
            .setStartTime(String::from("2025-07-12T06:32:36Z"));
        backend
            .dmut()
            .setStartTime(String::from("2025-07-12T06:32:36.215033Z"));
    }

    #[tokio::test]
    async fn track_bbox() {
        let mut backend = Backend::make();
        backend
            .load_filename("data/blackforest.gpx")
            .await
            .expect("fail");
        let bbox = backend.d().track.wgs84_bounding_box();
        println!("bbox={:?}", bbox);
        for x in [
            bbox.get_xmin(),
            bbox.get_ymin(),
            bbox.get_xmax(),
            bbox.get_ymax(),
        ] {
            assert!(x > 0f64);
        }
    }
}
