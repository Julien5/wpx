#![allow(non_snake_case)]

use crate::gpsdata;

pub fn generate_pdf(track: &gpsdata::Track, waypoints: &Vec<gpsdata::Waypoint>) -> Vec<u8> {
    let mut G = gpx::Gpx::default();
    G.version = gpx::GpxVersion::Gpx11;

    let segment = track.to_segment();
    let mut track = gpx::Track::new();
    track.segments.push(segment);
    G.tracks.push(track);

    for w in waypoints {
        G.waypoints.push(w.to_gpx());
    }

    let mut ret: Vec<u8> = Vec::new();
    gpx::write(&G, &mut ret).unwrap();
    ret
}
