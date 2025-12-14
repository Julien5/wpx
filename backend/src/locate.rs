use crate::bbox::BoundingBox;
use crate::math::Point2D;
use crate::mercator::MercatorPoint;
use crate::track::{self, Track};
use crate::{inputpoint::*, math, mercator};
use geo::LineLocatePoint;
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

fn middle_point(a: &(f64, f64, f64), b: &(f64, f64, f64), alpha: f64) -> (f64, f64, f64) {
    let ab = (b.0 - a.0, b.1 - a.1, b.2 - a.2);
    (a.0 + alpha * ab.0, a.1 + alpha * ab.1, a.2 + alpha * ab.2)
}

fn two_closest_index(track: &track::Track, index: &usize, p: &InputPoint) -> (usize, usize) {
    let tracklen = track.euclidian.len();
    if *index == 0 {
        return (0, 1);
    }
    if *index == tracklen - 1 {
        return (index - 1, *index);
    }
    let dbefore = p.euclidean.d2(&track.euclidian[index - 1]);
    let dafter = p.euclidean.d2(&track.euclidian[index - 1]);
    if dbefore < dafter {
        (index - 1, *index)
    } else {
        (*index, *index + 1)
    }
}

pub fn compute_track_projection(
    track: &track::Track,
    tracktree: &IndexedPointsTree,
    point: &InputPoint,
) -> TrackProjection {
    // user steps projection on track is unique...
    if point.kind() == InputType::UserStep {
        assert!(point.track_projection.is_some());
        return point.track_projection.as_ref().unwrap().clone();
    }
    // as opposed to GPX and OSM points, which may be on several segments
    assert!(!point.track_projection.is_some());
    let index = tracktree.nearest_neighbor(&point.euclidean).unwrap();
    let (index1, index2) = two_closest_index(track, &index, point);
    let p1 = &track.euclidian[index1];
    let p2 = &track.euclidian[index2];
    let linestring: geo::LineString = vec![p1.xy(), p2.xy()].into();
    let index_floating_part = linestring
        .line_locate_point(&geo::point!(point.euclidean.xy()))
        .unwrap();
    assert!(0.0 <= index_floating_part && index_floating_part <= 1f64);
    let floating_index = index1 as f64 + index_floating_part;
    let t1 = &track.euclidian[index1];
    let t2 = &track.euclidian[index2];
    let a1 = (t1.0, t1.1, track.elevation(index1));
    let a2 = (t2.0, t2.1, track.elevation(index2));
    let m = middle_point(&a1, &a2, index_floating_part);
    let euclidean = MercatorPoint::from_point2d(&Point2D::new(m.0, m.1));
    let elevation = m.2;
    let track_distance = euclidean.d2(&point.euclidean).sqrt();

    // check
    let di = point.euclidean.d2(&track.euclidian[index]);
    let df = point.euclidean.d2(&euclidean);
    debug_assert!(df <= di);
    let new_proj = TrackProjection {
        track_floating_index: floating_index,
        track_index: index1,
        euclidean,
        elevation,
        track_distance,
    };
    new_proj
}
