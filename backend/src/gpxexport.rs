#![allow(non_snake_case)]

use crate::track;
use crate::waypoint;

fn gps_name(w: &waypoint::Waypoint) -> Option<String> {
    match &w.info {
        Some(step) => {
            use chrono::*;
            let t: DateTime<Utc> = step.time.parse().unwrap();
            let time = format!("{}", t.format("%H:%M"));
            let percent = 100f64 * step.inter_slope;
            let info = if percent > 1f64 {
                format!("{:.1}%", percent)
            } else if w.name.is_some() {
                format!("{}", w.name.as_ref().unwrap())
            } else {
                format!("{:.1}%", percent)
            };
            return Some(format!("{}-{}", time, info));
        }
        _ => {}
    }
    w.name.clone()
}

fn to_gpx(w: &waypoint::Waypoint) -> gpx::Waypoint {
    let mut ret = gpx::Waypoint::new(geo::Point::new(w.wgs84.0, w.wgs84.1));
    ret.elevation = Some(w.wgs84.2);
    ret.name = gps_name(w);
    ret.description = match &w.info {
        Some(step) => Some(step.description.clone()),
        _ => w.description.clone(),
    };
    ret
}

pub fn generate(track: &track::Track, waypoints: &Vec<waypoint::Waypoint>) -> Vec<u8> {
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
