#![allow(non_snake_case)]

use std::collections::BTreeMap;

use gpx::TrackSegment;

use crate::track;
use crate::trackparts::parts_to_ranges;
use crate::waypoint;
use crate::waypoint::Waypoints;
use crate::wgs84point::WGS84Point;

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

pub fn flat_export(wgs84: &Vec<WGS84Point>, range: &std::ops::Range<usize>) -> TrackSegment {
    let mut ret = TrackSegment::new();
    for index in range.start..range.end {
        // remove z coordinate to avoid automatic "low" and "hight points" on etrex 10
        let wgs = wgs84[index];
        let w = gpx::Waypoint::new(geo::Point::new(wgs.x(), wgs.y()));
        ret.points.push(w);
    }
    ret
}

pub fn generate(track: &track::Track, groups: &Vec<Waypoints>) -> BTreeMap<String, Vec<u8>> {
    let mut ret: BTreeMap<String, Vec<u8>> = BTreeMap::new();
    {
        let mut G = gpx::Gpx::default();
        G.version = gpx::GpxVersion::Gpx11;

        let segment = flat_export(
            &track.wgs84,
            &std::ops::Range {
                start: 0,
                end: track.wgs84.len(),
            },
        );
        let mut gpxtrack = gpx::Track::new();
        gpxtrack.name = Some(format!("{:.0} km", track.total_distance() / 1000f64));
        gpxtrack.segments.push(segment);
        G.tracks.push(gpxtrack);
        let mut data: Vec<u8> = Vec::new();
        gpx::write(&G, &mut data).unwrap();
        ret.insert("flat-track.gpx".to_string(), data);
    };

    let ranges = parts_to_ranges(&track.parts);
    for (index, part) in track.parts.iter().enumerate() {
        let mut G = gpx::Gpx::default();
        G.version = gpx::GpxVersion::Gpx11;
        let segment = flat_export(&track.wgs84, &ranges[index]);
        let mut gpxtrack = gpx::Track::new();
        gpxtrack.name = Some(format!("{:0>2}: {}", index + 1, part.name));
        gpxtrack.segments.push(segment);
        G.tracks.push(gpxtrack);
        let mut data: Vec<u8> = Vec::new();
        gpx::write(&G, &mut data).unwrap();
        ret.insert(format!("flat-segment-{:0>2}.gpx", index + 1), data);
    }

    // export all users steps in one file (maybe useful)
    {
        let all: Waypoints = groups.iter().flatten().cloned().collect();
        let mut G = gpx::Gpx::default();
        G.version = gpx::GpxVersion::Gpx11;
        G.waypoints = all.iter().map(|w| to_gpx(w)).collect();

        let mut data: Vec<u8> = Vec::new();
        gpx::write(&G, &mut data).unwrap();
        ret.insert(format!("pacing-all.gpx"), data);
    }

    // only if there are several groups, export them.
    if groups.len() > 1 {
        for (index, group) in groups.iter().enumerate() {
            let mut G = gpx::Gpx::default();
            G.version = gpx::GpxVersion::Gpx11;
            G.waypoints = group.iter().map(|w| to_gpx(w)).collect();

            let mut data: Vec<u8> = Vec::new();
            gpx::write(&G, &mut data).unwrap();
            ret.insert(format!("pacing-{}.gpx", index + 1), data);
        }
    }

    ret
}
