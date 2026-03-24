#![allow(non_snake_case)]

use std::collections::BTreeMap;

use crate::track;
use crate::waypoint;
use crate::waypoint::Waypoints;

fn gps_name(w: &waypoint::Waypoint) -> String {
    match &w.info {
        Some(step) => {
            use chrono::*;
            let t: DateTime<Local> = step.time.parse().unwrap();
            let time = format!("{}", t.format("%H:%M"));
            let percent = 100f64 * step.inter_slope;
            let info = if true {
                format!("{:.1}%", percent)
            } else if !w.name.is_empty() {
                format!("{}", w.name)
            } else {
                format!("{:.1}%", percent)
            };
            return format!("{}-{}", time, info);
        }
        _ => {}
    }
    w.name.clone()
}

fn to_gpx(w: &waypoint::Waypoint) -> gpx::Waypoint {
    let mut ret = gpx::Waypoint::new(geo::Point::new(w.wgs84.x(), w.wgs84.y()));
    ret.elevation = Some(w.wgs84.z());
    ret.name = Some(gps_name(w));
    ret.description = match &w.info {
        Some(info) => Some(info.description.clone()),
        _ => Some(w.description.clone()),
    };
    ret
}

pub fn generate(track: &track::Track, groups: &Vec<Waypoints>) -> BTreeMap<String, Vec<u8>> {
    let mut ret: BTreeMap<String, Vec<u8>> = BTreeMap::new();
    let track = {
        let mut G = gpx::Gpx::default();
        G.version = gpx::GpxVersion::Gpx11;

        let segment = track.export_to_gpx();
        let mut gpxtrack = gpx::Track::new();
        gpxtrack.name = Some(format!("{:.0} km", track.total_distance() / 1000f64));
        gpxtrack.segments.push(segment);
        G.tracks.push(gpxtrack);
        let mut ret: Vec<u8> = Vec::new();
        gpx::write(&G, &mut ret).unwrap();
        ("track.gpx".to_string(), ret)
    };
    ret.insert(track.0, track.1);

    for (index, group) in groups.iter().enumerate() {
        let waypoints = {
            let mut G = gpx::Gpx::default();
            G.version = gpx::GpxVersion::Gpx11;
            G.waypoints = group.iter().map(|w| to_gpx(w)).collect();

            let mut ret: Vec<u8> = Vec::new();
            gpx::write(&G, &mut ret).unwrap();
            (format!("pacing-{}.gpx", index + 1), ret)
        };
        ret.insert(waypoints.0, waypoints.1);
    }
    ret
}
