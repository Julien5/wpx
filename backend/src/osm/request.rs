use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::{
    bbox::BoundingBox,
    inputpoint::Tags,
    mercator::{EuclideanBoundingBox, MercatorPoint, WebMercatorProjection},
    osm::{
        request_optim::optimize_tiles_into_boxes,
        request_split::{chunk_kinds, filter_string, split_zones, tile_kinds, DensityMap},
        tiles_debug,
    },
    point_collection::Kind,
    tile::*,
    track::WGS84BoundingBox,
    wgs84point::WGS84Point,
};

#[derive(Debug, Clone)]
pub enum Boxes {
    Tiled(Tiles),
    Chunked(Chunks),
}

impl Boxes {
    pub fn new_tiled() -> Self {
        Boxes::Tiled(Tiles::new())
    }
    pub fn new_chunked() -> Self {
        Boxes::Chunked(Chunks::new())
    }
    pub fn len(&self) -> usize {
        match self {
            Boxes::Tiled(tiles) => tiles.len(),
            Boxes::Chunked(chunks) => chunks.len(),
        }
    }

    pub fn from_tiles(tiles: &Tiles) -> Self {
        Boxes::Tiled(tiles.clone())
    }

    pub fn from_chunks(chunks: &Chunks) -> Self {
        Boxes::Chunked(chunks.clone())
    }

    pub fn add_tile(&mut self, tile: &Tile) {
        match self {
            Boxes::Tiled(tiles) => {
                tiles.insert(tile.clone());
            }
            Boxes::Chunked(_) => {
                panic!("unsupported add_tile for Boxes::Chunked")
            }
        }
    }

    pub fn add_chunk(&mut self, chunk: &Chunk) {
        match self {
            Boxes::Tiled(_) => {
                panic!("unsupported add_chunk for Boxes::Tiled")
            }
            Boxes::Chunked(chunks) => {
                chunks.insert(chunk.clone());
            }
        }
    }

    pub fn area(&self) -> f64 {
        match self {
            Boxes::Chunked(chunks) => chunks.iter().map(|c| c.bbox().area()).sum(),
            Boxes::Tiled(tiles) => tiles.iter().map(|t| t.bbox().area()).sum(),
        }
    }

    pub fn bboxes(&self) -> Vec<BoundingBox> {
        match self {
            Boxes::Chunked(chunks) => chunks.iter().map(|c| c.bbox()).collect(),
            Boxes::Tiled(tiles) => tiles.iter().map(|t| t.bbox()).collect(),
        }
    }

    pub fn chunks(&self) -> Chunks {
        match self {
            Boxes::Chunked(chunks) => chunks.clone(),
            Boxes::Tiled(tiles) => chunks(tiles),
        }
    }

    pub fn optimized(&self) -> Vec<EuclideanBoundingBox> {
        match self {
            Boxes::Chunked(chunks) => chunks.iter().map(|c| c.bbox()).collect(),
            Boxes::Tiled(tiles) => optimize_tiles_into_boxes(tiles),
        }
    }
}

#[derive(Debug)]
pub struct Request {
    pub boxes: Vec<Boxes>,
}

pub type Zone = Vec<BoundingBox>;
#[derive(Clone, Default)]
pub struct Zones {
    pub chunks: Zone,
    pub tiles: Zone,
}

impl Zones {
    pub fn new() -> Self {
        Self {
            chunks: Zone::new(),
            tiles: Zone::new(),
        }
    }
}

impl Request {
    pub fn new() -> Self {
        Self { boxes: Vec::new() }
    }
    const BBOX_EPSILON: f64 = 0.00001; // ~1m, well below any OSM feature spacing

    fn osm3(bbox: &WGS84BoundingBox) -> String {
        format!(
            "({:.5},{:.5},{:.5},{:.5})",
            bbox.get_min().y - Self::BBOX_EPSILON,
            bbox.get_min().x - Self::BBOX_EPSILON,
            bbox.get_max().y + Self::BBOX_EPSILON,
            bbox.get_max().x + Self::BBOX_EPSILON
        )
    }

    fn split(&self) -> Vec<Zones> {
        let mut chunks = Vec::new();
        let mut tiles = Vec::new();
        for boxes in &self.boxes {
            let o = boxes.optimized();
            for bbox in &o {
                log::trace!("optim: {:?}", bbox);
            }
            let svg = tiles_debug::paint_svg(&boxes, &o);
            // if debug
            if cfg!(debug_assertions) {
                let _ = tiles_debug::save_debug_svg_incrementally(&svg);
            }
            log::trace!(
                "reduces {} boxes to {} boxes [{:.1}km2 = {:.1}km2]",
                boxes.len(),
                o.len(),
                boxes.area() / 1_000_000f64,
                o.iter().map(|bbox| bbox.area()).sum::<f64>() / 1_000_000f64,
            );
            match boxes {
                Boxes::Tiled(_) => {
                    tiles.extend_from_slice(&o);
                }
                Boxes::Chunked(_) => {
                    chunks.extend_from_slice(&o);
                }
            }
        }
        let zones = Zones { tiles, chunks };
        // feature_count per km2
        let mut density_map = DensityMap::new();
        density_map.insert(Kind::Cities, 50f64 / 10_000f64);
        density_map.insert(Kind::Villages, 100f64 / 10_000f64);
        density_map.insert(Kind::Hamlets, 300f64 / 10_000f64);
        density_map.insert(Kind::Mountains, 100f64 / 10_000f64);
        split_zones(zones, &density_map, 500f64)
    }

    pub fn strings(&self) -> Vec<(Zones, String)> {
        let zone_packets = self.split();
        zone_packets
            .iter()
            .map(|zones| (zones.clone(), Self::string(&zones)))
            .collect()
    }

    pub fn string(zones: &Zones) -> String {
        let projection = WebMercatorProjection::make();
        let mut zone_strings = Vec::new();
        let mut nzones = 0;
        let tiles_boxes = filter_string(&tile_kinds())
            .iter()
            .map(|filter| {
                zones
                    .tiles
                    .iter()
                    .map(|bbox| Self::osm3(&bbox.unproject_with(&projection)))
                    .map(|zone| format!("node{}{};", filter, zone))
                    .collect::<Vec<_>>()
            })
            .flatten()
            .collect::<Vec<_>>();

        log::trace!("add: {} zones for {:?}", tiles_boxes.len(), tile_kinds());
        let chunks_boxes = filter_string(&chunk_kinds())
            .iter()
            .map(|filter| {
                zones
                    .chunks
                    .iter()
                    .map(|bbox| Self::osm3(&bbox.unproject_with(&projection)))
                    .map(|zone| format!("node{}{};", filter, zone))
                    .collect::<Vec<_>>()
            })
            .flatten()
            .collect::<Vec<_>>();

        log::trace!("add: {} zones for {:?}", chunks_boxes.len(), chunk_kinds());
        let all_boxes: Vec<_> = tiles_boxes
            .into_iter()
            .chain(chunks_boxes.into_iter())
            .collect();
        nzones += all_boxes.len();
        let footer = "out tags center".to_string();
        zone_strings.push(format!("(\n{}\n);\n{};", all_boxes.join("\n"), footer));
        log::trace!("total: {} zones", nzones);
        let timeout = 60;
        let header = format!("[out:json][timeout:{}]", timeout);
        format!("{};\n\n{}", header, zone_strings.join("\n"))
    }
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct OSMFeature {
    pub id: String,
    pub wgs84: WGS84Point,
    pub euc: MercatorPoint,
    pub tags: Tags,
}

pub type OSMFeatures = Vec<OSMFeature>;

impl OSMFeature {
    pub fn kind(&self) -> Kind {
        match self.tags.get("mountain_pass") {
            Some(pass) => {
                if pass == "yes" {
                    return Kind::Mountains;
                }
            }
            _ => {}
        }
        match self.tags.get("natural") {
            Some(natural) => {
                if natural == "peak" {
                    return Kind::Mountains;
                }
            }
            _ => {}
        }
        match self.tags.get("place") {
            Some(place) => {
                if place == "city" {
                    return Kind::Cities;
                }
                if place == "town" {
                    return Kind::Cities;
                }
                if place == "village" {
                    return Kind::Villages;
                }
                if place == "hamlet" {
                    return Kind::Hamlets;
                }
            }
            _ => {
                log::error!("no place tag");
            }
        }
        log::error!("tags:{:?}", self.tags);
        debug_assert!(false);
        Kind::Cities
    }
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct Response {
    pub features: BTreeMap<Tile, OSMFeatures>,
}

impl Response {
    pub fn select_chunk(&self, chunk: &Chunk) -> OSMFeatures {
        let mut ret = Vec::new();
        for (tile, features) in &self.features {
            if chunk.contains(tile) {
                for f in features {
                    if f.kind() == Kind::Cities {
                        ret.push(f.clone());
                    }
                }
            }
        }
        ret.sort_by_key(|f| f.id.clone());
        ret
    }
    pub fn select_tile(&self, target: &Tile) -> OSMFeatures {
        let mut ret = Vec::new();
        for (tile, features) in &self.features {
            if tile == target {
                for f in features {
                    if f.kind() != Kind::Cities {
                        ret.push(f.clone());
                    }
                }
            }
        }
        ret.sort_by_key(|f| f.id.clone());
        ret
    }
}

use serde_with::{serde_as, DisplayFromStr};
#[serde_as]
#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct DataPacket {
    #[serde_as(as = "BTreeMap<DisplayFromStr, _>")]
    pub tiles: BTreeMap<Tile, OSMFeatures>,
    #[serde_as(as = "BTreeMap<DisplayFromStr, _>")]
    pub chunks: BTreeMap<Chunk, OSMFeatures>,
}

impl DataPacket {
    pub fn new() -> Self {
        Self {
            tiles: BTreeMap::new(),
            chunks: BTreeMap::new(),
        }
    }
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct ChunkData {
    pub data: DataPacket,
}

impl ChunkData {
    pub fn new() -> Self {
        Self {
            data: DataPacket::new(),
        }
    }
    pub fn from_string(data: &str) -> Result<ChunkData, serde_json::Error> {
        let data: DataPacket = serde_json::from_str(data)?;
        Ok(Self { data })
    }

    pub fn as_string(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(&self.data)
    }
}
