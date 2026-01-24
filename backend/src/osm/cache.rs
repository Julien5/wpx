use std::collections::BTreeSet;

use crate::error::GenericResult;
use crate::event::{self, SenderHandlerLock};
use crate::inputpoint::InputPointMap;
use crate::mercator::EuclideanBoundingBox;
use crate::tile::{self, Chunk, Tile};

#[cfg(test)]
fn cache_dir() -> String {
    "data/ref/cache".to_string()
}

#[cfg(not(test))]
fn cache_dir() -> String {
    let standart_cache_dir = dirs::cache_dir()
        .unwrap()
        .as_os_str()
        .to_str()
        .unwrap()
        .to_string();
    return format!("{}/{}", standart_cache_dir, "WPX");
}

fn cache_path(filename: &str) -> String {
    format!("{}/{}", cache_dir(), filename)
}

#[cfg(not(target_arch = "wasm32"))]
async fn write_worker(filename: &str, data: String) {
    super::filesystem::write(&cache_path(filename), data)
}

#[cfg(not(target_arch = "wasm32"))]
async fn read_worker(filename: &str) -> GenericResult<String> {
    super::filesystem::read(&cache_path(filename))
}

#[cfg(target_arch = "wasm32")]
async fn write_worker(path: &str, data: String) {
    super::indexdb::write(&path, data).await
}

#[cfg(target_arch = "wasm32")]
async fn read_worker(path: &str) -> GenericResult<String> {
    super::indexdb::read(path).await
}

#[cfg(not(target_arch = "wasm32"))]
async fn _hit_cache_worker(filename: &str) -> bool {
    super::filesystem::hit_cache(&cache_path(filename))
}

#[cfg(target_arch = "wasm32")]
async fn _hit_cache_worker(path: &String) -> bool {
    super::indexdb::hit_cache(&path).await
}

async fn _valid_cache(key: &str) -> bool {
    match read_worker(key).await {
        Ok(data) => match InputPointMap::from_string(&data) {
            Ok(_map) => {
                return true;
            }
            _ => {
                log::info!("invalid cache at {}", key);
                return false;
            }
        },
        Err(_) => {
            panic!("this should not happen");
        }
    }
}

async fn _hit_cache(chunk: &Chunk) -> bool {
    let filename = chunk.basename();
    return _hit_cache_worker(&filename).await && _valid_cache(&filename).await;
}

pub type MissingTiles = BTreeSet<Tile>;

pub fn tiles_from_bbox(bbox: &EuclideanBoundingBox) -> MissingTiles {
    let ret: MissingTiles = tile::split_tiles(bbox)
        .iter()
        .map(|tile| tile.clone())
        .collect();
    ret
}

pub async fn read(bbox: &EuclideanBoundingBox) -> GenericResult<(InputPointMap, MissingTiles)> {
    let chunks = tile::split_chunks(&bbox);
    let mut missing: MissingTiles = tiles_from_bbox(bbox);
    let mut good = InputPointMap::new();
    for mut chunk in chunks {
        let key = chunk.basename();
        match read_worker(&key).await {
            Ok(bytes) => match chunk.load_map(&bytes) {
                Ok(()) => {
                    for (tile, points) in &chunk.data.map {
                        if bbox.overlap(&tile.bbox()) {
                            missing.remove(&tile);
                            good.insert_points(tile, &points);
                        }
                    }
                    for tile in &missing {
                        if tile.chunk_coord() == chunk.coord {
                            log::warn!(
                                "could not find data for tile {} in chunk {}",
                                tile.basename(),
                                chunk.basename()
                            );
                        }
                    }
                }
                Err(e) => {
                    log::info!(
                        "could not load map for chunk: {} because {:?}",
                        chunk.basename(),
                        e
                    );
                    log::info!("this is probably because the format has changed");
                }
            },
            Err(e) => {
                log::info!("failed to read cache at {}", key);
                log::info!("because: {:?}", e);
            }
        }
    }
    Ok((good, missing))
}

pub async fn write(points: &InputPointMap, logger: &SenderHandlerLock) -> GenericResult<()> {
    use tile::Chunk;
    let mut chunks = BTreeSet::new();
    for tile in points.map.keys() {
        chunks.insert(Chunk::from_coord(&tile::chunk_coord(tile)));
    }
    let mut index = 0;
    let total = chunks.len();
    for mut chunk in chunks {
        let key = chunk.basename();
        match read_worker(&key).await {
            Ok(bytes) => match chunk.load_map(&bytes) {
                Ok(()) => {}
                Err(e) => {
                    log::warn!(
                        "cannot load map for {} because {:?} => ignore",
                        chunk.basename(),
                        e
                    );
                }
            },
            Err(_) => {}
        }
        // we have a n^2 here.
        let ltiles: Vec<_> = points
            .map
            .keys()
            .filter(|tile| chunk.contains(&tile))
            .collect();
        for tile in ltiles {
            let tpoints = points.get(&tile).unwrap();
            log::trace!(
                "tile {} chunk {} npoints {}",
                tile.basename(),
                chunk.basename(),
                tpoints.len()
            );
            chunk.data.map.insert(tile.clone(), tpoints.clone());
        }
        event::send_worker(logger, &format!("write cache {}/{}", index + 1, total));
        let data = chunk.map_as_string();
        log::debug!("rewrite chunk {}", chunk.basename());
        write_worker(&key, data).await;
        index += 1;
    }
    Ok(())
}
