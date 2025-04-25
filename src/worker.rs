use crate::gpsdata;
use crate::pdf;
use crate::project;
use crate::render;

pub fn worker(filename: &str) {
    println!("read gpx");
    let mut gpx = gpsdata::read_gpx(filename);
    let segment = gpsdata::read_segment(&mut gpx);
    println!("make track");
    let track = gpsdata::Track::from_segment(&segment);
    println!("make waypoints");
    let mut waypoints = gpsdata::read_waypoints(&gpx);
    println!("add automatic waypoints");
    let indexes = track.interesting_indexes();
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
    println!("render");
    let typfile = render::compile(&track, &waypoints);
    println!("make pdf");
    let pdffile = typfile.replace(".typ", ".pdf");
    pdf::run(typfile.as_str(), pdffile.as_str());
}
