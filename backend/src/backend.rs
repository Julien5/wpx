#![allow(non_snake_case)]

use crate::elevation;
use crate::error::Error;
use crate::gpsdata;
use crate::gpsdata::distance_wgs84;
use crate::gpsdata::ProfileBoundingBox;
use crate::gpxexport;
use crate::inputpoint::InputPoints;
use crate::inputpoint::InputType;
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
    pub track_tree: locate::Locate,
    pub parameters: Parameters,
    pub track: track::Track,
    pub inputpoints: InputPoints,
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
    ) -> String {
        self.dmut().render_segment_what(segment, what, (W, H))
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

        self.send(&"download osm data".to_string()).await;
        let mut inputpoints = osm::download_for_track(&track).await;
        inputpoints
            .points
            .extend_from_slice(&gpsdata::read_waypoints(&gpx));
        // project::project_on_track::<InputPoint>(&track, &mut inputpoints.points);
        let parameters = Parameters::default();
        self.send(&"compute elevation".to_string()).await;
        let pointstree = locate::Locate::from_points(&inputpoints);
        let tracktree = locate::Locate::from_track(&track);
        let data = BackendData {
            inputpoints_tree: pointstree,
            track_tree: tracktree,
            track_smooth_elevation: elevation::smooth_elevation(
                &track,
                default_params.smooth_width,
            ),
            track,
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
    fn export_points(&self, subset: &Vec<usize>) -> Waypoints {
        log::trace!("export points");
        let mut ret = Waypoints::new();
        for k in subset {
            ret.push(self.inputpoints.points[*k].waypoint());
        }
        ret.retain(|w| w.track_index.is_some());
        ret.sort_by(|w1, w2| w1.track_index.cmp(&w2.track_index));
        WaypointInfo::make_waypoint_infos(
            &mut ret,
            &self.track,
            &self.track_smooth_elevation,
            &self.parameters.start_time,
            &self.parameters.speed,
        );
        ret
    }
    pub fn get_waypoints(&self) -> Vec<Waypoint> {
        log::trace!("get waypoints");
        let segment = self.complete_track_segment();
        let subset = self.select_points_for_profile(&segment);
        self.export_points(&subset)
    }

    pub fn set_parameters(self: &mut BackendData, parameters: &Parameters) {
        self.parameters = parameters.clone();
        if self.parameters.segment_overlap > self.parameters.segment_length {
            assert!(false);
        }
        self.track_smooth_elevation =
            elevation::smooth_elevation(&self.track, self.parameters.smooth_width);
    }

    pub fn get_waypoint_table(&self, segment: &Segment) -> Vec<Waypoint> {
        let subset = self.select_points_for_profile(segment);
        self.export_points(&subset)
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
            log::debug!("segment: {:.1} {:.1}", start / 1000f64, end / 1000f64);
            ret.push(Segment::new(k, range, &profile_bbox, &map_bbox));
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
            "profile" => self.render_segment(segment, (W, H)),
            "ylabels" => self.render_yaxis_labels_overlay(segment, (W, H)),
            "map" => self.render_segment_map(segment, (W, H)),
            _ => {
                // assert!(false);
                String::new()
            }
        }
    }

    fn select_points_for_profile(&self, segment: &Segment) -> Vec<usize> {
        let mut ret = Vec::new();
        let mut bbox = segment.map_bbox.clone();
        bbox.enlarge(&5000f64);
        let subset = self.inputpoints_tree.find_points_in_bbox(&bbox);
        for k in &subset {
            if self.inputpoints.points[*k].track_index.get().is_some() {
                continue;
            }
            let index = self
                .track_tree
                .nearest_neighbor(&self.inputpoints.points[*k].euclidian);
            self.inputpoints.points[*k].track_index.set(index);
        }
        for k in subset {
            let p = &self.inputpoints.points[k];
            // use only if there are no other points shown
            if p.kind() == InputType::Hamlet {
                continue;
            }
            if p.kind() == InputType::GPX {
                ret.push(k);
                continue;
            }
            let mut distance = f64::MAX;
            match p.track_index.get() {
                Some(index) => {
                    let ptrack = &self.track.wgs84[index];
                    distance = distance_wgs84(ptrack, &p.wgs84);
                }
                None => {}
            }
            if distance < 500f64 {
                ret.push(k);
                continue;
            }
            if p.population().is_some() {
                let pop = p.population().unwrap();
                if pop > 100000 && distance < 5000f64 {
                    ret.push(k);
                    continue;
                }
                if pop > 10000 && distance < 2000f64 {
                    ret.push(k);
                    continue;
                }
                if pop > 1000 && distance < 1000f64 {
                    ret.push(k);
                    continue;
                }
            }
            if distance < 500f64 {
                ret.push(k);
                continue;
            }
            // log::trace!("too far:{:?} d={:.1}", p.name(), distance);
        }
        ret
    }

    fn complete_track_segment(&self) -> Segment {
        let len = self.track.wgs84.len();
        assert!(len > 0);
        let range = self.track.segment(0f64, self.track.distance(len - 1));
        let profile_bbox = ProfileBoundingBox::from_track(&self.track, &range);
        let map_bbox = svgmap::bounding_box(&self.track, &range);
        Segment::new(usize::MAX, range, &profile_bbox, &map_bbox)
    }

    pub fn select_points_for_map(&self, segment: &Segment) -> Vec<usize> {
        let mut ret = Vec::new();
        let profile_indices = self.select_points_for_profile(segment);
        let points = &self.inputpoints.points;
        for k in 0..points.len() {
            if profile_indices.contains(&k) {
                ret.push(k);
                continue;
            }
            let p = &points[k];
            if p.kind() == InputType::City {
                ret.push(k);
                continue;
            }
        }
        ret
    }

    pub fn render_segment(&mut self, segment: &Segment, (W, H): (i32, i32)) -> String {
        log::info!("render_segment:{}", segment.id);
        let debug = self.get_parameters().debug;
        let subset = self.select_points_for_profile(segment);
        let ret = profile::profile(
            &self.track,
            &self.inputpoints,
            &subset,
            &segment,
            W,
            H,
            debug,
        );
        if self.get_parameters().debug {
            let filename = std::format!("/tmp/profile-{}.svg", segment.id);
            std::fs::write(filename, &ret).expect("Unable to write file");
        }
        ret
    }
    fn render_yaxis_labels_overlay(&mut self, segment: &Segment, (W, H): (i32, i32)) -> String {
        log::info!("render_segment_track:{}", segment.id);
        let mut profile = profile::ProfileView::init(&segment.profile_bbox, W, H);
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
        let subset = self.select_points_for_map(segment);
        let ret = svgmap::map(
            &self.track,
            &self.inputpoints,
            &subset,
            &segment,
            W,
            H,
            debug,
        );
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
        log::info!("export {} waypoints", self.inputpoints.points.len());
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
            let svg = backend.dmut().render_segment(&segment, (1420, 400));
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
