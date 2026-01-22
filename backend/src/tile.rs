use std::{cmp::Ordering, collections::BTreeSet};

use serde::{Deserialize, Serialize};

use crate::{
    bbox::*, error::GenericResult, inputpoint::InputPointMap, math::Point2D,
    mercator::MercatorPoint,
};

fn floor_snap_index(x: f64, step: f64) -> isize {
    (x / step).floor() as isize
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Tile {
    pub coord: (isize, isize),
}

impl Tile {
    pub fn chunk_coord(&self) -> (isize, isize) {
        let ix = (self.coord.0 as f64 / CHUNKWIDTH as f64).floor() as isize;
        let iy = (self.coord.1 as f64 / CHUNKWIDTH as f64).floor() as isize;
        (ix, iy)
    }

    pub fn for_point(p: &MercatorPoint) -> Tile {
        Tile {
            coord: (
                (p.x() / BBOXWIDTH).floor() as isize,
                (p.y() / BBOXWIDTH).floor() as isize,
            ),
        }
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

    pub fn bbox(&self) -> BoundingBox {
        BoundingBox::minmax(self.min(), self.max())
    }

    pub fn basename(&self) -> String {
        format!("{:03}-{:03}", self.coord.0, self.coord.1)
    }
}

// large, stored in cache
#[derive(Clone)]
pub struct Chunk {
    // many tiles in data
    pub data: InputPointMap,
    pub coord: (isize, isize),
}

impl Chunk {
    pub fn from_coord(coord: &(isize, isize)) -> Self {
        Self {
            data: InputPointMap::new(),
            coord: coord.clone(),
        }
    }
    fn step() -> f64 {
        (CHUNKWIDTH as f64) * BBOXWIDTH
    }
    /*
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
    }*/

    pub fn basename(&self) -> String {
        format!("{:03}-{:03}", self.coord.0, self.coord.1)
    }
    pub fn load_map(&mut self, data: &str) -> GenericResult<()> {
        let data = InputPointMap::from_string(data)?;
        self.data = data;
        Ok(())
    }
    pub fn map_as_string(&self) -> String {
        self.data.as_string().unwrap()
    }
    pub fn contains(&self, tile: &Tile) -> bool {
        tile.chunk_coord() == self.coord
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

pub const BBOXWIDTH: f64 = 10000f64;
pub const CHUNKWIDTH: usize = 10; // number of bbox per chunk (number * number)

fn split_index(bbox: &BoundingBox, step: f64) -> BTreeSet<(isize, isize)> {
    let iminx = floor_snap_index(bbox.get_xmin(), step);
    let iminy = floor_snap_index(bbox.get_ymin(), step);
    let imaxx = floor_snap_index(bbox.get_xmax(), step) + 1;
    let imaxy = floor_snap_index(bbox.get_ymax(), step) + 1;
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

pub fn split_tiles(orig: &BoundingBox) -> Vec<Tile> {
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
