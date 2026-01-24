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
use crate::osm::cache::{tiles_from_bbox, MissingTiles};
use crate::track::*;
use crate::{event, tile::*};

// 5 digits are enough for 1-meter precision.
fn osm3(bbox: &WGS84BoundingBox) -> String {
    format!(
        "({:.5},{:.5},{:.5},{:.5})",
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
            // std::fs::write("/tmp/dl.txt", &content).unwrap();
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

async fn download_tiles(
    tiles: &MissingTiles,
    logger: &SenderHandlerLock,
) -> GenericResult<InputPoints> {
    if tiles.is_empty() {
        return Ok(InputPoints::new());
    }
    let eucbbox = bounding_box(tiles);
    let wgsbbox = eucbbox.unproject();

    log::info!("downloading for {} tiles", tiles.len());
    let mut empty_tiles = 0;
    match download_chunk_real(&wgsbbox, logger).await {
        Ok(points) => {
            log::info!("downloaded {:3} points", points.points.len());
            let mut map = InputPointMap::from_vector(&points.points);
            // tiles where there is no data must also have an entry in the
            // map
            for tile in tiles {
                if !map.map.contains_key(&tile) {
                    log::info!("insert empty tile {}", tile.basename());
                    empty_tiles += 1;
                    map.map.insert(tile.clone(), Vec::new());
                }
            }
            log::info!("inserted {} empty tiles", empty_tiles);
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

async fn read(bbox: &EuclideanBoundingBox) -> GenericResult<(InputPointMap, MissingTiles)> {
    cache::read(bbox).await
}

#[allow(dead_code)]
fn print_missing(missing: &MissingTiles) {
    let mchunks = split_chunks(&bounding_box(missing));
    log::trace!(" {} tiles missing", missing.len());
    for mchunk in mchunks {
        log::trace!("missing in chunk {}", mchunk.basename());
        for tile in missing {
            if mchunk.contains(&tile) {
                log::trace!(
                    "missing in chunk {}: tile:{} BBOX:{:?}",
                    mchunk.basename(),
                    tile.basename(),
                    tile.bbox().unproject()
                );
            }
        }
    }
}

async fn process(
    bbox: &EuclideanBoundingBox,
    logger: &SenderHandlerLock,
) -> GenericResult<InputPointMap> {
    let mut found = InputPointMap::new();
    let mut missing = tiles_from_bbox(bbox);
    event::send_worker(logger, &format!("{}", "read from cache"));
    match read(bbox).await {
        Ok((local_found, local_missing)) => {
            found = local_found;
            missing = local_missing;
            log::info!(
                "cache is valid: {} tiles found, but {} tiles missing",
                found.map.len(),
                missing.len()
            );
            //print_missing(&missing);
            if missing.is_empty() {
                log::info!("the cache is complete.");
                return Ok(found);
            }
        }
        Err(e) => {
            log::info!("error: {}", e);
            log::info!("the cache could not be read => update");
        }
    }

    event::send_worker(logger, &format!("{}", "download"));
    // download and write in cache
    download_tiles(&missing, logger).await?;

    event::send_worker(logger, &format!("{}", "read from cache"));
    match read(bbox).await {
        Ok((map, missing)) => {
            log::info!("found: {} missing: {}", map.map.len(), missing.len());
            if missing.is_empty() {
                log::info!("the cache is complete with {} tiles", found.map.len());
                return Ok(map);
            } else {
                log::error!("the cache has still {} missing tiles", missing.len());
                //print_missing(&missing);
                return Err(GenericError::from(
                    "still missing data in cache".to_string(),
                ));
            }
        }
        Err(e) => {
            log::error!("error: {}", e);
            log::error!("the cache could not be read and could not be updated");
            return Err(e);
        }
    }
}

pub async fn download_for_track(
    track: &Track,
    logger: &SenderHandlerLock,
) -> GenericResult<InputPointMap> {
    let bbox = track.euclidean_bounding_box();
    assert!(!bbox.empty());
    let ret = process(&bbox, logger).await;
    ret
}
