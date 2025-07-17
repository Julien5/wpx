#![allow(non_snake_case)]

use crate::elevation;
pub use crate::gpsdata;
use crate::gpsdata::ProfileBoundingBox;
use crate::gpsdata::WaypointOrigin;
use crate::gpxexport;
use crate::pdf;
use crate::project;
use crate::render;
use crate::speed;
use crate::svgprofile;
use crate::utm::UTMPoint;
use std::str::FromStr;

type DateTime = crate::utm::DateTime;

pub struct Backend {
    pub track: gpsdata::Track,
    track_smooth_elevation: Vec<f64>,
    pub waypoints: Vec<gpsdata::Waypoint>,
    epsilon: f32,
    pub segment_length: f64,
    start_time: DateTime,
    speed: f64, // m/s
    smooth_width: f64,
}

#[derive(Clone)]
pub struct Segment {
    pub id: usize,
    pub range: std::ops::Range<usize>,
    pub profile: svgprofile::Profile,
}

impl Segment {
    pub fn shows_waypoint(&self, wp: &Step) -> bool {
        self.profile.shows_waypoint(wp)
    }
}

pub struct SegmentStatistics {
    pub length: f64,
    pub elevation_gain: f64,
    pub distance_start: f64,
    pub distance_end: f64,
}

#[derive(Clone)]
pub struct Step {
    pub wgs84: (f64, f64, f64),
    pub utm: UTMPoint,
    pub origin: WaypointOrigin,
    pub distance: f64,
    pub elevation: f64,
    pub inter_distance: f64,
    pub inter_elevation_gain: f64,
    pub inter_slope: f64,
    pub name: String,
    pub time: String,
    pub track_index: usize,
}

impl Segment {
    pub fn new(id: usize, range: std::ops::Range<usize>, bbox: &ProfileBoundingBox) -> Segment {
        Segment {
            id,
            range: range.clone(),
            profile: svgprofile::Profile::init(&range, &bbox),
        }
    }
}

fn waypoint_time(start_time: DateTime, distance: f64, speed: f64) -> DateTime {
    let dt = (distance / speed).ceil() as i64;
    let delta = chrono::TimeDelta::new(dt, 0).unwrap();
    start_time + delta
}

impl Backend {
    fn create_waypoint(
        self: &Backend,
        w: &gpsdata::Waypoint,
        wprev: Option<&gpsdata::Waypoint>,
    ) -> Step {
        let track = &self.track;
        assert!(w.track_index < track.len());
        let distance = track.distance(w.track_index);
        let (inter_distance, inter_elevation_gain, inter_slope) = match wprev {
            None => (0f64, 0f64, 0f64),
            Some(prev) => {
                let dx = track.distance(w.track_index) - track.distance(prev.track_index);
                let dy = self.elevation_gain(prev.track_index, w.track_index);
                let slope = match dx {
                    0f64 => 0f64,
                    _ => 100f64 * dy / dx,
                };
                (dx, dy, slope)
            }
        };
        let name = match &w.name {
            None => String::from_str("").unwrap(),
            Some(n) => n.clone(),
        };
        Step {
            wgs84: w.wgs84,
            utm: w.utm.clone(),
            origin: w.origin.clone(),
            distance: distance,
            inter_distance: inter_distance,
            inter_elevation_gain: inter_elevation_gain,
            inter_slope: inter_slope,
            elevation: track.elevation(w.track_index),
            name: name,
            time: waypoint_time(self.start_time, distance, self.speed).to_rfc3339(),
            track_index: w.track_index,
        }
    }
    pub fn get_steps(&self) -> Vec<Step> {
        let mut ret = Vec::new();
        for w in &self.waypoints {
            debug_assert!(w.track_index < self.track.len());
        }
        for k in 0..self.waypoints.len() {
            let w = &self.waypoints[k];
            let wprev = match k {
                0 => None,
                _ => Some(&self.waypoints[k - 1]),
            };
            let wp = self.create_waypoint(w, wprev);
            ret.push(wp.clone());
        }
        ret
    }
    pub fn setStartTime(&mut self, iso8601: String) {
        use chrono::*;
        println!("iso:{}", iso8601);
        let mut fixed = iso8601.clone();
        if !fixed.ends_with("Z") {
            fixed = String::from(format!("{}Z", iso8601));
        }
        let p: DateTime<Utc> = fixed.parse().unwrap();
        self.start_time = p;
    }
    pub fn setSpeed(&mut self, s: f64) {
        self.speed = s;
    }
    pub fn setSegmentLength(&mut self, length: f64) {
        self.segment_length = length;
    }
    fn enrichWaypoints(&mut self) {
        // not fast.
        let mut waypoints = Vec::new();
        // take GPX waypoints
        for w in &self.waypoints {
            match w.origin {
                WaypointOrigin::GPX => waypoints.push(w.clone()),
                _ => {}
            }
        }
        // add interesting ones (dougles peucker) with epsilon parameter
        let indexes = self.track.interesting_indexes(self.epsilon);
        for idx in indexes {
            let wgs = self.track.wgs84[idx].clone();
            let utm = self.track.utm[idx].clone();
            waypoints.push(gpsdata::Waypoint::from_track(wgs, utm, idx));
        }
        // find their indexes...
        let indexes = project::nearest_neighboor(&self.track.utm, &waypoints);
        debug_assert_eq!(waypoints.len(), indexes.len());
        for k in 0..indexes.len() {
            assert!(indexes[k] < self.track.len());
            waypoints[k].track_index = indexes[k];
        }
        for w in &waypoints {
            assert!(w.track_index < self.track.len());
        }
        // .. and sort them
        waypoints.sort_by(|w1, w2| w1.track_index.cmp(&w2.track_index));
        for k in 1..waypoints.len() {
            let k1 = waypoints[k].track_index;
            let k0 = waypoints[k - 1].track_index;
            debug_assert!(k1 >= k0);
        }
        for w in &waypoints {
            debug_assert!(w.track_index < self.track.len());
        }
        self.waypoints = waypoints;
    }

    pub fn elevation_gain(&self, from: usize, to: usize) -> f64 {
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

    pub fn set_smooth_width(&mut self, W: f64) {
        self.smooth_width = W;
        self.track_smooth_elevation = elevation::smooth(&self.track, self.smooth_width);
    }

    pub fn from_content(content: &Vec<u8>) -> Backend {
        let mut gpx = gpsdata::read_gpx_content(content);
        let segment = gpsdata::read_segment(&mut gpx);
        let track = gpsdata::Track::from_segment(&segment);
        let km = 1000f64;
        use chrono::TimeZone;
        let smooth_width_default = 200f64;
        let mut ret = Backend {
            track_smooth_elevation: elevation::smooth(&track, smooth_width_default),
            track: track,
            waypoints: gpsdata::read_waypoints(&gpx),
            epsilon: 150.0f32,
            segment_length: 100f64 * km,
            start_time: chrono::Utc.with_ymd_and_hms(2024, 4, 4, 8, 0, 0).unwrap(),
            speed: speed::mps(15f64),
            smooth_width: smooth_width_default,
        };
        ret.updateWaypoints();
        for w in &ret.waypoints {
            debug_assert!(w.track_index < ret.track.len());
        }
        ret
    }

    pub fn from_filename(filename: &str) -> Backend {
        println!("filename:{}", filename);
        let mut f = std::fs::File::open(filename).unwrap();
        let mut buffer = Vec::new();
        // read the whole file
        use std::io::prelude::*;
        f.read_to_end(&mut buffer).unwrap();
        Self::from_content(&buffer)
    }

    pub fn demo() -> Backend {
        let content = include_bytes!("../data/blackforest.gpx");
        Self::from_content(&content.to_vec())
    }

    fn updateWaypoints(&mut self) {
        self.enrichWaypoints();
    }

    pub fn adjustEpsilon(&mut self, delta: f32) {
        self.epsilon += delta;
        self.updateWaypoints();
    }

    pub fn epsilon(&self) -> f32 {
        self.epsilon
    }

    pub fn segments(&self) -> Vec<Segment> {
        let mut ret = Vec::new();

        let mut start = 0f64;
        let mut k = 0usize;
        loop {
            let end = start + self.segment_length;
            let range = self.track.segment(start, end);
            if range.is_empty() {
                break;
            }
            let bbox = ProfileBoundingBox::from_track(&self.track, &range);
            ret.push(Segment::new(k, range, &bbox));
            start = start + self.segment_length;
            k = k + 1;
        }
        ret
    }
    pub fn render_segment(&mut self, segment: &Segment, (W, H): (i32, i32)) -> String {
        println!("render_segment_track:{}", segment.id);
        let mut profile = segment.profile.clone();
        profile.reset_size(W, H);
        profile.add_canvas();
        profile.add_track(&self.track, &self.track_smooth_elevation);
        let W = self.get_steps();
        profile.add_waypoints(&W);
        let ret = profile.render();
        //let filename = std::format!("/tmp/segment-{}.svg", segment.id);
        //std::fs::write(filename, &ret).expect("Unable to write file");
        ret
    }
    pub fn render_segment_track(&mut self, segment: &Segment, (W, H): (i32, i32)) -> String {
        println!("render_segment_track:{}", segment.id);
        let mut profile = segment.profile.clone();
        profile.reset_size(W, H);
        profile.add_canvas();
        profile.add_track(&self.track, &self.track_smooth_elevation);
        let ret = profile.render();
        let filename = std::format!("/tmp/track-{}.svg", segment.id);
        println!("rendered {}", filename);
        //std::fs::write(filename, &ret).expect("Unable to write file");
        ret
    }
    pub fn render_segment_waypoints(&mut self, segment: &Segment, (W, H): (i32, i32)) -> String {
        println!("render_segment_track:{}", segment.id);
        let mut profile = segment.profile.clone();
        profile.reset_size(W, H);
        let W = self.get_steps();
        profile.add_waypoints(&W);
        let ret = profile.render();
        let _filename = std::format!("/tmp/waypoints-{}.svg", segment.id);
        // TODO: compile if not wasm
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
    pub fn generatePdf(&mut self) -> Vec<u8> {
        let typbytes = render::compile(self, (1400, 400));
        let ret = pdf::compile(&typbytes);
        println!("generated {} bytes", ret.len());
        ret
    }
    pub fn generateGpx(&mut self) -> Vec<u8> {
        gpxexport::generate_pdf(&self.track, &self.waypoints)
    }
}

#[cfg(test)]
mod tests {
    use crate::backend::Backend;
    #[test]
    fn svg_segment_track() {
        let mut backend = Backend::from_filename("data/blackforest.gpx");
        let segments = backend.segments();
        let mut ok_count = 0;
        for segment in &segments {
            let svg = backend.render_segment_track(&segment, (1420, 400));
            let reffilename = std::format!("data/ref/track-{}.svg", segment.id);
            println!("test {}", reffilename);
            if !std::fs::exists(&reffilename).unwrap() {
                continue;
            }
            let data = std::fs::read_to_string(&reffilename).unwrap();
            if data == svg {
                ok_count += 1;
            } else {
                println!("test failed");
            }
        }
        assert!(ok_count == segments.len());
    }

    #[test]
    fn svg_segment_waypoints() {
        let mut backend = Backend::from_filename("data/blackforest.gpx");
        let segments = backend.segments();
        let mut ok_count = 0;
        for segment in &segments {
            let svg = backend.render_segment_waypoints(&segment, (1420, 400));
            let reffilename = std::format!("data/ref/waypoints-{}.svg", segment.id);
            println!("test {}", reffilename);
            if !std::fs::exists(&reffilename).unwrap() {
                continue;
            }
            let data = std::fs::read_to_string(&reffilename).unwrap();
            if data == svg {
                ok_count += 1;
            } else {
                println!("test failed");
            }
        }
        assert!(ok_count == segments.len());
    }

    #[test]
    fn time_iso8601() {
        let mut backend = Backend::from_filename("data/blackforest.gpx");
        backend.setStartTime(String::from("2007-03-01T13:00:00Z"));
        backend.setStartTime(String::from("2025-07-12T06:32:36Z"));
        backend.setStartTime(String::from("2025-07-12T06:32:36.215033Z"));
    }
}
