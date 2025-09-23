#![allow(non_snake_case)]

use crate::automatic;
use crate::elevation;
use crate::error::Error;
use crate::gpsdata;
use crate::gpsdata::ProfileBoundingBox;
use crate::gpxexport;
use crate::osm;
use crate::osm::OSMWaypoints;
use crate::parameters::Parameters;
use crate::pdf;
use crate::project;
use crate::render;
use crate::render_device::RenderDevice;
use crate::svgmap;
use crate::svgprofile;
use crate::track;
use crate::waypoint::Waypoint;
use crate::waypoint::WaypointInfo;
use crate::waypoint::Waypoints;
use crate::waypoint_values;
use crate::waypoints_table;

type DateTime = crate::utm::DateTime;
pub type Segment = crate::segment::Segment;
pub type SegmentStatistics = crate::segment::SegmentStatistics;

pub struct BackendData {
    pub parameters: Parameters,
    pub track: track::Track,
    pub waypoints: Waypoints,
    pub osmwaypoints: OSMWaypoints,
    pub track_smooth_elevation: Vec<f64>,
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
    fn dmut(&mut self) -> &mut BackendData {
        self.backend_data.as_mut().unwrap()
    }
    pub fn set_sink(&mut self, sink: SenderHandler) {
        self.sender = std::sync::RwLock::new(Some(sink));
    }
    pub async fn send(&self, data: &String) {
        log::trace!("data:{}", data);
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
    pub fn statistics(&self) -> SegmentStatistics {
        self.d().statistics()
    }
    pub fn generatePdf(&mut self) -> Vec<u8> {
        self.dmut().generatePdf()
    }
    pub fn generateGpx(&mut self) -> Vec<u8> {
        self.dmut().generateGpx()
    }
    pub fn segments(&self) -> Vec<Segment> {
        self.d().segments()
    }
    pub fn render_segment_what(
        &mut self,
        segment: &Segment,
        what: String,
        (W, H): (i32, i32),
        render_device: RenderDevice,
    ) -> String {
        self.dmut()
            .render_segment_what(segment, what, (W, H), render_device)
    }
    pub fn get_waypoints(&self) -> Vec<Waypoint> {
        return self.d().get_waypoints();
    }
    pub fn get_waypoint_table(&self, segment: &Segment) -> Vec<Waypoint> {
        return self.d().get_waypoint_table(segment);
    }

    pub async fn load_content(&mut self, content: &Vec<u8>) -> Result<(), Error> {
        self.send(&"read gpx".to_string()).await;
        let mut gpx = gpsdata::read_gpx_content(content)?;
        self.send(&"read segment".to_string()).await;
        let segment = match gpsdata::read_segment(&mut gpx) {
            Ok(s) => s,
            Err(e) => {
                return Err(e);
            }
        };
        self.send(&"read track".to_string()).await;
        let track = match track::Track::from_segment(&segment) {
            Ok(t) => t,
            Err(e) => {
                return Err(e);
            }
        };
        let default_params = Parameters::default();
        self.send(&"read waypoints".to_string()).await;
        let mut gpxwaypoints = gpsdata::read_waypoints(&gpx);
        project::project_on_track(&track, &mut gpxwaypoints);
        self.send(&"download osm data".to_string()).await;
        let osmwaypoints = osm::download_for_track(&track, 1000f64).await;
        let parameters = Parameters::default();
        self.send(&"compute elevation".to_string()).await;
        let mut data = BackendData {
            track_smooth_elevation: elevation::smooth_elevation(
                &track,
                default_params.smooth_width,
            ),
            track,
            waypoints: gpxwaypoints,
            osmwaypoints,
            parameters,
        };
        self.send(&"update waypoints".to_string()).await;
        data.update_waypoints();
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

fn waypoint_time(start_time: DateTime, distance: f64, speed: f64) -> DateTime {
    let dt = (distance / speed).ceil() as i64;
    let delta = chrono::TimeDelta::new(dt, 0).unwrap();
    start_time + delta
}

impl BackendData {
    pub fn get_parameters(self: &BackendData) -> Parameters {
        self.parameters.clone()
    }
    fn update_waypoints(&mut self) {
        self.waypoints = automatic::generate(
            &self.track,
            &self.waypoints,
            &self.osmwaypoints,
            &self.parameters,
        );
        self.make_waypoint_infos();
        for w in &self.waypoints {
            debug_assert!(w.get_track_index() < self.track.len());
        }
        waypoint_values::compute_values(&mut self.waypoints, &self.track);
        log::info!("generated {} waypoints", self.waypoints.len());
    }
    pub fn get_waypoints(&self) -> Vec<Waypoint> {
        return self.waypoints.clone();
    }

    pub fn set_parameters(self: &mut BackendData, parameters: &Parameters) {
        self.parameters = parameters.clone();
        if self.parameters.segment_overlap > self.parameters.segment_length {
            assert!(false);
        }
        self.track_smooth_elevation =
            elevation::smooth_elevation(&self.track, self.parameters.smooth_width);
        self.update_waypoints();
    }

    fn create_waypoint_info(
        self: &BackendData,
        w: &Waypoint,
        wprev: Option<&Waypoint>,
    ) -> WaypointInfo {
        let track = &self.track;
        assert!(w.get_track_index() < track.len());
        let distance = track.distance(w.get_track_index());
        let (inter_distance, inter_elevation_gain, inter_slope_prev) = match wprev {
            None => (0f64, 0f64, 0f64),
            Some(prev) => {
                let dx =
                    track.distance(w.get_track_index()) - track.distance(prev.get_track_index());
                let dy = self.elevation_gain(prev.get_track_index(), w.get_track_index());
                let slope = match dx {
                    0f64 => 0f64,
                    _ => dy / dx,
                };
                (dx, dy, slope)
            }
        };
        use chrono::*;
        let start_time: DateTime<Utc> = self.parameters.start_time.parse().unwrap();
        let time = waypoint_time(start_time, distance, self.parameters.speed);
        let name = match &w.name {
            None => String::new(),
            Some(n) => n.clone(),
        };
        let description = match &w.description {
            None => name.clone(),
            Some(desc) => match name.is_empty() {
                true => format!("{}", desc),
                false => format!("{} - {}", name, desc),
            },
        };
        WaypointInfo {
            wgs84: w.wgs84.clone(),
            origin: w.origin.clone(),
            distance,
            inter_distance,
            inter_elevation_gain,
            inter_slope: inter_slope_prev,
            elevation: track.elevation(w.get_track_index()),
            name,
            description,
            time: time.to_rfc3339(),
            track_index: w.get_track_index(),
            value: None,
        }
    }
    fn make_waypoint_infos(&mut self) {
        let mut infos = Vec::new();
        for w in &self.waypoints {
            debug_assert!(w.get_track_index() < self.track.len());
        }
        for k in 0..self.waypoints.len() {
            let w = &self.waypoints[k];
            let wprev = match k {
                0 => None,
                _ => Some(&self.waypoints[k - 1]),
            };
            let step = self.create_waypoint_info(w, wprev);
            infos.push(step.clone());
        }
        for k in 0..self.waypoints.len() {
            let w = &mut self.waypoints[k];
            w.info = Some(infos[k].clone());
        }
    }
    pub fn get_waypoint_table(&self, segment: &Segment) -> Vec<Waypoint> {
        let mut ret = Vec::new();
        let waypoints = &self.waypoints;
        let V = waypoints_table::show_waypoints_in_table(&self.waypoints, &segment.bbox);
        let mut wprev: Option<&Waypoint> = None;
        for k in 0..waypoints.len() {
            if !V.contains(&k) {
                continue;
            }
            assert!(waypoints[k].info.is_some());
            let mut copy = waypoints[k].clone();
            copy.info = Some(self.create_waypoint_info(&waypoints[k], wprev));
            wprev = Some(&self.waypoints[k]);
            ret.push(copy);
        }
        ret
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

    pub fn elevation_gain(&self, from: usize, to: usize) -> f64 {
        debug_assert!(from <= to);
        let mut ret = 0f64;
        for k in from..to {
            if k == 0 {
                continue;
            }
            let d = self.track_smooth_elevation[k] - self.track_smooth_elevation[k - 1];
            if d > 0f64 {
                ret += d;
            }
        }
        ret
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
            let bbox = ProfileBoundingBox::from_track(&self.track, &range);
            ret.push(Segment::new(k, range, &bbox));
            start = start + self.parameters.segment_length - self.parameters.segment_overlap;
            k = k + 1;
        }
        ret
    }
    pub fn render_segment_what(
        &mut self,
        segment: &Segment,
        what: String,
        (W, H): (i32, i32),
        render_device: RenderDevice,
    ) -> String {
        log::info!("render_segment_what:{} {}", segment.id, what);
        match what.as_str() {
            "profile" => self.render_segment(segment, (W, H), render_device),
            "ylabels" => self.render_yaxis_labels_overlay(segment, (W, H), render_device),
            "map" => self.render_segment_map(segment, (W, H)),
            _ => {
                // assert!(false);
                String::new()
            }
        }
    }
    pub fn render_segment(
        &mut self,
        segment: &Segment,
        (W, H): (i32, i32),
        render_device: RenderDevice,
    ) -> String {
        log::info!("render_segment:{}", segment.id);
        let debug = self.get_parameters().debug;
        let ret = svgprofile::profile(&self, &segment, W, H, render_device, debug);
        if self.get_parameters().debug {
            let filename = std::format!("/tmp/profile-{}.svg", segment.id);
            std::fs::write(filename, &ret).expect("Unable to write file");
        }
        ret
    }
    fn render_yaxis_labels_overlay(
        &mut self,
        segment: &Segment,
        (W, H): (i32, i32),
        render_device: RenderDevice,
    ) -> String {
        log::info!("render_segment_track:{}", segment.id);
        let mut profile = svgprofile::ProfileView::init(&segment.bbox);
        profile.set_render_device(render_device);
        profile.reset_size(W as f64, H as f64);
        profile.add_yaxis_labels_overlay();
        let ret = profile.render();
        if self.get_parameters().debug {
            let filename = std::format!("/tmp/yaxis-{}.svg", segment.id);
            std::fs::write(filename, &ret).expect("Unable to write file");
        }
        ret
    }
    pub fn render_segment_map(&self, segment: &Segment, (W, H): (i32, i32)) -> String {
        let debug = self.get_parameters().debug;
        let ret = svgmap::map(&self, &segment, W, H, debug);
        if self.get_parameters().debug {
            let filename = std::format!("/tmp/map-{}.svg", segment.id);
            std::fs::write(filename, &ret).expect("Unable to write file");
        }
        ret
    }
    pub fn segment_statistics(&self, segment: &Segment) -> SegmentStatistics {
        let range = &segment.range;
        assert!(range.end > 0);
        SegmentStatistics {
            length: self.track.distance(range.end - 1) - self.track.distance(range.start),
            elevation_gain: self.track.elevation_gain(&range),
            distance_start: self.track.distance(range.start),
            distance_end: self.track.distance(range.end - 1),
        }
    }
    pub fn statistics(&self) -> SegmentStatistics {
        let range = 0..self.track.wgs84.len();
        assert!(range.end > 0);
        SegmentStatistics {
            length: self.track.distance(range.end - 1) - self.track.distance(range.start),
            elevation_gain: self.track.elevation_gain(&range),
            distance_start: self.track.distance(range.start),
            distance_end: self.track.distance(range.end - 1),
        }
    }
    pub fn generatePdf(&mut self) -> Vec<u8> {
        let typbytes = render::make_typst_document(self, (1000, 285));
        //let typbytes = render::compile_pdf(self, debug, (1400, 400));
        let ret = pdf::compile(&typbytes, self.get_parameters().debug);
        log::info!("generated {} bytes", ret.len());
        ret
    }
    pub fn generateGpx(&mut self) -> Vec<u8> {
        log::info!("export {} waypoints", self.waypoints.len());
        gpxexport::generate(&self.track, &self.waypoints)
    }
}

#[cfg(test)]
mod tests {
    use crate::{backend::Backend, render_device::RenderDevice};

    #[tokio::test]
    async fn svg_profile() {
        let mut backend = Backend::make();
        backend
            .load_filename("data/blackforest.gpx")
            .await
            .expect("fail");
        let segments = backend.segments();
        let mut ok_count = 0;
        for segment in &segments {
            let svg = backend
                .dmut()
                .render_segment(&segment, (1420, 400), RenderDevice::Native);
            let reffilename = std::format!("data/ref/profile-{}.svg", segment.id);
            log::info!("test {}", reffilename);
            let data = if std::fs::exists(&reffilename).unwrap() {
                std::fs::read_to_string(&reffilename).unwrap()
            } else {
                String::new()
            };
            if data == svg {
                ok_count += 1;
            } else {
                let tmpfilename = std::format!("/tmp/profile-{}.svg", segment.id);
                std::fs::write(&tmpfilename, svg).unwrap();
                println!("test failed: {} {}", tmpfilename, reffilename);
            }
        }
        assert!(ok_count == segments.len());
    }

    #[tokio::test]
    async fn svg_map() {
        let mut backend = Backend::make();
        backend
            .load_filename("data/blackforest.gpx")
            .await
            .expect("fail");
        let segments = backend.segments();
        let mut ok_count = 0;
        for segment in &segments {
            let svg = backend.d().render_segment_map(&segment, (400, 400));
            let reffilename = std::format!("data/ref/map-{}.svg", segment.id);
            log::info!("test {}", reffilename);
            let data = if std::fs::exists(&reffilename).unwrap() {
                std::fs::read_to_string(&reffilename).unwrap()
            } else {
                String::new()
            };
            if data == svg {
                ok_count += 1;
            } else {
                let tmpfilename = std::format!("/tmp/map-{}.svg", segment.id);
                std::fs::write(&tmpfilename, svg).unwrap();
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
        log::info!("bbox={:?}", bbox);
        for x in [bbox.min.0, bbox.min.1, bbox.max.0, bbox.max.1] {
            assert!(x > 0f64);
        }
    }
}
