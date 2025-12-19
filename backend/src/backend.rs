#![allow(non_snake_case)]

use std::collections::BTreeMap;

use crate::controls;
use crate::error::Error;
use crate::event;
use crate::gpsdata;
use crate::gpxexport;
use crate::inputpoint::*;
use crate::make_points;
use crate::math::IntegerSize2D;
use crate::osm;
use crate::parameters::Parameters;
use crate::pdf;
use crate::profile;
use crate::render;
use crate::track;
use crate::track::Track;
use crate::track_projection::is_close_to_track;
use crate::track_projection::ProjectionTrees;
use crate::waypoint::Waypoint;
use crate::waypoint::WaypointInfo;
use crate::waypoint::Waypoints;
use crate::wheel;

pub type Segment = crate::segment::Segment;
pub type SegmentStatistics = crate::segment::SegmentStatistics;
pub use crate::event::Sender;
pub type SenderHandler = crate::event::SenderHandler;
pub type SenderHandlerLock = crate::event::SenderHandlerLock;

pub struct BackendData {
    pub parameters: Parameters,
    pub track: std::sync::Arc<track::Track>,
    pub inputpoints: SharedPointMaps,
}

pub struct Backend {
    backend_data: Option<BackendData>,
    pub sender: SenderHandlerLock,
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
        event::send_worker(&self.sender, data).await
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
    pub fn set_userstep_gpx_name_format(&mut self, format: &String) {
        self.dmut().set_userstep_gpx_name_format(format);
    }
    pub fn set_control_gpx_name_format(&mut self, format: &String) {
        self.dmut().set_control_gpx_name_format(format);
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
        let mut gpxdata = gpsdata::read_content(content)?;
        let track_data = Track::from_tracks(&gpxdata.tracks)?;
        let track = std::sync::Arc::new(track_data);
        log::trace!("make projection trees");
        let trees = ProjectionTrees::make(&track);
        self.send(&"download osm data".to_string()).await;
        let mut inputpoints_map = BTreeMap::new();
        let mut osmpoints = osm::download_for_track(&track, &self.sender).await;
        log::trace!("project osm points");
        trees.iter_on(&mut osmpoints, &track);
        inputpoints_map.insert(InputType::OSM, osmpoints);
        trees.iter_on(&mut gpxdata.waypoints, &track);
        let gpx_waypoints = gpxdata.waypoints.as_vector();
        inputpoints_map.insert(InputType::GPX, gpxdata.waypoints);

        let mut controls = controls::infer_controls_from_gpx_data(&track, &gpx_waypoints);
        if controls.is_empty() {
            log::info!("infer_controls_from_gpx_data empty => try waypoints");
            controls = controls::make_controls_with_waypoints(&track, &gpx_waypoints);
        } else {
            log::trace!("infer_controls_from_gpx_data OK");
        }

        let inputpoints = SharedPointMaps::new(
            InputPointMaps {
                maps: inputpoints_map,
            }
            .into(),
        );

        if controls.is_empty() {
            controls = controls::make_controls_with_osm(&track, inputpoints.clone());
            log::info!(
                "control from gpx_data or waypoints empty => tried osm => {} points",
                controls.len()
            );
        }

        {
            let mut locked = inputpoints.write().unwrap();

            locked
                .maps
                .insert(InputType::Control, InputPointMap::from_vector(&controls));
        }

        let parameters = Parameters::default();
        self.send(&"compute elevation".to_string()).await;
        let data = BackendData {
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
    pub fn export_points(&self, points: &Vec<InputPoint>) -> Waypoints {
        let mut ret = Waypoints::new();
        for p in points {
            ret.push(p.waypoint());
        }
        WaypointInfo::make_waypoint_infos(&mut ret, &self.track, &self.parameters);
        ret
    }

    pub fn get_points(&self, segment: &Segment, kinds: Kinds) -> Vec<InputPoint> {
        let mut points = Vec::new();
        let range = &segment.range();
        for kind in &kinds {
            match self.inputpoints.read().unwrap().maps.get(kind) {
                Some(kpoints) => {
                    let mut copy = kpoints.as_vector();
                    copy.retain(|w| {
                        assert!(kinds.contains(&w.kind()));
                        assert!(is_close_to_track(&w));
                        range.contains(&w.track_projections.first().unwrap().track_index)
                    });
                    points.extend_from_slice(&copy);
                }
                None => {}
            }
        }
        log::trace!("segment: {} export {} waypoints", segment.id, points.len());
        points
    }

    pub fn get_waypoints(&self, segment: &Segment, kinds: Kinds) -> Vec<Waypoint> {
        self.export_points(&self.get_points(segment, kinds))
    }

    // used by bridge
    pub fn set_userstep_gpx_name_format(&mut self, format: &String) {
        self.parameters.user_steps_options.gpx_name_format = format.clone();
    }

    // used by bridge
    pub fn set_control_gpx_name_format(&mut self, format: &String) {
        self.parameters.control_gpx_name_format = format.clone();
    }

    pub fn set_parameters(self: &mut BackendData, parameters: &Parameters) {
        self.parameters = parameters.clone();
        if self.parameters.segment_overlap > self.parameters.segment_length {
            assert!(false);
        }

        // update user steps
        {
            let mut locked = self.inputpoints.write().unwrap();
            locked
                .maps
                .insert(InputType::UserStep, InputPointMap::new());

            // update user points
            match locked.maps.get_mut(&InputType::UserStep) {
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
            let range = self.track.subrange(start, end);
            if range.is_empty() {
                break;
            }
            log::trace!("make segment: {:.1} {:.1}", start / 1000f64, end / 1000f64);
            ret.push(Segment::new(
                k as i32,
                start,
                end,
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
        let ret = Segment::new(
            0,
            start,
            end,
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
        log::info!(
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
        log::info!("generated {} pdf bytes", ret.len());
        ret
    }
    pub fn generateGpx(&mut self) -> Vec<u8> {
        let mut gpxpoints = Vec::new();
        for kind in [InputType::UserStep] {
            match self.inputpoints.read().unwrap().maps.get(&kind) {
                Some(p) => {
                    let v = p.as_vector();
                    v.iter().for_each(|p| {
                        assert!(!p.track_projections.is_empty());
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
    use crate::{
        backend::Backend,
        inputpoint::{self, InputType},
        math::IntegerSize2D,
        wheel,
    };
    static START_TIME: &'static str = "1985-04-12T08:05:00.00Z";

    #[tokio::test]
    async fn svg_profile() {
        let _ = env_logger::try_init();
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
    async fn test_get_waypoints() {
        let _ = env_logger::try_init();
        let mut backend = Backend::make();
        let _ = backend.load_demo().await;
        let seg = backend.trackSegment();
        let controls = seg
            .pointmaps
            .read()
            .unwrap()
            .maps
            .get(&InputType::Control)
            .unwrap()
            .as_vector();
        let len = controls.len();
        assert!(len > 0);
        let kinds = std::collections::HashSet::from([InputType::Control]);
        let waypoints = backend.get_waypoints(&seg, kinds);
        assert!(!waypoints.is_empty());
        for waypoint in waypoints {
            log::info!("gpx name={}", waypoint.info.unwrap().gpx_name);
        }
    }

    #[tokio::test]
    async fn svg_map() {
        let _ = env_logger::try_init();
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
}
