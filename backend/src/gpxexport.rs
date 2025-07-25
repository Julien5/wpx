#![allow(non_snake_case)]

use std::str::FromStr;

use crate::gpsdata;

fn gps_name(w: &gpsdata::Waypoint) -> String {
    match &w.step {
        Some(step) => {
            use chrono::*;
            let t: DateTime<Utc> = step.time.parse().unwrap();
            let time = format!("{}", t.format("%H:%M"));
            let slope = format!("{:.1}", 100f64 * step.inter_slope);
            return format!("{}-{}%", time, slope);
        }
        _ => {}
    }
    match &w.name {
        Some(s) => {
            return s.clone();
        }
        _ => {}
    }
    String::from_str("no data").unwrap()
}

fn to_gpx(w: &gpsdata::Waypoint) -> gpx::Waypoint {
    let mut ret = gpx::Waypoint::new(geo::Point::new(w.wgs84.0, w.wgs84.1));
    ret.elevation = Some(w.wgs84.2);
    ret.name = Some(gps_name(w));
    ret.description = None;
    ret
}

pub fn generate(track: &gpsdata::Track, waypoints: &Vec<gpsdata::Waypoint>) -> Vec<u8> {
    let mut G = gpx::Gpx::default();
    G.version = gpx::GpxVersion::Gpx11;

    let segment = track.to_segment();
    let mut track = gpx::Track::new();
    track.segments.push(segment);
    G.tracks.push(track);

    for w in waypoints {
        let g = to_gpx(&w);
        G.waypoints.push(g);
    }

    let mut ret: Vec<u8> = Vec::new();
    gpx::write(&G, &mut ret).unwrap();
    ret
}
