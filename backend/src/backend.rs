#![allow(non_snake_case)]

use crate::automatic;
use crate::elevation;
use crate::error::Error;
pub use crate::gpsdata;
use crate::gpsdata::ProfileBoundingBox;
use crate::gpxexport;
use crate::parameters::Parameters;
use crate::pdf;
use crate::render;
use crate::render_device::RenderDevice;
use crate::waypoint;
use crate::waypoint_values;
use crate::waypoints_table;

type DateTime = crate::utm::DateTime;
pub type Segment = crate::segment::Segment;
pub type SegmentStatistics = crate::segment::SegmentStatistics;

pub struct Backend {
    parameters: Parameters,
    pub track: gpsdata::Track,
    pub waypoints: Vec<waypoint::Waypoint>,
    track_smooth_elevation: Vec<f64>,
}

fn waypoint_time(start_time: DateTime, distance: f64, speed: f64) -> DateTime {
    let dt = (distance / speed).ceil() as i64;
    let delta = chrono::TimeDelta::new(dt, 0).unwrap();
    start_time + delta
}

impl Backend {
    pub fn get_parameters(self: &Backend) -> Parameters {
        self.parameters.clone()
    }

    fn update_waypoints(&mut self) {
        self.waypoints = automatic::generate(&self.track, &self.waypoints, &self.parameters);
        self.make_waypoint_infos();
        for w in &self.waypoints {
            debug_assert!(w.get_track_index() < self.track.len());
        }
        waypoint_values::compute_values(&mut self.waypoints, &self.track);
        println!("generated {} waypoints", self.waypoints.len());
    }
    pub fn get_waypoints(&self) -> Vec<waypoint::Waypoint> {
        return self.waypoints.clone();
    }

    pub fn set_parameters(self: &mut Backend, parameters: &Parameters) {
        self.parameters = parameters.clone();
        self.track_smooth_elevation =
            elevation::smooth_elevation(&self.track, self.parameters.smooth_width);
        self.update_waypoints();
    }

    fn create_waypoint_info(
        self: &Backend,
        w: &waypoint::Waypoint,
        wprev: Option<&waypoint::Waypoint>,
    ) -> waypoint::WaypointInfo {
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
        waypoint::WaypointInfo {
            wgs84: w.wgs84,
            utm: w.utm.clone(),
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
    pub fn get_waypoint_table(&self, segment: &Segment) -> Vec<waypoint::Waypoint> {
        let mut ret = Vec::new();
        let waypoints = &self.waypoints;
        let V = waypoints_table::show_waypoints_in_table(&self.waypoints, &segment.profile.bbox);
        let mut wprev: Option<&waypoint::Waypoint> = None;
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

    pub fn from_content(content: &Vec<u8>) -> Result<Backend, Error> {
        let mut gpx = gpsdata::read_gpx_content(content)?;
        let segment = match gpsdata::read_segment(&mut gpx) {
            Ok(s) => s,
            Err(e) => {
                return Err(e);
            }
        };
        let track = match gpsdata::Track::from_segment(&segment) {
            Ok(t) => t,
            Err(e) => {
                return Err(e);
            }
        };
        let default_params = Parameters::default();
        let mut gpxwaypoints = gpsdata::read_waypoints(&gpx);
        gpsdata::project_waypoints(&track, &mut gpxwaypoints);
        let parameters = Parameters::default();
        let mut ret = Backend {
            track_smooth_elevation: elevation::smooth_elevation(
                &track,
                default_params.smooth_width,
            ),
            track,
            waypoints: gpxwaypoints,
            parameters,
        };
        ret.update_waypoints();
        Ok(ret)
    }

    pub fn from_filename(filename: &str) -> Result<Backend, Error> {
        let mut f = std::fs::File::open(filename).unwrap();
        let mut buffer = Vec::new();
        // read the whole file
        use std::io::prelude::*;
        f.read_to_end(&mut buffer).unwrap();
        Self::from_content(&buffer)
    }

    pub fn demo() -> Result<Backend, Error> {
        let content = include_bytes!("../data/blackforest.gpx");
        Self::from_content(&content.to_vec())
    }

    pub fn epsilon(&self) -> f64 {
        self.parameters.epsilon
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
            start = start + self.parameters.segment_length;
            k = k + 1;
        }
        ret
    }
    pub fn render_segment(
        &mut self,
        segment: &Segment,
        (W, H): (i32, i32),
        render_device: RenderDevice,
    ) -> String {
        println!("render_segment_track:{}", segment.id);
        let mut profile = segment.profile.clone();
        profile.set_render_device(render_device);
        profile.reset_size(W, H);
        profile.add_canvas();
        profile.add_track(&self.track, &self.track_smooth_elevation);
        let W = self.get_waypoints();
        profile.add_waypoints(&W);
        profile.render()
    }
    pub fn render_yaxis_labels_overlay(
        &mut self,
        segment: &Segment,
        (W, H): (i32, i32),
        render_device: RenderDevice,
    ) -> String {
        println!("render_segment_track:{}", segment.id);
        let mut profile = segment.profile.clone();
        profile.set_render_device(render_device);
        profile.reset_size(W, H);
        profile.add_yaxis_labels_overlay();
        profile.render()
    }
    pub fn render_segment_track(
        &mut self,
        segment: &Segment,
        (W, H): (i32, i32),
        render_device: RenderDevice,
    ) -> String {
        println!("render_segment_track:{}", segment.id);
        let mut profile = segment.profile.clone();
        profile.set_render_device(render_device);
        profile.reset_size(W, H);
        profile.add_canvas();
        profile.add_track(&self.track, &self.track_smooth_elevation);
        let ret = profile.render();
        let filename = std::format!("/tmp/segment-{}.svg", segment.id);
        //std::fs::write(filename, &ret).expect("Unable to write file");
        ret
    }
    pub fn render_segment_waypoints(
        &mut self,
        segment: &Segment,
        (W, H): (i32, i32),
        render_device: RenderDevice,
    ) -> String {
        println!("render_segment_track:{}", segment.id);
        let mut profile = segment.profile.clone();
        profile.set_render_device(render_device);
        profile.reset_size(W, H);
        profile.add_waypoints(&self.get_waypoints());
        let ret = profile.render();
        //let filename = std::format!("/tmp/waypoints-{}.svg", segment.id);
        //std::fs::write(filename, &ret).expect("Unable to write file");
        ret
    }
    pub fn segment_statistics(&self, segment: &Segment) -> SegmentStatistics {
        let range = &segment.range;
        SegmentStatistics {
            length: self.track.distance(range.end - 1) - self.track.distance(range.start),
            elevation_gain: self.track.elevation_gain(&range),
            distance_start: self.track.distance(range.start),
            distance_end: self.track.distance(range.end - 1),
        }
    }
    pub fn statistics(&self) -> SegmentStatistics {
        let range = 0..self.track.wgs84.len();
        SegmentStatistics {
            length: self.track.distance(range.end - 1) - self.track.distance(range.start),
            elevation_gain: self.track.elevation_gain(&range),
            distance_start: self.track.distance(range.start),
            distance_end: self.track.distance(range.end - 1),
        }
    }
    pub fn generatePdf(&mut self, debug: bool) -> Vec<u8> {
        let typbytes = render::compile_pdf(self, debug, (1000, 285));
        //let typbytes = render::compile_pdf(self, debug, (1400, 400));
        let ret = pdf::compile(&typbytes, debug);
        println!("generated {} bytes", ret.len());
        ret
    }
    pub fn generateGpx(&mut self) -> Vec<u8> {
        println!("export {} waypoints", self.waypoints.len());
        gpxexport::generate(&self.track, &self.waypoints)
    }
}

#[cfg(test)]
mod tests {
    use crate::{backend::Backend, render_device::RenderDevice};
    #[test]
    fn svg_segment_track() {
        let mut backend = Backend::from_filename("data/blackforest.gpx").expect("fail");
        let segments = backend.segments();
        let mut ok_count = 0;
        for segment in &segments {
            let svg = backend.render_segment_track(&segment, (1420, 400), RenderDevice::Native);
            let reffilename = std::format!("data/ref/track-{}.svg", segment.id);
            println!("test {}", reffilename);
            if !std::fs::exists(&reffilename).unwrap() {
                continue;
            }
            let data = std::fs::read_to_string(&reffilename).unwrap();
            if data == svg {
                ok_count += 1;
            } else {
                let tmpfilename = std::format!("/tmp/track-{}.svg", segment.id);
                std::fs::write(&tmpfilename, svg).unwrap();
                println!("test failed: {} {}", tmpfilename, reffilename);
            }
        }
        assert!(ok_count == segments.len());
    }

    #[test]
    fn svg_segment_waypoints() {
        let mut backend = Backend::from_filename("data/blackforest.gpx").expect("fail");
        let segments = backend.segments();
        let mut ok_count = 0;
        let mut parameters = backend.get_parameters();
        use chrono::TimeZone;
        parameters.start_time = chrono::offset::Utc
            .with_ymd_and_hms(2025, 11, 10, 8, 0, 0)
            .unwrap()
            .to_rfc3339();
        backend.set_parameters(&parameters);
        for segment in &segments {
            let svg = backend.render_segment_waypoints(&segment, (1420, 400), RenderDevice::Native);
            let reffilename = std::format!("data/ref/waypoints-{}.svg", segment.id);
            println!("test {}", reffilename);
            if !std::fs::exists(&reffilename).unwrap() {
                continue;
            }
            let data = std::fs::read_to_string(&reffilename).unwrap();
            if data == svg {
                ok_count += 1;
            } else {
                let tmpfilename = std::format!("/tmp/waypoints-{}.svg", segment.id);
                std::fs::write(&tmpfilename, svg).unwrap();
                println!("test failed: {} {}", tmpfilename, reffilename);
            }
        }
        assert!(ok_count == segments.len());
    }

    #[test]
    fn time_iso8601() {
        let mut backend = Backend::from_filename("data/blackforest.gpx").expect("fail");
        backend.setStartTime(String::from("2007-03-01T13:00:00Z"));
        backend.setStartTime(String::from("2025-07-12T06:32:36Z"));
        backend.setStartTime(String::from("2025-07-12T06:32:36.215033Z"));
    }
}
