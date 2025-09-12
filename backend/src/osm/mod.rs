mod cache;
mod download;
pub mod osmpoint;

use crate::bboxes::bounding_box;
use crate::bboxes::split;
use crate::gpsdata::distance_wgs84;
use crate::project;
use crate::track::Track;
use crate::track::WGS84BoundingBox;
use crate::utm::UTMPoint;
use crate::waypoint::Waypoint;
use crate::waypoint::WaypointOrigin;
use crate::waypoint::Waypoints;
use cache::*;
use osmpoint::*;
use std::collections::BTreeMap;
use std::path::Path;

use proj4rs::Proj;

pub fn osm3(bbox: &WGS84BoundingBox) -> String {
    format!(
        "({:0.3},{:0.3},{:0.3},{:0.3})",
        bbox.min.1, bbox.min.0, bbox.max.1, bbox.max.0
    )
}

fn retain(waypoints: &mut Waypoints, track: &Track, delta: f64) {
    project::project_on_track(track, waypoints);
    waypoints.retain(|w| {
        let index = w.track_index.unwrap();
        let p1 = (track.wgs84[index].0, track.wgs84[index].1);
        let p2 = (w.wgs84.0, w.wgs84.1);
        let d = distance_wgs84(p1.0, p1.1, p2.0, p2.1);
        d < delta
    })
}

pub fn convert_osmpoints(osmpoints: &OSMPoints) -> Waypoints {
    let mut ret = Vec::new();
    // TODO: fix proj!
    let spec = "+proj=longlat +ellps=WGS84 +datum=WGS84 +no_defs";
    let wgs84 = Proj::from_proj_string(spec).unwrap();
    let spec = "+proj=utm +zone=32 +datum=WGS84 +units=m +no_defs +type=crs";
    let utm32n = Proj::from_proj_string(spec).unwrap();
    for city in &osmpoints.points {
        let (lon, lat) = (city.lon, city.lat);
        let mut p = (lon.to_radians(), lat.to_radians());
        proj4rs::transform::transform(&wgs84, &utm32n, &mut p).unwrap();
        let ele = match city.ele {
            Some(m) => m,
            None => 0f64,
        };
        let w = Waypoint {
            wgs84: (lon, lat, ele),
            utm: UTMPoint(p.0, p.1),
            track_index: None,
            name: city.name.clone(),
            description: None,
            info: None,
            origin: WaypointOrigin::OpenStreetMap,
        };
        ret.push(w);
    }
    ret
}

fn download_chunk_real(
    bbox: &WGS84BoundingBox,
    kind: &str,
) -> std::result::Result<OSMPoints, std::io::Error> {
    use download::*;
    let bboxparam = osm3(&bbox);
    let result = if kind == "passes" {
        parse_osm_content(passes(&bboxparam).unwrap().as_bytes())
    } else {
        parse_osm_content(places(&bboxparam, kind).unwrap().as_bytes())
    };
    match result {
        Ok(points) => Ok(points),
        Err(e) => {
            println!("could not download {} (ignore)", kind);
            println!("reason: {}", e.to_string());
            Err(std::io::Error::new(std::io::ErrorKind::InvalidData, kind))
        }
    }
}

fn hit_cache(bbox: &WGS84BoundingBox, kind: &str) -> bool {
    let filename = cache_filename(bbox, kind);
    let path = Path::new(filename.as_str());
    path.exists()
}

fn download_chunk(bboxes: &Vec<WGS84BoundingBox>, kind: &str) -> OSMPoints {
    if bboxes.is_empty() {
        return OSMPoints::new();
    }
    let bbox = bounding_box(&bboxes);
    let osmpoints = match download_chunk_real(&bbox, kind) {
        Ok(points) => {
            println!("downloaded {:3} {:20}", points.points.len(), kind);
            cache::write_chunk_cache(bboxes, &points, kind);
            points
        }
        Err(e) => {
            println!("error downloading for {:20}: {:?}", kind, e);
            println!("assuming there is no {}", kind);
            OSMPoints::new()
        }
    };
    osmpoints
}

fn read(bbox: &WGS84BoundingBox, kind: &str) -> Waypoints {
    let osmpoints = match cache::read_chunk_cache(bbox, kind) {
        Some(d) => d,
        None => {
            // "could not find any data for {} (download probably failed) => skip",
            OSMPoints::new()
        }
    };
    convert_osmpoints(&osmpoints)
}

fn reducebbox(bbox: &WGS84BoundingBox, kind: &str, step: &f64) -> Vec<WGS84BoundingBox> {
    let many = split(&bbox, step);
    let mut uncached = Vec::new();
    for (_index, atom) in many {
        if !hit_cache(&atom, kind) {
            uncached.push(atom.clone());
        }
    }
    uncached
}

fn process(bbox: &WGS84BoundingBox, kind: &str) -> Waypoints {
    let step = if kind == "village" {
        0.05f64 // ~ 5km
    } else {
        0.2f64 // ~ 20km
    };
    let atoms = split(&bbox, &step);
    // download missing
    let not_cached = reducebbox(&bbox, &kind, &step);
    if !not_cached.is_empty() {
        println!("atoms:{}", atoms.len());
        println!("not in cache:{}", not_cached.len());
    }
    download_chunk(&not_cached, kind);
    let mut ret = Vec::new();
    println!("about to read {:20} atoms:{:3}", kind, atoms.len());
    for (_index, atom) in atoms {
        let points = read(&atom, &kind);
        ret.extend(points);
    }
    ret
}

pub type OSMWaypoints = BTreeMap<OSMType, Waypoints>;

pub fn download_for_track(track: &Track, distance: f64) -> OSMWaypoints {
    let mut ret = OSMWaypoints::new();
    let bbox = track.wgs84_bounding_box();
    assert!(!bbox.empty());

    let mut cities = process(&bbox, "town");
    retain(&mut cities, track, 10f64 * distance);
    ret.insert(OSMType::City, cities);

    let mut passes = process(&bbox, "passes");
    retain(&mut passes, track, 2f64 * distance);
    ret.insert(OSMType::MountainPass, passes);

    let mut villages = process(&bbox, "village");
    retain(&mut villages, track, distance * 0.5f64);
    ret.insert(OSMType::Village, villages);

    ret
}
