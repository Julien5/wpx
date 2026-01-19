use serde::{Deserialize, Serialize};
use serde_json::json;
use std::{cmp::Ordering, collections::BTreeSet};

use crate::{
    bbox::*,
    math::Point2D,
    mercator::{EuclideanBoundingBox, MercatorPoint},
};

/*
pub fn _enlarge(bbox: &mut BoundingBox, epsilon: &f64) {
    bbox._min = (bbox._min.0 - epsilon, bbox._min.y - epsilon);
    bbox._max = (bbox._max.0 + epsilon, bbox._max.y + epsilon);
}
*/

fn floor_snap(x: f64, step: f64) -> f64 {
    (x / step).floor() * step
}

fn ceil_snap(x: f64, step: f64) -> f64 {
    (x / step).ceil() * step
}

fn snap(bbox: &mut BoundingBox, step: f64) {
    bbox.set_min(Point2D::new(
        floor_snap(bbox.get_min().x, step),
        floor_snap(bbox.get_min().y, step),
    ));
    bbox.set_max(Point2D::new(
        ceil_snap(bbox.get_max().x, step),
        ceil_snap(bbox.get_max().y, step),
    ));
}

pub fn pointbox(p: &MercatorPoint) -> EuclideanBoundingBox {
    let step = BBOXWIDTH;
    let min = Point2D::new(floor_snap(p.x(), step), floor_snap(p.y(), step));
    let max = Point2D::new(ceil_snap(p.x(), step), ceil_snap(p.y(), step));
    EuclideanBoundingBox::minmax(min, max)
}

#[derive(Deserialize, Serialize, PartialEq, Clone)]
pub struct Chunk {
    pub bbox: BoundingBox,
}

impl Chunk {
    pub fn new() -> Self {
        Self {
            bbox: BoundingBox::new(),
        }
    }
    fn step() -> f64 {
        (CHUNKWIDTH as f64) * BBOXWIDTH
    }
    fn xy(&self) -> (i32, i32) {
        let rx = self.bbox.get_xmin() / Self::step();
        let ry = self.bbox.get_ymin() / Self::step();
        assert!((rx - rx.round()).abs() < 0.0001);
        assert!((ry - ry.round()).abs() < 0.0001);
        return (rx.round() as i32, ry.round() as i32);
    }
    pub fn basename(&self) -> String {
        let coord = self.xy();
        format!("{}-{}", coord.0, coord.1)
    }
    pub fn from_string(data: &String) -> Chunk {
        match serde_json::from_str(data.as_str()) {
            Ok(points) => points,
            Err(e) => {
                log::error!("could not read osmpoints from: {}", data);
                log::error!("because: {:?}", e);
                Chunk::new()
            }
        }
    }
    pub fn as_string(&self) -> String {
        json!(self).to_string()
    }
}

impl PartialOrd for Chunk {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.bbox.partial_cmp(&other.bbox)
    }
}

impl Ord for Chunk {
    fn cmp(&self, other: &Self) -> Ordering {
        self.bbox.cmp(&other.bbox)
    }
}

impl Eq for Chunk {}

pub fn chunk(b: &BoundingBox) -> Chunk {
    let step = (CHUNKWIDTH as f64) * BBOXWIDTH;
    let min = Point2D::new(
        floor_snap(b.get_xmin(), step),
        floor_snap(b.get_ymin(), step),
    );
    let max = Point2D::new(ceil_snap(b.get_xmax(), step), ceil_snap(b.get_ymax(), step));
    let bbox = EuclideanBoundingBox::minmax(min, max);
    assert_eq!(bbox.width(), step);
    assert_eq!(bbox.height(), step);
    Chunk { bbox }
}

pub fn neighbors(middle: &EuclideanBoundingBox) -> [EuclideanBoundingBox; 8] {
    let step = BBOXWIDTH;
    [
        middle.make_translate(&Point2D::new(-step, -step)),
        middle.make_translate(&Point2D::new(0f64, -step)),
        middle.make_translate(&Point2D::new(step, -step)),
        middle.make_translate(&Point2D::new(-step, 0f64)),
        middle.make_translate(&Point2D::new(step, 0f64)),
        middle.make_translate(&Point2D::new(-step, step)),
        middle.make_translate(&Point2D::new(0f64, step)),
        middle.make_translate(&Point2D::new(step, step)),
    ]
}

pub type BoundingBoxes = BTreeSet<BoundingBox>;

pub const BBOXWIDTH: f64 = 10000f64;
pub const CHUNKWIDTH: usize = 10; // number of bbox per chunk (number * number)

pub fn split(orig: &BoundingBox, step: f64) -> BoundingBoxes {
    let mut bbox = orig.clone();
    snap(&mut bbox, step);
    let nx = (bbox.width() / step).ceil() as usize;
    let ny = (bbox.height() / step).ceil() as usize;
    let min0 = bbox.get_min();
    let mut ret = BoundingBoxes::new();
    for kx in 0..nx {
        for ky in 0..ny {
            let min = Point2D::new(min0.x + (kx as f64) * step, min0.y + (ky as f64) * step);
            let max = Point2D::new(min.x + step, min.y + step);
            ret.insert(BoundingBox::minmax(min, max));
        }
    }
    ret
}

pub fn bounding_box<'a, I>(boxes: I) -> BoundingBox
where
    I: IntoIterator<Item = &'a BoundingBox>,
{
    let mut ret = BoundingBox::new();
    for b in boxes {
        ret.update(&b.get_min());
        ret.update(&b.get_max());
    }
    ret
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn snaptest() {
        let mut backend = crate::backend::Backend::make();
        backend
            .load_filename("data/blackforest.gpx")
            .await
            .expect("fail");
        let bbox = backend.d().track.wgs84_bounding_box();
        let bboxes = split(&bbox, 0.1f64);
        assert_eq!(bboxes.len(), 60);
    }
}
