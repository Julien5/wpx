use petgraph::{
    algo::astar,
    graph::{NodeIndex, UnGraph},
};

use super::Obstacles;
use crate::math::*;

pub fn _compute(
    start_point: &Point2D,
    dest_point: &Point2D,
    original_obstacles: &Obstacles,
) -> Vec<Point2D> {
    // Currently we rebuild the whole graph for each request.
    // => TODO: cache the graph.

    // 1. Collect candidate points: A, B, corners of all bboxes
    let mut pointsnet: Vec<Point2D> = vec![start_point.clone(), dest_point.clone()];
    let eps = 2f64;
    for bbox in &original_obstacles.bboxes {
        let mut ebbox = bbox.clone();
        ebbox.enlarge(&eps);
        let corners = ebbox.corners();
        pointsnet.extend(corners);
    }

    // 2. Build visibility graph using petgraph
    let n = pointsnet.len();
    let mut graph = UnGraph::<(), f64>::new_undirected();
    let node_indices: Vec<NodeIndex> = (0..n).map(|_| graph.add_node(())).collect();

    for i in 0..n {
        for j in (i + 1)..n {
            if original_obstacles._is_clear(&pointsnet[i], &pointsnet[j]) {
                let d = pointsnet[i].distance_to(pointsnet[j]);
                graph.add_edge(node_indices[i], node_indices[j], d);
            }
        }
    }

    // 3. Use astar for shortest path and path reconstruction
    let start = node_indices[0];
    let goal = node_indices[1];
    if let Some((_cost, path_indices)) = astar(
        &graph,
        start,
        |finish| finish == goal,
        |e| *e.weight(),
        |_| 0.0,
    ) {
        let path: Vec<Point2D> = path_indices
            .into_iter()
            .map(|idx| {
                let i = node_indices.iter().position(|&n| n == idx).unwrap();
                pointsnet[i].clone()
            })
            .collect();
        if path.len() >= 2 {
            return path;
        }
    }

    // Fallback: if no path found, return direct A–B
    log::trace!("shortest path failed (no path found)");
    vec![start_point.clone(), dest_point.clone()]
}

#[cfg(test)]
mod tests {
    use crate::{bbox::BoundingBox, label_placement::DrawingArea};

    use super::*;

    #[test]
    fn test_segments_intersect_basic() {
        // Intersecting segments
        let a1 = Point2D::new(0.0, 0.0);
        let a2 = Point2D::new(2.0, 2.0);
        let b1 = Point2D::new(0.0, 2.0);
        let b2 = Point2D::new(2.0, 0.0);
        assert!(segments_intersect(&a1, &a2, &b1, &b2));
    }

    #[test]
    fn test_segments_intersect_parallel() {
        // Parallel, non-intersecting
        let a1 = Point2D::new(0.0, 0.0);
        let a2 = Point2D::new(2.0, 0.0);
        let b1 = Point2D::new(0.0, 1.0);
        let b2 = Point2D::new(2.0, 1.0);
        assert!(!segments_intersect(&a1, &a2, &b1, &b2));
    }

    #[test]
    fn test_segments_intersect_colinear_overlap() {
        // Colinear and overlapping
        let a1 = Point2D::new(0.0, 0.0);
        let a2 = Point2D::new(2.0, 0.0);
        let b1 = Point2D::new(1.0, 0.0);
        let b2 = Point2D::new(3.0, 0.0);
        assert!(segments_intersect(&a1, &a2, &b1, &b2));
    }

    #[test]
    fn test_segments_intersect_colinear_disjoint() {
        // Colinear but disjoint
        let a1 = Point2D::new(0.0, 0.0);
        let a2 = Point2D::new(1.0, 0.0);
        let b1 = Point2D::new(2.0, 0.0);
        let b2 = Point2D::new(3.0, 0.0);
        assert!(!segments_intersect(&a1, &a2, &b1, &b2));
    }

    #[test]
    fn test_segments_intersect_endpoint_touch() {
        // Touching at endpoint
        let a1 = Point2D::new(0.0, 0.0);
        let a2 = Point2D::new(1.0, 1.0);
        let b1 = Point2D::new(1.0, 1.0);
        let b2 = Point2D::new(2.0, 0.0);
        assert!(segments_intersect(&a1, &a2, &b1, &b2));
    }

    #[test]
    fn test_routing() {
        // Touching at endpoint
        let from = Point2D::new(0.0, 5.0);
        let to = Point2D::new(10.0, 5.0);
        let _bbox0 = BoundingBox::init(Point2D::new(-3.0, 3.0), Point2D::new(0.0, 7.0));
        let bbox1 = BoundingBox::init(Point2D::new(3.0, 3.0), Point2D::new(7.0, 7.0));
        let _bbox2 = BoundingBox::init(Point2D::new(10.0, 0.0), Point2D::new(17.0, 10.0));
        let obstables = Obstacles {
            bboxes: vec![bbox1],
            polylines: Vec::new(),
            drawingbox: DrawingArea {
                bbox: BoundingBox::init(Point2D::new(0.0, 0.0), Point2D::new(10.0, 40.0)),
                max_area_ratio: 0.0f64,
            },
        };
        let path = super::_compute(&from, &to, &obstables);
        for p in path {
            println!("{:.1} {:.1}", p.x, p.y);
        }
    }
}
