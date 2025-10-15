#![allow(non_snake_case)]

use crate::error::Error;
use crate::gpsdata;
use crate::gpsdata::ProfileBoundingBox;
use crate::gpxexport;
use crate::inputpoint::InputPoint;
use crate::inputpoint::InputPointMap;
use crate::locate;
use crate::osm;
use crate::parameters::Parameters;
use crate::pdf;
use crate::profile;
use crate::render;
use crate::svgmap;
use crate::track;
use crate::waypoint::Waypoint;
use crate::waypoint::WaypointInfo;
use crate::waypoint::Waypoints;

pub type Segment = crate::segment::Segment;
pub type SegmentStatistics = crate::segment::SegmentStatistics;

pub struct BackendData {
    pub inputpoints_tree: locate::Locate,
    pub parameters: Parameters,
    pub track: std::sync::Arc<track::Track>,
    pub inputpoints: InputPointMap,
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
    ) -> String {
        self.dmut().render_segment_what(segment, what, (W, H))
    }
    pub fn get_waypoints(&self, segment: &Segment) -> Vec<Waypoint> {
        return self.d().get_waypoints(segment);
    }
    pub fn get_waypoint_table(&self, segment: &Segment) -> Vec<Waypoint> {
        return self.d().get_waypoint_table(segment);
    }

    pub async fn load_content(&mut self, content: &Vec<u8>) -> Result<(), Error> {
        self.send(&"read gpx".to_string()).await;
        let gpxdata = gpsdata::read_content(content)?;
        self.send(&"download osm data".to_string()).await;
        let mut inputpoints = osm::download_for_track(&gpxdata.track).await;
        inputpoints.extend(&gpxdata.waypoints);
        // project::project_on_track::<InputPoint>(&track, &mut inputpoints.points);
        let parameters = Parameters::default();
        self.send(&"compute elevation".to_string()).await;
        let pointstree = locate::Locate::from_points(&inputpoints.as_vector());
        let data = BackendData {
            inputpoints_tree: pointstree,
            track: std::sync::Arc::new(gpxdata.track),
            inputpoints,
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
        WaypointInfo::make_waypoint_infos(
            &mut ret,
            &self.track,
            &self.parameters.start_time,
            &self.parameters.speed,
        );
        ret
    }
    pub fn get_waypoints(&self, segment: &Segment) -> Vec<Waypoint> {
        let points = segment.profile_points();
        self.export_points(&points)
    }

    pub fn set_parameters(self: &mut BackendData, parameters: &Parameters) {
        self.parameters = parameters.clone();
        if self.parameters.segment_overlap > self.parameters.segment_length {
            assert!(false);
        }
    }

    pub fn get_waypoint_table(&self, segment: &Segment) -> Vec<Waypoint> {
        self.get_waypoints(segment)
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
            let profile_bbox = ProfileBoundingBox::from_track(&self.track, &range);
            let map_bbox = svgmap::bounding_box(&self.track, &range);
            let tracktree = locate::Locate::from_track(&self.track, &range);
            log::trace!("make segment: {:.1} {:.1}", start / 1000f64, end / 1000f64);
            ret.push(Segment::new(
                k,
                range,
                &profile_bbox,
                &map_bbox,
                tracktree,
                self.track.clone(),
                &self.inputpoints,
            ));
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
    ) -> String {
        log::info!("render_segment_what:{} {}", segment.id, what);
        match what.as_str() {
            "profile" => segment.render_profile((W, H), self.parameters.debug),
            "ylabels" => self.render_yaxis_labels_overlay(segment, (W, H)),
            "map" => segment.render_map((W, H), self.parameters.debug),
            _ => {
                // assert!(false);
                String::new()
            }
        }
    }

    fn render_yaxis_labels_overlay(&mut self, segment: &Segment, (W, H): (i32, i32)) -> String {
        log::info!("render_segment_track:{}", segment.id);
        let mut profile = profile::ProfileView::init(
            &segment.profile_bbox,
            profile::ProfileIndications::None,
            W,
            H,
        );
        profile.add_yaxis_labels_overlay();
        let ret = profile.render();
        if self.get_parameters().debug {
            let filename = std::format!("/tmp/yaxis-{}.svg", segment.id);
            std::fs::write(filename, &ret).expect("Unable to write file");
        }
        ret
    }
    pub fn render_segment_map(&self, segment: &Segment, (W, H): (i32, i32)) -> String {
        let ret = segment.render_map((W, H), self.parameters.debug);
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
            elevation_gain: self.track.elevation_gain_on_range(&range),
            distance_start: self.track.distance(range.start),
            distance_end: self.track.distance(range.end - 1),
        }
    }
    pub fn statistics(&self) -> SegmentStatistics {
        let range = 0..self.track.wgs84.len();
        assert!(range.end > 0);
        SegmentStatistics {
            length: self.track.distance(range.end - 1) - self.track.distance(range.start),
            elevation_gain: self.track.elevation_gain_on_range(&range),
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
        gpxexport::generate(&self.track, &self.inputpoints)
    }
}

#[cfg(test)]
mod tests {
    use crate::backend::Backend;

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
            let svg = segment.render_profile((1420, 400), backend.get_parameters().debug);
            let reffilename = std::format!("data/ref/profile-{}.svg", segment.id);
            println!("test {}", reffilename);
            let data = if std::fs::exists(&reffilename).unwrap() {
                std::fs::read_to_string(&reffilename).unwrap()
            } else {
                String::new()
            };
            if data == svg {
                ok_count += 1;
            }
            let tmpfilename = std::format!("/tmp/profile-{}.svg", segment.id);
            std::fs::write(&tmpfilename, svg.clone()).unwrap();
            if data != svg {
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
            let svg = segment.render_map((400, 400), true);
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
