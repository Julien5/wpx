#![allow(non_snake_case)]

use std::str::FromStr;

use chrono::TimeZone;

use crate::elevation;
use crate::gpsdata;
use crate::gpsdata::ProfileBoundingBox;
use crate::gpsdata::WaypointOrigin;
use crate::project;
use crate::render;
use crate::speed;

pub struct Backend {
    pub track: gpsdata::Track,
    track_smooth_elevation: Vec<f64>,
    pub waypoints: Vec<gpsdata::Waypoint>,
    epsilon: f32,
    pub shift: f64,
    start_time: i64, // seconds since epoch
    speed: f64,      // m/s
}

#[derive(Clone)]
pub struct Segment {
    pub id: usize,
    pub range: std::ops::Range<usize>,
}

pub struct SegmentStatistics {
    pub length: f64,
    pub elevation_gain: f64,
    pub distance_start: f64,
    pub distance_end: f64,
}

#[derive(Clone)]
pub struct WayPoint {
    pub wgs84: (f64, f64, f64),
    pub utm: gpsdata::UTMPoint,
    pub origin: gpsdata::WaypointOrigin,
    pub distance: f64,
    pub elevation: f64,
    pub inter_distance: f64,
    pub inter_elevation_gain: f64,
    pub inter_slope: f64,
    pub name: String,
    pub time: i64, // seconds since epoch
    pub track_index: usize,
}

impl Segment {
    pub fn new(id: usize, range: std::ops::Range<usize>) -> Segment {
        Segment { id, range }
    }
}

fn waypoint_time(start_time: i64, distance: f64, speed: f64) -> i64 {
    let dt = (distance / speed).ceil() as i64;
    start_time + dt
}

impl Backend {
    fn create_waypoint(
        self: &Backend,
        w: &gpsdata::Waypoint,
        wprev: Option<&gpsdata::Waypoint>,
    ) -> WayPoint {
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
        WayPoint {
            wgs84: w.wgs84,
            utm: w.utm.clone(),
            origin: w.origin.clone(),
            distance: distance,
            inter_distance: inter_distance,
            inter_elevation_gain: inter_elevation_gain,
            inter_slope: inter_slope,
            elevation: track.elevation(w.track_index),
            name: name,
            time: waypoint_time(self.start_time, distance, self.speed),
            track_index: w.track_index,
        }
    }
    pub fn get_waypoints(&self) -> Vec<WayPoint> {
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
    pub fn setStartTime(&mut self, t: i64) {
        self.start_time = t;
    }
    pub fn setSpeed(&mut self, s: f64) {
        self.speed = s;
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

    pub fn new(filename: &str) -> Backend {
        let filename = String::from_str(filename).unwrap();
        let mut gpx = gpsdata::read_gpx(filename.as_str());
        let segment = gpsdata::read_segment(&mut gpx);
        let track = gpsdata::Track::from_segment(&segment);
        let km = 1000f64;
        let mut ret = Backend {
            track_smooth_elevation: elevation::smooth(&track),
            track: track,
            waypoints: gpsdata::read_waypoints(&gpx),
            epsilon: 70.0f32,
            shift: 100f64 * km,
            start_time: chrono::Utc
                .with_ymd_and_hms(2024, 4, 4, 8, 0, 0)
                .unwrap()
                .timestamp(),
            speed: speed::mps(15f64),
        };
        ret.enrichWaypoints();
        for w in &ret.waypoints {
            debug_assert!(w.track_index < ret.track.len());
        }
        ret
    }

    pub fn adjustEpsilon(&mut self, delta: f32) {
        self.epsilon += delta;
    }

    pub fn epsilon(&self) -> f32 {
        self.epsilon
    }

    pub fn render_track(&mut self) -> String {
        self.enrichWaypoints();
        let range = 0..self.track.len();
        let viewBox = ProfileBoundingBox::from_track(&self.track, &range);
        render::track_profile(&self.track, &range, &viewBox)
    }
    pub fn render_waypoints(&mut self) -> String {
        self.enrichWaypoints();
        let range = 0..self.track.len();
        let viewBox = ProfileBoundingBox::from_track(&self.track, &range);
        let W = self.get_waypoints();
        render::waypoints_profile(&self.track, &W, &range, &viewBox)
    }

    pub fn segments(&self) -> Vec<Segment> {
        let mut ret = Vec::new();

        let mut start = 0f64;
        let mut k = 0usize;
        loop {
            let end = start + self.shift;
            let range = self.track.segment(start, end);
            if range.is_empty() {
                break;
            }
            ret.push(Segment::new(k, range));
            start = start + self.shift;
            k = k + 1;
        }
        ret
    }
    pub fn render_segment_track(&mut self, segment: &Segment) -> String {
        println!("render_segment_track:{}", segment.id);
        let range = &segment.range;
        self.enrichWaypoints();
        let bbox = ProfileBoundingBox::from_track(&self.track, &range);
        let ret = render::track_profile(&self.track, &range, &bbox);
        let filename = std::format!("/tmp/track-{}.svg", segment.id);
        std::fs::write(filename, &ret).expect("Unable to write file");
        ret
    }
    pub fn render_segment_waypoints(&mut self, segment: &Segment) -> String {
        println!("render_segment_waypoints:{}", segment.id);
        let range = &segment.range;
        self.enrichWaypoints();
        let bbox = ProfileBoundingBox::from_track(&self.track, &range);
        let W = self.get_waypoints();
        let ret = render::waypoints_profile(&self.track, &W, &range, &bbox);
        let filename = std::format!("/tmp/waypoints-{}.svg", segment.id);
        std::fs::write(filename, &ret).expect("Unable to write file");
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
}
