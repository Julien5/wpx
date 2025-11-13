## Problem

Let A and B be two points in the 2D space:
```
type Point = Point2D;
```
and obstacles be a struct containing a vector of bounding boxes and a polyline:
```
pub struct Obstacles {
    pub bboxes: Vec<BoundingBox>,
}
```
Here is the definition of a BoundingBox:
```
pub struct BoundingBox {
    _min: Point2D,
    _max: Point2D,
}
```
I need a function to compute a polyline such that:
1. the start point is A
2. the end point is B
3. no segment of that polyline crosses any bounding boxes in Obstacles::bboxes

A first implementation of that function would be:
```
pub type Point = Point2D;

pub fn compute(A: &Point, B: &Point, _obstacles: &Obstacles) -> Vec<Point> {
    let mut ret = Vec::new();
    ret.push(A.clone());
    ret.push(B.clone());
    ret
}
```
That implementation fulfills (1) and (2) but not (3). Make a plan for for completing the implementation.

```
// ======================================================
// PLAN: Compute a polyline from A to B avoiding obstacles
// ======================================================

pub type Point = Point2D;

pub struct BoundingBox {
    pub _min: Point2D,
    pub _max: Point2D,
}

pub struct Obstacles {
    pub bboxes: Vec<BoundingBox>,
}

// ======================================================
// 1. Geometry helper functions
// ======================================================

// Check if a point is inside a bounding box
fn point_in_bbox(p: &Point, bbox: &BoundingBox) -> bool {
    p.0 >= bbox._min.0 && p.0 <= bbox._max.0 &&
    p.1 >= bbox._min.1 && p.1 <= bbox._max.1
}

// Return the four edges of the bounding box as line segments
fn bbox_edges(bbox: &BoundingBox) -> [(Point, Point); 4] {
    let (xmin, ymin) = bbox._min;
    let (xmax, ymax) = bbox._max;

    [
        ((xmin, ymin), (xmax, ymin)), // bottom
        ((xmax, ymin), (xmax, ymax)), // right
        ((xmax, ymax), (xmin, ymax)), // top
        ((xmin, ymax), (xmin, ymin)), // left
    ]
}

// Check if two line segments intersect
fn segments_intersect(p1: &Point, p2: &Point, q1: &Point, q2: &Point) -> bool {
    fn cross(a: &Point, b: &Point, c: &Point) -> f64 {
        (b.0 - a.0) * (c.1 - a.1) - (b.1 - a.1) * (c.0 - a.0)
    }

    let d1 = cross(p1, p2, q1);
    let d2 = cross(p1, p2, q2);
    let d3 = cross(q1, q2, p1);
    let d4 = cross(q1, q2, p2);

    if (d1 * d2) < 0.0 && (d3 * d4) < 0.0 {
        return true;
    }
    false
}

// Check if a segment intersects a bounding box
fn segment_intersects_bbox(p1: &Point, p2: &Point, bbox: &BoundingBox) -> bool {
    if point_in_bbox(p1, bbox) || point_in_bbox(p2, bbox) {
        return true;
    }
    for (q1, q2) in bbox_edges(bbox) {
        if segments_intersect(p1, p2, &q1, &q2) {
            return true;
        }
    }
    false
}

// ======================================================
// 2. Check if a path segment is clear of all obstacles
// ======================================================
fn is_clear(p1: &Point, p2: &Point, obstacles: &Obstacles) -> bool {
    for bbox in &obstacles.bboxes {
        if segment_intersects_bbox(p1, p2, bbox) {
            return false;
        }
    }
    true
}

// ======================================================
// 3. Main function: compute polyline from A to B
// ======================================================

pub fn compute(A: &Point, B: &Point, obstacles: &Obstacles) -> Vec<Point> {
    // If direct line is clear, just connect A to B
    if is_clear(A, B, obstacles) {
        return vec![A.clone(), B.clone()];
    }

    // ==================================================
    // Simple heuristic: try routing around one obstacle
    // ==================================================
    let mut best_path: Option<Vec<Point>> = None;

    for bbox in &obstacles.bboxes {
        if !segment_intersects_bbox(A, B, bbox) {
            continue;
        }

        // Compute detour points just outside each face
        let (xmin, ymin) = bbox._min;
        let (xmax, ymax) = bbox._max;
        let eps = 1e-3; // small offset to stay outside box

        let detours = vec![
            ((xmin - eps, (ymin + ymax) / 2.0)), // left
            ((xmax + eps, (ymin + ymax) / 2.0)), // right
            (((xmin + xmax) / 2.0, ymin - eps)), // bottom
            (((xmin + xmax) / 2.0, ymax + eps)), // top
        ];

        // Try each detour: A → detour → B
        for d in detours {
            if is_clear(A, &d, obstacles) && is_clear(&d, B, obstacles) {
                let path = vec![A.clone(), d, B.clone()];
                best_path = Some(path);
                break;
            }
        }

        if best_path.is_some() {
            break;
        }
    }

    // Return found path or fallback to direct A–B
    best_path.unwrap_or_else(|| vec![A.clone(), B.clone()])
}
```
