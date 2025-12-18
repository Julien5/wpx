use std::{cmp::Ordering, collections::BTreeMap};

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

fn floor_snap(x: &f64, step: &f64) -> f64 {
    (x / step).floor() * step
}

fn ceil_snap(x: &f64, step: &f64) -> f64 {
    (x / step).ceil() * step
}

fn snap(bbox: &mut BoundingBox, step: &f64) {
    bbox.set_min(Point2D::new(
        floor_snap(&bbox.get_min().x, step),
        floor_snap(&bbox.get_min().y, step),
    ));
    bbox.set_max(Point2D::new(
        ceil_snap(&bbox.get_max().x, step),
        ceil_snap(&bbox.get_max().y, step),
    ));
}

pub fn pointbox(p: &MercatorPoint) -> EuclideanBoundingBox {
    let step = &BBOXWIDTH;
    let min = Point2D::new(floor_snap(&p.x(), step), floor_snap(&p.y(), step));
    let max = Point2D::new(ceil_snap(&p.x(), step), ceil_snap(&p.y(), step));
    EuclideanBoundingBox::minmax(min, max)
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

pub struct Index {
    index: (usize, usize),
    size: (usize, usize),
}

impl Index {
    fn flat(&self) -> usize {
        let (x, y) = &self.index;
        let (nx, _ny) = &self.size;
        y * nx + x
    }
}

impl PartialEq for Index {
    fn eq(&self, other: &Self) -> bool {
        if self.size.0 != other.size.0 {
            return false;
        }
        if self.size.1 != other.size.1 {
            return false;
        }
        if self.index.0 != other.index.0 {
            return false;
        }
        if self.index.1 != other.index.1 {
            return false;
        }
        true
    }
}

impl Eq for Index {}
impl PartialOrd for Index {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.flat().partial_cmp(&other.flat())
    }
}

impl Ord for Index {
    fn cmp(&self, other: &Self) -> Ordering {
        self.flat().cmp(&other.flat())
    }
}

pub type BoundingBoxes = BTreeMap<Index, BoundingBox>;

pub const BBOXWIDTH: f64 = 10000f64;

pub fn split(orig: &BoundingBox, step: &f64) -> BoundingBoxes {
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
            let index = Index {
                index: (kx, ky),
                size: (nx, ny),
            };
            ret.insert(index, BoundingBox::minmax(min, max));
        }
    }
    ret
}

pub fn bounding_box(boxes: &Vec<BoundingBox>) -> BoundingBox {
    assert!(!boxes.is_empty());
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
        let bboxes = split(&bbox, &0.1f64);
        assert_eq!(bboxes.len(), 60);
    }
}
