use std::collections::BTreeSet;

use crate::bboxes::{self, chunk_bbox, Chunk};
#[cfg(not(target_arch = "wasm32"))]
use crate::error::GenericResult;
use crate::event::{self, SenderHandlerLock};
use crate::inputpoint::InputPointMap;
use crate::mercator::EuclideanBoundingBox;

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
async fn read_worker(path: &str) -> Option<String> {
    super::indexdb::read(path).await
}

#[cfg(not(target_arch = "wasm32"))]
async fn hit_cache_worker(filename: &String) -> bool {
    super::filesystem::hit_cache(&cache_path(filename))
}

#[cfg(target_arch = "wasm32")]
async fn hit_cache_worker(path: &String) -> bool {
    super::indexdb::hit_cache(&path).await
}

pub async fn hit_cache(chunk: &Chunk) -> bool {
    let filename = chunk.basename();
    return hit_cache_worker(&filename).await;
}

pub async fn read(bbox: &EuclideanBoundingBox) -> GenericResult<InputPointMap> {
    let chunks = bboxes::split_chunks(&bbox);
    let mut ret = InputPointMap::new();
    for mut chunk in chunks {
        let key = chunk.basename();
        let data = read_worker(&key).await.unwrap();
        chunk.load_map(&data);
        for (tile, points) in &chunk.data.map {
            if bbox.contains_other(&tile) {
                ret.insert_points(&tile, &points);
            }
        }
    }
    Ok(ret)
}

pub async fn write(points: &InputPointMap, logger: &SenderHandlerLock) -> GenericResult<()> {
    use bboxes::Chunk;
    let mut chunks = BTreeSet::new();
    for b in points.map.keys() {
        chunks.insert(Chunk::from_boundingbox(&chunk_bbox(b)));
    }
    let index = 0;
    let total = chunks.len();
    for mut chunk in chunks {
        let key = chunk.basename();
        let data = read_worker(&key).await?;
        chunk.load_map(&data);
        for atom in points
            .map
            .keys()
            .filter(|atom| chunk.bbox.contains_other(&atom))
        {
            chunk
                .data
                .map
                .insert(atom.clone(), points.get(&atom).unwrap().clone());
        }
        event::send_worker(logger, &format!("write cache {}/{}", index + 1, total)).await;
        let data = chunk.map_as_string();
        write_worker(&key, data).await;
    }
    Ok(())
}
