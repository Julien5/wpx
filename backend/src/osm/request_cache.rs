use std::collections::BTreeMap;

use crate::tile::Chunk;
use crate::tile::Chunks;
use crate::tile::Tile;
use crate::tile::Tiles;

use super::cache::read_worker;
use super::cache::write_worker;
use super::request::*;

pub async fn write_cache(req: &Request, response: &Response) {
    let mut cached_chunks = BTreeMap::new();
    // load all necessary chunks from storage
    for req_boxes in &req.boxes {
        for req_chunk in req_boxes.chunks() {
            let filename = req_chunk.basename();
            if cached_chunks.contains_key(&req_chunk) {
                continue;
            }
            match read_worker(&filename).await {
                Ok(bytes) => match ChunkData::from_string(&bytes) {
                    Ok(resp) => {
                        cached_chunks.insert(req_chunk, resp);
                    }
                    Err(e) => {
                        log::error!("could not read cache chunk: {} because {:?}", filename, e);
                        panic!("could not read cache chunk: {} because {:?}", filename, e);
                    }
                },
                Err(_e) => {
                    //log::info!("could not read cache chunk: {} because {:?}", filename, e);
                }
            }
        }
    }

    // fill them with new data
    for req_boxes in &req.boxes {
        match req_boxes {
            Boxes::Tiled(req_tiles) => {
                for req_tile in req_tiles {
                    let cached_chunk = cached_chunks
                        .entry(req_tile.chunk().clone())
                        .or_insert_with(ChunkData::new);
                    let features = response.select_tile(&req_tile);
                    //log::trace!("writing {} features in tile cache", features.len());
                    /*for f in &features {
                        log::trace!("writing:{:?}", f.tags);
                    }*/
                    let e = cached_chunk
                        .data
                        .tiles
                        .entry(req_tile.clone())
                        .or_insert_with(OSMFeatures::new);
                    let oldlen = e.len();
                    e.extend_from_slice(&features);
                    log::trace!(
                        "cache tile {:?} has: {} features | added {} => {}",
                        req_tile,
                        oldlen,
                        features.len(),
                        e.len()
                    );
                }
            }
            Boxes::Chunked(req_chunks) => {
                for req_chunk in req_chunks {
                    let cached_chunk = cached_chunks
                        .entry(req_chunk.clone())
                        .or_insert_with(ChunkData::new);
                    let features = response.select_chunk(&req_chunk);
                    // log::trace!("writing {} features in chunk cache", features.len());
                    let e = cached_chunk
                        .data
                        .chunks
                        .entry(req_chunk.clone())
                        .or_insert_with(OSMFeatures::new);
                    let oldlen = e.len();
                    e.extend_from_slice(&features);
                    log::trace!(
                        "cache chunk {:?} has: {} feature | added {} => {}",
                        req_chunk,
                        oldlen,
                        features.len(),
                        e.len()
                    );
                }
            }
        }
    }

    // write chunks to storage
    for (chunk, chunk_data) in cached_chunks {
        let filename = chunk.basename();
        match chunk_data.as_string() {
            Ok(bytes) => {
                write_worker(&filename, bytes).await;
            }
            Err(e) => {
                log::error!(
                    "could not serialize chunk data for chunk {:?} because {:?}",
                    chunk,
                    e
                );
            }
        }
    }
}

pub async fn read_cache(req: &Request) -> (ChunkData, Request) {
    let mut cached_chunks = BTreeMap::new();
    // load all necessary chunks from storage

    for req_boxes in &req.boxes {
        for req_chunk in req_boxes.chunks() {
            let filename = req_chunk.basename();
            if cached_chunks.contains_key(&req_chunk) {
                continue;
            }
            match read_worker(&filename).await {
                Ok(bytes) => match ChunkData::from_string(&bytes) {
                    Ok(chunk_data) => {
                        cached_chunks.insert(req_chunk, chunk_data);
                    }
                    Err(e) => {
                        log::warn!(
                            "could not parse data for chunk: {} because {:?}",
                            filename,
                            e
                        );
                        log::warn!("this is probably because the format has changed");
                    }
                },
                Err(_e) => {
                    /*log::info!(
                        "could not read cache for chunk: {} because {:?}",
                        filename,
                        e
                    );*/
                }
            }
        }
    }

    let mut missing_tiles = Boxes::new_tiled();
    let mut missing_chunks = Boxes::new_chunked();
    let mut inplace_tiles = BTreeMap::new();
    let mut inplace_chunks = BTreeMap::new();
    for req_boxes in &req.boxes {
        match req_boxes {
            Boxes::Tiled(req_tiles) => {
                let (found, mtiles) = read_cache_tiles(&req_tiles, &mut cached_chunks);
                for tile in mtiles {
                    missing_tiles.add_tile(&tile);
                }
                for (tile, features) in found {
                    log::trace!("found {} features at tile {:?}", features.len(), tile);
                    inplace_tiles.insert(tile, features);
                }
            }
            Boxes::Chunked(req_chunks) => {
                let (found, mchunks) = read_cache_chunks(&req_chunks, &mut cached_chunks);
                for chunk in mchunks {
                    missing_chunks.add_chunk(&chunk);
                }
                for (chunk, features) in found {
                    log::trace!("found {} features at chunk {:?}", features.len(), chunk);
                    inplace_chunks.insert(chunk, features);
                }
            }
        };
        log::trace!(
            "found {} boxes with area:{:.1}km2)",
            req_boxes.len(),
            req_boxes.area() / 1_000_000f64
        );
    }
    log::trace!(
        "request total:{} missing: [{} tiles] [{} chunks]",
        req.boxes.iter().map(|boxes| boxes.len()).sum::<usize>(),
        missing_tiles.len(),
        missing_chunks.len(),
    );

    let data = DataPacket {
        tiles: inplace_tiles,
        chunks: inplace_chunks,
    };

    let mut boxes = Vec::new();
    if missing_tiles.len() > 0 {
        boxes.push(missing_tiles);
    }
    if missing_chunks.len() > 0 {
        boxes.push(missing_chunks);
    }
    (ChunkData { data }, Request { boxes })
}

fn read_cache_tiles(
    req_tiles: &Tiles,
    cached_chunks: &mut BTreeMap<Chunk, ChunkData>,
) -> (BTreeMap<Tile, Vec<OSMFeature>>, Tiles) {
    let mut found = BTreeMap::new();
    let mut missing = Tiles::new();
    for req_tile in req_tiles {
        if let Some(cached_chunk) = cached_chunks.get(&req_tile.chunk()) {
            if let Some(cached_tile_data) = cached_chunk.data.tiles.get(&req_tile) {
                let ret = found.entry(req_tile.clone()).or_insert_with(Vec::new);
                ret.extend_from_slice(&cached_tile_data);
            } else {
                missing.insert(req_tile.clone());
            }
        } else {
            missing.insert(req_tile.clone());
        }
    }
    (found, missing)
}

fn read_cache_chunks(
    req_chunks: &Chunks,
    cached_chunks: &mut BTreeMap<Chunk, ChunkData>,
) -> (BTreeMap<Chunk, OSMFeatures>, Chunks) {
    let mut missing = Chunks::new();
    let mut found = BTreeMap::new();
    for req_chunk in req_chunks {
        if let Some(cached_chunks) = cached_chunks.get(&req_chunk) {
            debug_assert!(cached_chunks.data.chunks.len() <= 1);
            for (chunk, cached_chunk) in &cached_chunks.data.chunks {
                debug_assert!(chunk == req_chunk);
                let ret = found.entry(req_chunk.clone()).or_insert_with(Vec::new);
                ret.extend_from_slice(&cached_chunk);
            }
        } else {
            missing.insert(req_chunk.clone());
        }
    }
    (found, missing)
}
