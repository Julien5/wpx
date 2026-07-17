use geo::SimplifyIdx;

use crate::{
    locate::{self, IndexedPointsTree},
    mercator::{EuclideanBoundingBox, MercatorPoint},
    track_projection::TrackProjection,
};

#[derive(Clone)]
pub struct MapGeometry {
    xypoints: Vec<MercatorPoint>,
    simplified_indices: Vec<usize>,
    simplified_points: Vec<MercatorPoint>,
    graphics_tree: IndexedPointsTree,
}

impl MapGeometry {
    pub fn new(euclidean: &Vec<MercatorPoint>, track_distance: f64) -> Self {
        let simplified_indices = {
            let coords: Vec<geo::Coord> = euclidean
                .iter()
                .map(|p| geo::coord!(x: p.x(), y: p.y()))
                .collect();
            let line = geo::LineString::new(coords);
            let epsilon = track_distance * 500f64 / 1200_000f64;
            line.simplify_idx(epsilon)
        };
        let simplified_points: Vec<MercatorPoint> = simplified_indices
            .iter()
            .map(|idx| euclidean[*idx].clone())
            .collect();

        let graphics_tree =
            IndexedPointsTree::from_track(&simplified_points, &(0..simplified_points.len()));

        Self {
            xypoints: euclidean.clone(),
            simplified_indices,
            simplified_points,
            graphics_tree,
        }
    }

    pub fn len(&self) -> usize {
        self.xypoints.len()
    }

    pub fn is_empty(&self) -> bool {
        self.xypoints.is_empty()
    }

    pub fn point_at(&self, idx: usize) -> &MercatorPoint {
        &self.xypoints[idx]
    }

    pub fn first(&self) -> &MercatorPoint {
        &self.xypoints[0]
    }

    pub fn last(&self) -> &MercatorPoint {
        &self.xypoints[self.xypoints.len() - 1]
    }

    pub fn all_points(&self) -> &Vec<MercatorPoint> {
        &self.xypoints
    }

    pub fn simplified_indices(&self) -> &[usize] {
        &self.simplified_indices
    }

    pub fn simplified_points(&self) -> &Vec<MercatorPoint> {
        &self.simplified_points
    }

    pub fn iter(&self) -> std::slice::Iter<'_, MercatorPoint> {
        self.xypoints.iter()
    }

    pub fn bounding_box(&self) -> EuclideanBoundingBox {
        debug_assert!(!self.xypoints.is_empty());
        let mut ret = EuclideanBoundingBox::new();
        for p in &self.xypoints {
            ret.update(&p.point2d());
        }
        ret
    }

    pub fn project_graphics(&self, point: &MercatorPoint) -> TrackProjection {
        locate::compute_track_projection_2d(&self.simplified_points, &self.graphics_tree, point)
    }
}
