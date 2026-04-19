#![allow(non_snake_case)]

use std::collections::BTreeMap;

use gpx::TrackSegment;

use crate::inputpoint::InputPoint;
use crate::point_collection::Kind;
use crate::track;
use crate::trackparts::control_to_segments;
use crate::waypoint;
use crate::waypoint::Waypoints;
use crate::wgs84point::WGS84Point;

fn gps_name(w: &waypoint::Waypoint) -> String {
    match &w.origin {
        Kind::UserStep => match &w.info {
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
        },
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

pub fn elevated_export(wgs84: &Vec<WGS84Point>, range: &std::ops::Range<usize>) -> TrackSegment {
    let mut ret = TrackSegment::new();
    for index in range.start..range.end {
        // remove z coordinate to avoid automatic "low" and "hight points" on etrex 10
        let wgs = wgs84[index];
        let mut w = gpx::Waypoint::new(geo::Point::new(wgs.x(), wgs.y()));
        w.elevation = Some(wgs.z());
        ret.points.push(w);
    }
    ret
}

fn write_gpx_file(
    waypoints: Vec<gpx::Waypoint>,
    tracks: Vec<gpx::Track>,
    filename: &str,
    result: &mut BTreeMap<String, Vec<u8>>,
) {
    let mut gpx = gpx::Gpx::default();
    gpx.version = gpx::GpxVersion::Gpx11;
    gpx.waypoints = waypoints;
    gpx.tracks = tracks;
    let mut data: Vec<u8> = Vec::new();
    gpx::write(&gpx, &mut data).unwrap();
    result.insert(filename.to_string(), data);
}

pub fn generate(
    track: &track::Track,
    controls: &Vec<InputPoint>,
    groups: &Vec<Waypoints>,
    waypoints: &Waypoints,
) -> BTreeMap<String, Vec<u8>> {
    let mut ret: BTreeMap<String, Vec<u8>> = BTreeMap::new();
    let parts = control_to_segments(&track, &controls);

    let mut archive_tracks = Vec::new();
    for (index, part) in parts.iter().enumerate() {
        let range = &part.range;

        // Flat segment
        let segment = flat_export(&track.wgs84, &range);
        let mut gpxtrack = gpx::Track::new();
        gpxtrack.name = Some(format!("[flat] {:0>2}: {}", index + 1, part.name));
        gpxtrack.segments.push(segment);
        write_gpx_file(
            vec![],
            vec![gpxtrack],
            &format!("flat-segment-{:0>2}.gpx", index + 1),
            &mut ret,
        );

        // Elevated segment
        let segment = elevated_export(&track.wgs84, &range);
        let mut gpxtrack = gpx::Track::new();
        gpxtrack.name = Some(format!("[ele] {:0>2}: {}", index + 1, part.name));
        gpxtrack.segments.push(segment);
        write_gpx_file(
            vec![],
            vec![gpxtrack.clone()],
            &format!("elevated-segment-{:0>2}.gpx", index + 1),
            &mut ret,
        );

        let mut gpxtrack_archive = gpxtrack.clone();
        gpxtrack_archive.name = Some(format!("{:0>2}: {}", index + 1, part.name));
        archive_tracks.push(gpxtrack_archive);
    }

    // Archive with all waypoints and tracks
    let archive_waypoints: Vec<gpx::Waypoint> = waypoints.iter().map(|w| to_gpx(w)).collect();
    write_gpx_file(
        archive_waypoints,
        archive_tracks,
        "track-waypoints.gpx",
        &mut ret,
    );

    // Export all waypoints in one file
    let all_waypoints: Vec<gpx::Waypoint> = waypoints.iter().map(|w| to_gpx(w)).collect();
    write_gpx_file(all_waypoints, vec![], "waypoints-all.gpx", &mut ret);

    // Export all user steps in one file
    let all: Waypoints = groups.iter().flatten().cloned().collect();
    let all_pacing: Vec<gpx::Waypoint> = all.iter().map(|w| to_gpx(w)).collect();
    write_gpx_file(all_pacing, vec![], "pacing-all.gpx", &mut ret);

    // Export individual groups if there are several
    if groups.len() > 1 {
        for (index, group) in groups.iter().enumerate() {
            let group_waypoints: Vec<gpx::Waypoint> = group.iter().map(|w| to_gpx(w)).collect();
            write_gpx_file(
                group_waypoints,
                vec![],
                &format!("pacing-{}.gpx", index + 1),
                &mut ret,
            );
        }
    }

    ret
}
