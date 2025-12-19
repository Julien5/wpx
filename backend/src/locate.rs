use crate::math::Point2D;
use crate::mercator::MercatorPoint;
use crate::track::{self};
use crate::track_projection::TrackProjection;
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

#[derive(Clone)]
pub struct IndexedPointsTree {
    tree: RTree<IndexedPoint>,
    pub range: std::ops::Range<usize>,
}

fn coord(point: &MercatorPoint) -> [f64; 2] {
    [point.x(), point.y()]
}

impl IndexedPointsTree {
    fn indexed_track(
        euclideans: &Vec<MercatorPoint>,
        range: &std::ops::Range<usize>,
    ) -> Vec<IndexedPoint> {
        let mut ret = Vec::new();
        for k in range.start..range.end {
            ret.push(IndexedPoint {
                coord: euclideans[k].clone(),
                index: k,
            });
        }
        ret
    }

    pub fn from_track(
        euclideans: &Vec<MercatorPoint>,
        range: &std::ops::Range<usize>,
    ) -> IndexedPointsTree {
        let ipoints = Self::indexed_track(euclideans, range);
        let tree = RTree::bulk_load(ipoints);
        IndexedPointsTree {
            tree,
            range: range.clone(),
        }
    }
    pub fn nearest_neighbor(&self, point: &MercatorPoint) -> Option<usize> {
        let nearest = self.tree.nearest_neighbor(&coord(point));
        match nearest {
            Some(indexed) => Some(indexed.index),
            None => None,
        }
    }
}

fn middle_point(a: &(f64, f64, f64), b: &(f64, f64, f64), alpha: f64) -> (f64, f64, f64) {
    let ab = (b.0 - a.0, b.1 - a.1, b.2 - a.2);
    (a.0 + alpha * ab.0, a.1 + alpha * ab.1, a.2 + alpha * ab.2)
}

fn two_closest_index(track: &track::Track, index: &usize, p: &InputPoint) -> (usize, usize) {
    let tracklen = track.euclidean.len();
    if *index == 0 {
        return (0, 1);
    }
    if *index == tracklen - 1 {
        return (index - 1, *index);
    }
    let dbefore = p.euclidean.d2(&track.euclidean[index - 1]);
    let dafter = p.euclidean.d2(&track.euclidean[index - 1]);
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
        assert!(!point.track_projections.is_empty());
        return point.track_projections.first().unwrap().clone();
    }
    // as opposed to GPX and OSM points, which may be on several segments
    let index = tracktree.nearest_neighbor(&point.euclidean).unwrap();
    let (index1, index2) = two_closest_index(track, &index, point);
    let p1 = &track.euclidean[index1];
    let p2 = &track.euclidean[index2];
    let linestring: geo::LineString = vec![p1.xy(), p2.xy()].into();
    let index_floating_part = linestring
        .line_locate_point(&geo::point!(point.euclidean.xy()))
        .unwrap();
    assert!(0.0 <= index_floating_part && index_floating_part <= 1f64);
    let floating_index = index1 as f64 + index_floating_part;
    let t1 = &track.euclidean[index1];
    let t2 = &track.euclidean[index2];
    let a1 = (t1.0, t1.1, track.elevation(index1));
    let a2 = (t2.0, t2.1, track.elevation(index2));
    let m = middle_point(&a1, &a2, index_floating_part);

    let middle = MercatorPoint::from_point2d(&Point2D::new(m.0, m.1));

    let elevation = m.2;
    let track_distance = middle.d2(&point.euclidean).sqrt();

    let di = point.euclidean.d2(&track.euclidean[index]);
    let df = point.euclidean.d2(&middle);
    debug_assert!(df <= di);

    let distance_on_track_to_projection =
        track.distance(index) + track.euclidean[index].d2(&middle).sqrt();
    let new_proj = TrackProjection {
        track_floating_index: floating_index,
        track_index: index1,
        euclidean: middle,
        elevation,
        track_distance,
        distance_on_track_to_projection,
    };
    new_proj
}
