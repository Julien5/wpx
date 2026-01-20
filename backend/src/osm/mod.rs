mod cache;
mod download;
mod filesystem;
#[cfg(target_arch = "wasm32")]
mod indexdb;
pub mod osmpoint;

use crate::error::{GenericError, GenericResult};
use crate::event::SenderHandlerLock;
use crate::inputpoint::{InputPointMap, InputPoints};
use crate::mercator::EuclideanBoundingBox;
use crate::{bboxes, track::*};
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
    let dl_result = all(&bboxparam, logger).await;
    match dl_result {
        Err(e) => {
            log::error!("download failed, error = {:?}", e);
            Err(std::io::Error::new(
                std::io::ErrorKind::ConnectionRefused,
                e.to_string(),
            ))
        }
        Ok(content) => {
            let result = parse_osm_content(content.as_bytes());
            match result {
                Ok(points) => Ok(points),
                Err(e) => {
                    log::error!("cannot read OSM data: len:{}", content.len());
                    let mut short = content.clone();
                    short.truncate(1000);
                    log::error!("cannot read OSM data: {}", short);
                    log::error!("reason: {}", e.to_string());
                    Err(std::io::Error::new(std::io::ErrorKind::InvalidData, "data"))
                }
            }
        }
    }
}

async fn download_chunk(
    bboxes: &Vec<EuclideanBoundingBox>,
    logger: &SenderHandlerLock,
) -> GenericResult<InputPoints> {
    if bboxes.is_empty() {
        return Ok(InputPoints::new());
    }
    let eucbbox = bounding_box(bboxes);
    let wgsbbox = eucbbox.unproject();

    log::info!("downloading for {} tiles", bboxes.len());
    match download_chunk_real(&wgsbbox, logger).await {
        Ok(points) => {
            log::info!("downloaded {:3} points", points.points.len());
            let map = InputPointMap::from_vector(&points.points);
            cache::write(&map, logger).await?;
            Ok(points)
        }
        Err(e) => {
            log::info!("error downloading: {:?}", e);
            log::info!("assuming there is nothing");
            return Err(GenericError::from(e));
        }
    }
}

async fn read(bbox: &EuclideanBoundingBox) -> GenericResult<InputPointMap> {
    cache::read(bbox).await
}

async fn remove_cache(tiles: &BoundingBoxes) -> Vec<EuclideanBoundingBox> {
    let mut uncached = Vec::new();

    for tile in tiles {
        let chunk_bboxes = bboxes::split_chunks(&tile);
        assert!(chunk_bboxes.len() == 1);
        let chunk = chunk_bboxes.first().unwrap();
        if !(cache::hit_cache(&chunk).await) {
            uncached.push(tile.clone());
        }
    }
    log::trace!("not in  cache: {}", uncached.len());
    uncached
}

async fn process(
    bbox: &EuclideanBoundingBox,
    logger: &SenderHandlerLock,
) -> GenericResult<InputPointMap> {
    let tiles = split(&bbox, BBOXWIDTH);
    let not_cached = remove_cache(&tiles).await;
    if !not_cached.is_empty() {
        log::info!(
            "there are {} tiles, {} not in cache",
            tiles.len(),
            not_cached.len()
        );
    }
    // we should probe the cache if there is something to read
    // or version the cache.
    download_chunk(&not_cached, logger).await?;
    read(bbox).await
}

pub async fn download_for_track(
    track: &Track,
    logger: &SenderHandlerLock,
) -> GenericResult<InputPointMap> {
    let bbox = track.euclidean_bounding_box();
    assert!(!bbox.empty());
    event::send_worker(logger, &format!("{}", "download")).await;
    let ret = process(&bbox, logger).await;
    ret
}
