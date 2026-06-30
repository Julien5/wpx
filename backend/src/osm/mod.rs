pub mod download;
#[cfg(target_arch = "wasm32")]
pub mod osmpoint;
pub mod request;
pub mod request_cache;
pub mod request_handler;
pub mod request_optim;
pub mod request_parse;
pub mod request_sort;
pub mod request_split;
mod tiles_debug;

use std::collections::BTreeMap;

use tokio_util::sync::CancellationToken;

use crate::backend::SenderHandlerLock;
use crate::error::GenericResult;
use crate::inputpoint::{InputPoint, InputPointData, InputPointMap, OSMData};
use crate::osm::request::{Boxes, OSMFeature, Request};
use crate::osm::request_handler::get_response;
use crate::tile::Tile;
use crate::track::*;
use crate::track_projection::TrackProjections;

pub struct DownloadSideData<'a> {
    pub logger: &'a SenderHandlerLock,
    pub cancel_token: &'a CancellationToken,
}

fn input_point_from_feature(feature: &OSMFeature) -> InputPoint {
    let data = OSMData {
        tags: feature.tags.clone(),
        osmid: feature.id.clone(),
    };
    InputPoint {
        wgs84: feature.wgs84.clone(),
        euclidean: feature.euc.clone(),
        data: InputPointData::OSM(data),
        track_projections: TrackProjections::new(),
        index: None,
    }
}

pub async fn download_for_track(
    track: &Track,
    side: &DownloadSideData<'_>,
    try_download: bool,
) -> GenericResult<InputPointMap> {
    let (tiles, chunks) = track.boxes(0f64, track.total_distance());
    log::trace!("there are {} tiles on the track", tiles.len());
    log::trace!("there are {} chunks on the track", chunks.len());
    let mut boxes = Vec::new();
    boxes.push(Boxes::from_tiles(&tiles));
    boxes.push(Boxes::from_chunks(&chunks));
    let request = Request { boxes };
    match get_response(&request, &side, try_download).await {
        Ok(chunk_data) => {
            let mut map = BTreeMap::new();
            for (tile, tile_features) in &chunk_data.data.tiles {
                for f in tile_features {
                    let i = input_point_from_feature(f);
                    map.entry(tile.clone()).or_insert_with(Vec::new).push(i);
                }
            }
            for (_chunk, chunk_features) in &chunk_data.data.chunks {
                for f in chunk_features {
                    let i = input_point_from_feature(f);
                    map.entry(Tile::for_point(&i.euclidean))
                        .or_insert_with(Vec::new)
                        .push(i);
                }
            }
            Ok(InputPointMap { map })
        }
        Err(e) => {
            log::error!("download error: {:?}", e);
            // panic!("download");
            Err(e.into())
        }
    }
}
