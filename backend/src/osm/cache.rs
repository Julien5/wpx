use std::collections::BTreeSet;

use crate::bbox::BoundingBox;
use crate::bboxes::{self, chunk_bbox, Chunk};
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

pub fn key(bbox: &EuclideanBoundingBox) -> String {
    format!(
        "{:0.0}+{:0.0}-{:0.0}+{:0.0}",
        bbox.get_min().y,
        bbox.get_min().x,
        bbox.get_max().y,
        bbox.get_max().x
    )
}

pub fn cache_filename(bbox: &EuclideanBoundingBox) -> String {
    format!("{}/{}", key(bbox), "data")
}

fn cache_path(filename: &str) -> String {
    format!("{}/{}", cache_dir(), filename)
}

#[cfg(not(target_arch = "wasm32"))]
async fn write_worker(filename: &str, data: String) {
    super::filesystem::write(&cache_path(filename), data)
}

#[cfg(not(target_arch = "wasm32"))]
async fn read_worker(filename: &str) -> Option<String> {
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

pub async fn hit_cache(bbox: &EuclideanBoundingBox) -> bool {
    let filename = cache_filename(bbox);
    return hit_cache_worker(&filename).await;
}

pub async fn read(bbox: &EuclideanBoundingBox) -> InputPointMap {
    let chunk_bboxes = bboxes::split_chunks(&bbox);
    let mut ret = InputPointMap::new();
    for (_index, chunkbox) in chunk_bboxes.iter().enumerate() {
        let key = Chunk::basename(chunkbox);
        let data = read_worker(&key).await;
        let chunk = make_chunk(chunkbox, data);
        for (tile, points) in chunk.data.map {
            if bbox.contains_other(&tile) {
                ret.insert_points(&tile, &points);
            }
        }
    }
    ret
}

fn make_chunk(bbox: &BoundingBox, data: Option<String>) -> Chunk {
    let mut ret = Chunk::new();
    ret.bbox = bbox.clone();
    match data {
        Some(bytes) => ret.load_map(&bytes),
        None => {}
    }
    ret
}

pub async fn write(points: &InputPointMap, logger: &SenderHandlerLock) {
    use bboxes::Chunk;
    let mut chunk_bboxes: BTreeSet<BoundingBox> = BTreeSet::new();
    for b in points.map.keys() {
        chunk_bboxes.insert(chunk_bbox(b));
    }
    for (index, chunkbox) in chunk_bboxes.iter().enumerate() {
        let key = Chunk::basename(chunkbox);
        let data = read_worker(&key).await;
        let mut chunk = make_chunk(chunkbox, data);
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
        event::send_worker(
            logger,
            &format!("write cache {}/{}", index + 1, chunk_bboxes.len()),
        )
        .await;
        let data = chunk.map_as_string();
        write_worker(&key, data).await;
    }
}
