use std::{cmp::Ordering, collections::BTreeSet};

use serde::{Deserialize, Serialize};

use crate::{
    bbox::*,
    math::Point2D,
    mercator::{EuclideanBoundingBox, MercatorPoint},
};

fn floor_snap_index(x: f64, step: f64) -> isize {
    (x / step).floor() as isize
}

fn ceil_snap_index(x: f64, step: f64) -> isize {
    (x / step).ceil() as isize
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Tile {
    pub coord: (isize, isize),
}

pub fn tile_to_point(x: isize, y: isize) -> Point2D {
    Point2D {
        x: x as f64 * BBOXWIDTH,
        y: y as f64 * BBOXWIDTH,
    }
}

impl Tile {
    pub fn chunk_coord(&self) -> (isize, isize) {
        let ix = (self.coord.0 as f64 / CHUNKWIDTH as f64).floor() as isize;
        let iy = (self.coord.1 as f64 / CHUNKWIDTH as f64).floor() as isize;
        (ix, iy)
    }

    pub fn chunk(&self) -> Chunk {
        let coord = self.chunk_coord();
        Chunk::from_coord(&coord)
    }

    pub fn for_point(p: &MercatorPoint) -> Self {
        Self {
            coord: (
                (p.x() / Self::step()).floor() as isize,
                (p.y() / Self::step()).floor() as isize,
            ),
        }
    }
    fn step() -> f64 {
        BBOXWIDTH
    }

    fn min(&self) -> Point2D {
        Point2D::new(
            self.coord.0 as f64 * BBOXWIDTH,
            self.coord.1 as f64 * BBOXWIDTH,
        )
    }
    fn max(&self) -> Point2D {
        Point2D::new(
            (self.coord.0 + 1) as f64 * BBOXWIDTH,
            (self.coord.1 + 1) as f64 * BBOXWIDTH,
        )
    }

    pub fn bbox(&self) -> EuclideanBoundingBox {
        EuclideanBoundingBox::minmax(self.min(), self.max())
    }

    pub fn basename(&self) -> String {
        format!("{:03}-{:03}", self.coord.0, self.coord.1)
    }
}

impl std::fmt::Display for Tile {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{},{}", self.coord.0, self.coord.1)
    }
}

impl std::fmt::Display for Chunk {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{},{}", self.coord.0, self.coord.1)
    }
}

impl std::str::FromStr for Tile {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (x, y) = s.split_once(',').ok_or("missing comma")?;
        Ok(Tile {
            coord: (
                x.trim().parse().map_err(|e| format!("{e}"))?,
                y.trim().parse().map_err(|e| format!("{e}"))?,
            ),
        })
    }
}

impl std::str::FromStr for Chunk {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (x, y) = s.split_once(',').ok_or("missing comma")?;
        Ok(Chunk {
            coord: (
                x.trim().parse().map_err(|e| format!("{e}"))?,
                y.trim().parse().map_err(|e| format!("{e}"))?,
            ),
        })
    }
}

// large, stored in cache
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Chunk {
    pub coord: (isize, isize),
}

impl Chunk {
    pub fn from_coord(coord: &(isize, isize)) -> Self {
        Self {
            coord: coord.clone(),
        }
    }
    pub fn for_point(p: &MercatorPoint) -> Self {
        Self {
            coord: (
                (p.x() / Self::step()).floor() as isize,
                (p.y() / Self::step()).floor() as isize,
            ),
        }
    }
    fn step() -> f64 {
        (CHUNKWIDTH as f64) * BBOXWIDTH
    }

    fn min(&self) -> Point2D {
        Point2D::new(
            self.coord.0 as f64 * Self::step(),
            self.coord.1 as f64 * Self::step(),
        )
    }
    fn max(&self) -> Point2D {
        Point2D::new(
            (self.coord.0 + 1) as f64 * Self::step(),
            (self.coord.1 + 1) as f64 * Self::step(),
        )
    }
    pub fn bbox(&self) -> BoundingBox {
        BoundingBox::minmax(self.min(), self.max())
    }

    pub fn basename(&self) -> String {
        let x = if self.coord.0 < 0 {
            format!("W{:03}", -self.coord.0)
        } else {
            format!("E{:03}", self.coord.0)
        };
        let y = if self.coord.1 < 0 {
            format!("S{:03}", -self.coord.1)
        } else {
            format!("N{:03}", self.coord.1)
        };
        format!("{}-{}", x, y)
    }
    pub fn contains(&self, tile: &Tile) -> bool {
        tile.chunk_coord() == self.coord
    }
    pub fn tiles(&self) -> Tiles {
        split_tiles(&self.bbox())
    }
}

impl PartialEq for Chunk {
    fn eq(&self, other: &Self) -> bool {
        self.coord == other.coord
    }
}

impl Eq for Chunk {}

impl PartialOrd for Chunk {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Chunk {
    fn cmp(&self, other: &Self) -> Ordering {
        self.coord.cmp(&other.coord)
    }
}

pub fn chunk_coord(b: &Tile) -> (isize, isize) {
    (
        (b.coord.0 as f64 / CHUNKWIDTH as f64).floor() as isize,
        (b.coord.1 as f64 / CHUNKWIDTH as f64).floor() as isize,
    )
}

fn tile(tile: &Tile, dx: isize, dy: isize) -> Tile {
    Tile {
        coord: (tile.coord.0 + dx, tile.coord.1 + dy),
    }
}

pub fn neighbors(middle: &Tile) -> [Tile; 8] {
    let step = 1;
    [
        tile(middle, -step, -step),
        tile(middle, 0, -step),
        tile(middle, step, -step),
        tile(middle, -step, 0),
        tile(middle, step, 0),
        tile(middle, -step, step),
        tile(middle, 0, step),
        tile(middle, step, step),
    ]
}

pub type Tiles = BTreeSet<Tile>;
pub type Chunks = BTreeSet<Chunk>;

pub const BBOXWIDTH: f64 = 10000f64;
pub const CHUNKWIDTH: usize = 10; // number of bbox per chunk (number * number)

fn snap_max(x: f64, step: f64) -> isize {
    return ceil_snap_index(x, step);
}

fn snap_min(x: f64, step: f64) -> isize {
    return floor_snap_index(x, step);
}

fn split_index(bbox: &BoundingBox, step: f64) -> BTreeSet<(isize, isize)> {
    let iminx = snap_min(bbox.get_xmin(), step);
    let iminy = snap_min(bbox.get_ymin(), step);
    let imaxx = snap_max(bbox.get_xmax(), step);
    let imaxy = snap_max(bbox.get_ymax(), step);
    let mut ret = BTreeSet::new();
    for x in iminx..imaxx {
        for y in iminy..imaxy {
            ret.insert((x, y));
        }
    }
    ret
}

pub fn split_chunks(orig: &BoundingBox) -> Vec<Chunk> {
    split_index(orig, Chunk::step())
        .iter()
        .map(|coord| Chunk::from_coord(coord))
        .collect()
}

pub fn chunks(tiles: &Tiles) -> Chunks {
    tiles
        .iter()
        .map(|tile| Chunk::from_coord(&tile.chunk_coord()))
        .collect()
}

pub fn tiles_in_chunk(tiles: &Tiles, chunk: &Chunk) -> Tiles {
    let a = chunk.tiles();
    a.intersection(&tiles).map(|t| t.clone()).collect()
}

pub fn split_tiles_vector(orig: &BoundingBox) -> Vec<Tile> {
    split_index(orig, BBOXWIDTH)
        .iter()
        .map(|coord| Tile {
            coord: coord.clone(),
        })
        .collect()
}

pub fn split_tiles(orig: &BoundingBox) -> Tiles {
    split_index(orig, BBOXWIDTH)
        .iter()
        .map(|coord| Tile {
            coord: coord.clone(),
        })
        .collect()
}

pub fn bounding_box<'a, I>(tiles: I) -> BoundingBox
where
    I: IntoIterator<Item = &'a Tile>,
{
    let mut ret = BoundingBox::new();
    for tile in tiles {
        ret.update(&tile.bbox().get_min());
        ret.update(&tile.bbox().get_max());
    }
    ret
}

pub fn bounding_box_chunks<'a, I>(chunks: I) -> BoundingBox
where
    I: IntoIterator<Item = &'a Chunk>,
{
    let mut ret = BoundingBox::new();
    for chunk in chunks {
        ret.update(&chunk.bbox().get_min());
        ret.update(&chunk.bbox().get_max());
    }
    ret
}

pub fn bounding_tiles(tiles: &Tiles) -> Tiles {
    if tiles.is_empty() {
        return Tiles::new();
    }

    let mut min_x = isize::MAX;
    let mut min_y = isize::MAX;
    let mut max_x = isize::MIN;
    let mut max_y = isize::MIN;

    for tile in tiles {
        let (x, y) = tile.coord;
        if x < min_x {
            min_x = x;
        }
        if x > max_x {
            max_x = x;
        }
        if y < min_y {
            min_y = y;
        }
        if y > max_y {
            max_y = y;
        }
    }
    let mut ret = Tiles::new();
    for x in min_x..=max_x {
        for y in min_y..=max_y {
            ret.insert(Tile { coord: (x, y) });
        }
    }
    ret
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;

    use crate::{
        mercator::WebMercatorProjection,
        tile::{bounding_box, bounding_tiles, split_tiles, Chunk, Tile},
        wgs84point::WGS84Point,
    };

    #[test]
    fn test_tile_bbox() {
        let _ = env_logger::try_init();
        let a_wgs = WGS84Point::new(&9.753282, &47.9227, &0.0);
        let projection = WebMercatorProjection::make();
        let a_euc = projection.project(&a_wgs);

        let tile = Tile::for_point(&a_euc);
        let tilesb = split_tiles(&tile.bbox());
        let tiles = BTreeSet::from([tile.clone()]);
        let bounding_tiles = bounding_tiles(&tiles);
        log::trace!("0:{:?}", tile.bbox());
        log::trace!("1:{:?}", bounding_box(&bounding_tiles));
        log::trace!("2:{:?}", bounding_box(&tiles));
        log::trace!("3:{:?}", bounding_box(&tilesb));

        let chunk = Chunk::from_coord(&tile.chunk_coord());
        let chunk_box = chunk.bbox();
        let chunk_tiles = split_tiles(&chunk_box);
        let chunk_box2 = bounding_box(&chunk_tiles);
        log::trace!("chunk_box:{}x{}", chunk_box.width(), chunk_box.height());
        log::trace!("ntiles:{:?}", chunk_tiles.len());
        log::trace!("4:{:?}", chunk_box);
        log::trace!("5:{:?}", chunk_box2);
        assert_eq!(chunk_box, chunk_box2);
    }
}
