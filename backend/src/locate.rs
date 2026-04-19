use crate::math::Point2D;
use crate::mercator::MercatorPoint;
use crate::point_collection::Kind;
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
    fn indexed_track_range(
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
        let ipoints = Self::indexed_track_range(euclideans, range);
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

mod projection {
    use crate::mercator::MercatorPoint;

    pub struct PartialProjection {
        pub track_floating_index: f64,
        pub projection: MercatorPoint,
    }

    pub fn compute(
        track: &Vec<MercatorPoint>,
        point: &MercatorPoint,
        closest_index: &usize,
    ) -> PartialProjection {
        let idx = *closest_index;

        // Define potential segments to check: (idx-1, idx) and (idx, idx+1)
        let mut candidates = Vec::new();

        if idx > 0 {
            candidates.push(project_on_segment(idx - 1, idx, track, point));
        }
        if idx < track.len() - 1 {
            candidates.push(project_on_segment(idx, idx + 1, track, point));
        }

        // Fallback if the track has only one point
        if candidates.is_empty() {
            return PartialProjection {
                track_floating_index: idx as f64,
                projection: MercatorPoint(track[idx].0, track[idx].1),
            };
        }

        // Return the candidate with the smallest Euclidean distance to the target point
        candidates
            .into_iter()
            .min_by(|a, b| {
                let dist_a = point.d2(&a.projection);
                let dist_b = point.d2(&b.projection);
                dist_a
                    .partial_cmp(&dist_b)
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
            .unwrap()
    }

    fn project_on_segment(
        i0: usize,
        i1: usize,
        track: &[MercatorPoint],
        p: &MercatorPoint,
    ) -> PartialProjection {
        let p0 = &track[i0];
        let p1 = &track[i1];

        let dx = p1.0 - p0.0;
        let dy = p1.1 - p0.1;
        let line_len_sq = dx * dx + dy * dy;

        if line_len_sq == 0.0 {
            return PartialProjection {
                track_floating_index: i0 as f64,
                projection: MercatorPoint(p0.0, p0.1),
            };
        }

        // Scalar projection factor t
        let t = ((p.0 - p0.0) * dx + (p.1 - p0.1) * dy) / line_len_sq;

        // Clamp t to the segment [0, 1]
        let t_clamped = t.max(0.0).min(1.0);

        PartialProjection {
            track_floating_index: i0 as f64 + t_clamped,
            projection: MercatorPoint(p0.0 + t_clamped * dx, p0.1 + t_clamped * dy),
        }
    }
}

pub fn compute_track_projection_2d(
    track: &Vec<MercatorPoint>,
    tracktree: &IndexedPointsTree,
    point: &MercatorPoint,
) -> TrackProjection {
    // as opposed to GPX and OSM points, which may be on several segments
    let index = tracktree.nearest_neighbor(&point).unwrap();
    let partial = projection::compute(track, point, &index);
    let floating_index = partial.track_floating_index;
    let m = partial.projection;

    let middle = MercatorPoint::from_point2d(&Point2D::new(m.0, m.1));

    let track_distance = middle.d2(&point).sqrt();

    let di = point.d2(&track[index]);
    let df = point.d2(&middle);
    debug_assert!(df <= di);

    let new_proj = TrackProjection {
        track_floating_index: floating_index,
        track_index: index,
        euclidean: middle,
        elevation: 0f64,
        track_distance,
        distance_on_track_to_projection: 0f64,
    };
    new_proj
}

pub fn compute_track_projection(
    track: &Vec<MercatorPoint>,
    distance: impl Fn(usize) -> f64,
    elevation: impl Fn(usize) -> f64,
    tracktree: &IndexedPointsTree,
    point: &InputPoint,
) -> TrackProjection {
    // user steps projection on track is unique...
    if point.kind() == Kind::CutOff {
        assert!(!point.track_projections.is_empty());
        return point.track_projections.first().unwrap().clone();
    }
    // as opposed to GPX and OSM points, which may be on several segments
    let index = tracktree.nearest_neighbor(&point.euclidean).unwrap();
    let partial = projection::compute(track, &point.euclidean, &index);
    let index1 = partial.track_floating_index.floor() as usize;
    let index2 = (index1 + 1).min(track.len() - 1) as usize;
    let p1 = &track[index1];
    let p2 = &track[index2];
    let linestring: geo::LineString = vec![p1.xy(), p2.xy()].into();
    let index_floating_part = linestring
        .line_locate_point(&geo::point!(point.euclidean.xy()))
        .unwrap();
    assert!(0.0 <= index_floating_part && index_floating_part <= 1f64);
    let floating_index = index1 as f64 + index_floating_part;
    let t1 = &track[index1];
    let t2 = &track[index2];
    let a1 = (t1.0, t1.1, elevation(index1));
    let a2 = (t2.0, t2.1, elevation(index2));
    let m = middle_point(&a1, &a2, index_floating_part);

    let middle = MercatorPoint::from_point2d(&Point2D::new(m.0, m.1));

    let elevation = m.2;
    let track_distance = middle.d2(&point.euclidean).sqrt();

    let di = point.euclidean.d2(&track[index]);
    let df = point.euclidean.d2(&middle);
    debug_assert!(df <= di);

    let distance_on_track_to_projection = distance(index) + track[index].d2(&middle).sqrt();
    let new_proj = TrackProjection {
        track_floating_index: floating_index,
        track_index: index,
        euclidean: middle,
        elevation,
        track_distance,
        distance_on_track_to_projection,
    };
    new_proj
}
