#![allow(non_snake_case)]
use crate::gpsdata;
use crate::pdf;
use crate::project;
use crate::render;

pub struct Backend {
    epsilon: f32,
}

impl Backend {
    pub fn new() -> Backend {
        let ret = Backend { epsilon: 70.0f32 };
        ret
    }
    pub fn changeParameter(&mut self, delta: f32) {
        self.epsilon += delta;
    }
    pub fn testSvg(&self) -> String {
        let mut gpx = gpsdata::read_gpx("/tmp/track.gpx");
        let segment = gpsdata::read_segment(&mut gpx);
        println!("make track");
        let track = gpsdata::Track::from_segment(&segment);
        println!("make waypoints");
        let mut waypoints = gpsdata::read_waypoints(&gpx);
        println!("add automatic waypoints with {}", self.epsilon);
        let indexes = track.interesting_indexes(self.epsilon);
        for idx in indexes {
            let wgs = track.wgs84[idx].clone();
            let utm = track.utm[idx].clone();
            waypoints.push(gpsdata::Waypoint::from_track(wgs, utm, idx));
        }
        println!("project waypoints");
        let indexes = project::nearest_neighboor(&track.utm, &waypoints);
        debug_assert_eq!(waypoints.len(), indexes.len());
        for k in 0..indexes.len() {
            waypoints[k].track_index = indexes[k];
        }
        // sort
        waypoints.sort_by(|w1, w2| w1.track_index.cmp(&w2.track_index));
        for k in 1..waypoints.len() {
            let k1 = waypoints[k].track_index;
            let k0 = waypoints[k - 1].track_index;
            // println!("{}:{}", k0, k1);
            debug_assert!(k1 >= k0);
        }
        println!("render");
        render::test_svg(&track, &waypoints)
    }
}

pub fn worker(filename: &str) {
    println!("read gpx");
    let mut gpx = gpsdata::read_gpx(filename);
    let segment = gpsdata::read_segment(&mut gpx);
    println!("make track");
    let track = gpsdata::Track::from_segment(&segment);
    println!("make waypoints");
    let mut waypoints = gpsdata::read_waypoints(&gpx);
    println!("add automatic waypoints");
    let indexes = track.interesting_indexes(70.0f32);
    for idx in indexes {
        let wgs = track.wgs84[idx].clone();
        let utm = track.utm[idx].clone();
        waypoints.push(gpsdata::Waypoint::from_track(wgs, utm, idx));
    }
    println!("project waypoints");
    let indexes = project::nearest_neighboor(&track.utm, &waypoints);
    debug_assert_eq!(waypoints.len(), indexes.len());
    for k in 0..indexes.len() {
        waypoints[k].track_index = indexes[k];
    }
    // sort
    waypoints.sort_by(|w1, w2| w1.track_index.cmp(&w2.track_index));
    for k in 1..waypoints.len() {
        let k1 = waypoints[k].track_index;
        let k0 = waypoints[k - 1].track_index;
        println!("{}:{}", k0, k1);
        debug_assert!(k1 >= k0);
    }
    println!("render");
    let typfile = render::compile(&track, &waypoints);
    println!("make pdf");
    let pdffile = typfile.replace(".typ", ".pdf");
    pdf::run(typfile.as_str(), pdffile.as_str());
}
