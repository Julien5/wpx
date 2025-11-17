use crate::bbox::BoundingBox;
use crate::math::Point2D;
use crate::mercator::MercatorPoint;
use crate::track::Track;
use crate::{inputpoint::*, math, mercator};
use rstar::{RTree, AABB};

#[derive(Clone, PartialEq)]
pub struct IndexedPoint {
    pub coord: mercator::MercatorPoint,
    pub index: usize,
}

impl std::fmt::Debug for IndexedPoint {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("IndexWGS84Point")
            .field("x", &self.coord.0)
            .field("y", &self.coord.1)
            .field("index", &self.index)
            .finish()
    }
}

/*impl rstar::Point for IndexedWGS84Point {
    type Scalar = f64;
    const DIMENSIONS: usize = 2;

    fn generate(mut generator: impl FnMut(usize) -> Self::Scalar) -> Self {
        IndexedWGS84Point {
            wgs84: WGS84Point::new(&generator(0), &generator(1), &0f64),
            index: usize::MAX,
        }
    }

    fn nth(&self, index: usize) -> Self::Scalar {
        match index {
            0 => self.wgs84.longitude(),
            1 => self.wgs84.latitude(),
            _ => unreachable!(),
        }
    }

    fn nth_mut(&mut self, index: usize) -> &mut Self::Scalar {
        self.wgs84.nth_mut(index)
    }
}
*/

impl rstar::RTreeObject for IndexedPoint {
    type Envelope = AABB<[f64; 2]>;
    //type Envelope = AABB<WGS84Point>;

    fn envelope(&self) -> Self::Envelope {
        AABB::from_point([self.coord.0, self.coord.1])
        //AABB::from_point(self.wgs84)
    }
}

impl rstar::PointDistance for IndexedPoint {
    fn distance_2(&self, point: &[f64; 2]) -> f64 {
        let p1 = &self.coord;
        let p2 = Point2D::new(point[0], point[1]);
        math::distance2(&p1.point2d(), &p2)
    }

    fn contains_point(&self, _point: &[f64; 2]) -> bool {
        false
    }
}

fn indexed_points(points: &Vec<InputPoint>) -> Vec<IndexedPoint> {
    let mut ret = Vec::new();
    for k in 0..points.len() {
        ret.push(IndexedPoint {
            coord: points[k].euclidean.clone(),
            index: k,
        });
    }
    ret
}

fn indexed_track(points: &Track, range: &std::ops::Range<usize>) -> Vec<IndexedPoint> {
    let mut ret = Vec::new();
    for k in range.start..range.end {
        ret.push(IndexedPoint {
            coord: points.euclidian[k].clone(),
            index: k,
        });
    }
    ret
}

#[derive(Clone)]
pub struct IndexedPointsTree {
    tree: RTree<IndexedPoint>,
}

fn coord(point: &MercatorPoint) -> [f64; 2] {
    [point.x(), point.y()]
}

impl IndexedPointsTree {
    pub fn from_points(points: &Vec<InputPoint>) -> IndexedPointsTree {
        let ipoints = indexed_points(points);
        let tree = RTree::bulk_load(ipoints);
        IndexedPointsTree { tree }
    }
    pub fn from_track(track: &Track, range: &std::ops::Range<usize>) -> IndexedPointsTree {
        let ipoints = indexed_track(track, range);
        let tree = RTree::bulk_load(ipoints);
        IndexedPointsTree { tree }
    }
    pub fn nearest_neighbor(&self, point: &MercatorPoint) -> Option<usize> {
        let nearest = self.tree.nearest_neighbor(&coord(point));
        match nearest {
            Some(indexed) => Some(indexed.index),
            None => None,
        }
    }
    pub fn find_points_in_bbox(&self, bbox: &BoundingBox) -> Vec<usize> {
        let mut ret = Vec::new();
        let min = coord(&MercatorPoint::from_point2d(&bbox.get_min()));
        let max = coord(&MercatorPoint::from_point2d(&bbox.get_max()));
        let aabb = AABB::from_corners(min, max);
        for p in self.tree.locate_in_envelope(&aabb) {
            ret.push(p.index);
        }
        ret
    }
}
