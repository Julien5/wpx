mod cache;
mod download;
mod filesystem;
#[cfg(target_arch = "wasm32")]
mod indexdb;
pub mod osmpoint;

use crate::event::SenderHandlerLock;
use crate::inputpoint::{InputPointMap, InputPoints};
use crate::mercator::EuclideanBoundingBox;
use crate::track::*;
use crate::{bboxes::*, event};

fn osm3(bbox: &WGS84BoundingBox) -> String {
    format!(
        "({:0.3},{:0.3},{:0.3},{:0.3})",
        bbox.get_min().y,
        bbox.get_min().x,
        bbox.get_max().y,
        bbox.get_max().x
    )
}

async fn download_chunk_real(
    bbox: &WGS84BoundingBox,
    logger: &SenderHandlerLock,
) -> std::result::Result<InputPoints, std::io::Error> {
    use download::*;
    let bboxparam = osm3(&bbox);
    let result = parse_osm_content(all(&bboxparam, logger).await.unwrap().as_bytes());
    match result {
        Ok(points) => Ok(points),
        Err(e) => {
            log::info!("could not download(ignore)");
            log::info!("reason: {}", e.to_string());
            Err(std::io::Error::new(std::io::ErrorKind::InvalidData, "data"))
        }
    }
}

async fn download_chunk(
    bboxes: &Vec<EuclideanBoundingBox>,
    logger: &SenderHandlerLock,
) -> InputPoints {
    if bboxes.is_empty() {
        return InputPoints::new();
    }
    let eucbbox = bounding_box(bboxes);
    let wgsbbox = eucbbox.unproject();

    log::info!("downloading for {} tiles", bboxes.len());
    let osmpoints = match download_chunk_real(&wgsbbox, logger).await {
        Ok(points) => {
            log::info!("downloaded {:3} points", points.points.len());
            cache::write(bboxes, &points, logger).await;
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

async fn read(bbox: &EuclideanBoundingBox) -> InputPoints {
    let osmpoints = match cache::read(bbox).await {
        Some(d) => d,
        None => {
            // "could not find any data for {} (download probably failed) => skip",
            InputPoints::new()
        }
    };
    osmpoints
}

async fn remove_cache(tiles: &BoundingBoxes) -> Vec<EuclideanBoundingBox> {
    let mut uncached = Vec::new();
    for (_index, tile) in tiles {
        if !(cache::hit_cache(&tile).await) {
            uncached.push(tile.clone());
        }
    }
    uncached
}

async fn process(bbox: &EuclideanBoundingBox, logger: &SenderHandlerLock) -> InputPointMap {
    let tiles = split(&bbox, &BBOXWIDTH);
    let not_cached = remove_cache(&tiles).await;
    if !not_cached.is_empty() {
        log::info!(
            "there are {} tiles, {} not in cache",
            tiles.len(),
            not_cached.len()
        );
    }
    download_chunk(&not_cached, logger).await;
    let mut ret = InputPointMap::new();
    for (_index, tile) in tiles {
        // event::send_worker(logger, &format!("read {}/{}", count, total)).await;
        let points = read(&tile).await;
        ret.insert_points(&tile, &points.points);
    }
    ret
}

pub async fn download_for_track(track: &Track, logger: &SenderHandlerLock) -> InputPointMap {
    let bbox = track.euclidean_bounding_box();
    assert!(!bbox.empty());
    event::send_worker(logger, &format!("{}", "download")).await;
    let ret = process(&bbox, logger).await;
    ret
}
