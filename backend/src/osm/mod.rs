mod cache;
mod download;
mod filesystem;
#[cfg(target_arch = "wasm32")]
mod indexdb;
pub mod osmpoint;

use crate::bboxes::*;
use crate::gpsdata::distance_wgs84;
use crate::project;
use crate::track::*;
use crate::waypoint::*;
use osmpoint::*;
use std::collections::BTreeMap;

pub fn osm3(bbox: &WGS84BoundingBox) -> String {
    format!(
        "({:0.3},{:0.3},{:0.3},{:0.3})",
        bbox.min.1, bbox.min.0, bbox.max.1, bbox.max.0
    )
}

fn retain(waypoints: &mut Waypoints, track: &Track, delta: f64) {
    // TODO: make that faster, by holding the tree (?)
    project::project_on_track(track, waypoints);
    waypoints.retain(|w| {
        if w.track_index.is_none() {
            return false;
        }
        let index = w.track_index.unwrap();
        let d = distance_wgs84(&track.wgs84[index], &w.wgs84);
        match w.name.as_ref() {
            Some(name) => {
                if name.contains("Brest") || name.contains("Carhaix") {
                    log::info!("name={name} => d={} delta={}", d, delta);
                    log::info!("p1={:?}", &w.wgs84);
                    log::info!("p2={:?}", &track.wgs84[index]);
                }
            }
            _ => {}
        }
        d < delta
    })
}

pub fn convert_osmpoints(osmpoints: &OSMPoints) -> Waypoints {
    log::trace!("project points");
    let mut ret = Vec::new();
    for city in &osmpoints.points {
        let (lon, lat) = (city.lon, city.lat);
        let ele = match city.ele() {
            Some(m) => m,
            None => 0f64,
        };
        let w = Waypoint {
            wgs84: WGS84Point::new(&lon, &lat, &ele),
            track_index: None,
            name: city.name().clone(),
            description: None,
            info: None,
            origin: WaypointOrigin::OpenStreetMap,
        };
        ret.push(w);
    }
    log::trace!("project points done");
    ret
}

async fn download_chunk_real(
    bbox: &WGS84BoundingBox,
    kind: &str,
) -> std::result::Result<OSMPoints, std::io::Error> {
    use download::*;
    let bboxparam = osm3(&bbox);
    let result = if kind == "passes" {
        parse_osm_content(passes(&bboxparam).await.unwrap().as_bytes())
    } else {
        parse_osm_content(places(&bboxparam, kind).await.unwrap().as_bytes())
    };
    match result {
        Ok(points) => Ok(points),
        Err(e) => {
            log::info!("could not download {} (ignore)", kind);
            log::info!("reason: {}", e.to_string());
            Err(std::io::Error::new(std::io::ErrorKind::InvalidData, kind))
        }
    }
}

async fn download_chunk(bboxes: &Vec<WGS84BoundingBox>, kind: &str) -> OSMPoints {
    if bboxes.is_empty() {
        return OSMPoints::new();
    }
    let bbox = bounding_box(&bboxes);
    let osmpoints = match download_chunk_real(&bbox, kind).await {
        Ok(points) => {
            log::info!("downloaded {:3} {:20}", points.points.len(), kind);
            cache::write(bboxes, &points, kind).await;
            points
        }
        Err(e) => {
            log::info!("error downloading for {:20}: {:?}", kind, e);
            log::info!("assuming there is no {}", kind);
            OSMPoints::new()
        }
    };
    osmpoints
}

async fn read(bbox: &WGS84BoundingBox, kind: &str) -> OSMPoints {
    let osmpoints = match cache::read(bbox, kind).await {
        Some(d) => d,
        None => {
            // "could not find any data for {} (download probably failed) => skip",
            OSMPoints::new()
        }
    };
    osmpoints
}

async fn reducebbox(bbox: &WGS84BoundingBox, kind: &str, step: &f64) -> Vec<WGS84BoundingBox> {
    let many = split(&bbox, step);
    let mut uncached = Vec::new();
    for (_index, atom) in many {
        if !(cache::hit_cache(&atom, kind).await) {
            uncached.push(atom.clone());
        }
    }
    uncached
}

async fn process(bbox: &WGS84BoundingBox, kind: &str) -> OSMPoints {
    let step = if kind == "village" {
        0.05f64 // ~ 5km
    } else {
        0.2f64 // ~ 20km
    };
    let atoms = split(&bbox, &step);
    let not_cached = reducebbox(&bbox, &kind, &step).await;
    if !not_cached.is_empty() {
        log::info!("atoms:{}", atoms.len());
        log::info!("not in cache:{}", not_cached.len());
    }
    download_chunk(&not_cached, kind).await;
    let mut ret = Vec::new();
    log::trace!("about to read {:20} atoms:{:3}", kind, atoms.len());
    for (_index, atom) in atoms {
        let points = read(&atom, &kind).await;
        ret.extend(points.points);
    }
    log::trace!("done");
    OSMPoints { points: ret }
}

pub type OSMWaypoints = BTreeMap<OSMType, Waypoints>;

pub async fn download_for_track(track: &Track, distance: f64) -> OSMWaypoints {
    let mut ret = OSMWaypoints::new();
    let bbox = track.wgs84_bounding_box();
    assert!(!bbox.empty());

    let mut cities = convert_osmpoints(&process(&bbox, "town").await);
    retain(&mut cities, track, 10f64 * distance);
    ret.insert(OSMType::City, cities);

    let mut passes = convert_osmpoints(&process(&bbox, "passes").await);
    retain(&mut passes, track, 2f64 * distance);
    ret.insert(OSMType::MountainPass, passes);

    let mut villages = process(&bbox, "village").await;
    villages.points.retain(|p| match p.population() {
        Some(number) => number >= 1000,
        None => false,
    });
    let mut v = convert_osmpoints(&villages);
    retain(&mut v, track, distance * 0.5f64);
    ret.insert(OSMType::Village, v);

    ret
}
