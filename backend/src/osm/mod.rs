mod cache;
mod download;
mod filesystem;
#[cfg(target_arch = "wasm32")]
mod indexdb;
pub mod osmpoint;

use crate::bboxes::*;
use crate::gpsdata::distance_wgs84;
use crate::inputpoint::InputPoint;
use crate::inputpoint::InputPoints;
use crate::inputpoint::InputType;
use crate::project;
use crate::track::*;

pub fn osm3(bbox: &WGS84BoundingBox) -> String {
    format!(
        "({:0.3},{:0.3},{:0.3},{:0.3})",
        bbox.min.1, bbox.min.0, bbox.max.1, bbox.max.0
    )
}

async fn download_chunk_real(
    bbox: &WGS84BoundingBox,
) -> std::result::Result<InputPoints, std::io::Error> {
    use download::*;
    let bboxparam = osm3(&bbox);
    let result = parse_osm_content(all(&bboxparam).await.unwrap().as_bytes());
    match result {
        Ok(points) => Ok(points),
        Err(e) => {
            log::info!("could not download(ignore)");
            log::info!("reason: {}", e.to_string());
            Err(std::io::Error::new(std::io::ErrorKind::InvalidData, "data"))
        }
    }
}

async fn download_chunk(bboxes: &Vec<WGS84BoundingBox>) -> InputPoints {
    if bboxes.is_empty() {
        return InputPoints::new();
    }
    let bbox = bounding_box(&bboxes);
    let osmpoints = match download_chunk_real(&bbox).await {
        Ok(points) => {
            log::info!("downloaded {:3}", points.points.len());
            cache::write(bboxes, &points).await;
            points
        }
        Err(e) => {
            log::info!("error downloading: {:?}", e);
            log::info!("assuming there is nothing");
            InputPoints::new()
        }
    };
    osmpoints
}

async fn read(bbox: &WGS84BoundingBox) -> InputPoints {
    let osmpoints = match cache::read(bbox).await {
        Some(d) => d,
        None => {
            // "could not find any data for {} (download probably failed) => skip",
            InputPoints::new()
        }
    };
    osmpoints
}

async fn reducebbox(bbox: &WGS84BoundingBox, step: &f64) -> Vec<WGS84BoundingBox> {
    let many = split(&bbox, step);
    let mut uncached = Vec::new();
    for (_index, atom) in many {
        if !(cache::hit_cache(&atom).await) {
            uncached.push(atom.clone());
        }
    }
    uncached
}

async fn process(bbox: &WGS84BoundingBox) -> InputPoints {
    let step = 0.05f64;
    let atoms = split(&bbox, &step);
    let not_cached = reducebbox(&bbox, &step).await;
    if !not_cached.is_empty() {
        log::info!("atoms:{}", atoms.len());
        log::info!("not in cache:{}", not_cached.len());
    }
    download_chunk(&not_cached).await;
    let mut ret = Vec::new();
    log::trace!("about to read atoms:{:3}", atoms.len());
    for (_index, atom) in atoms {
        let points = read(&atom).await;
        ret.extend(points.points);
    }
    log::trace!("done");
    InputPoints { points: ret }
}

pub async fn download_for_track(track: &Track) -> InputPoints {
    let bbox = track.wgs84_bounding_box();
    assert!(!bbox.empty());
    let mut ret = process(&bbox).await;
    project::project_on_track::<InputPoint>(track, &mut ret.points);
    // prefiltering is not very good ("Walke")
    ret.points.retain(|w| {
        if w.kind() == InputType::City {
            return true;
        }
        if w.kind() == InputType::Hamlet {
            return false;
        }
        // no filtering on the population, "Walke"
        // if w.kind() == InputType::Village ...
        match w.track_index {
            Some(index) => {
                let p1 = &w.wgs84;
                let p2 = &track.wgs84[index];
                return distance_wgs84(p1, p2) < 1000f64;
            }
            None => {
                return false;
            }
        }
    });

    ret
}
