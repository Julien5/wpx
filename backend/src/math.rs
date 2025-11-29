use std::cmp::Ordering;
pub struct ScreenSpace;
pub type Point2D = euclid::Point2D<f64, ScreenSpace>;
pub type IntegerSize2D = euclid::Size2D<i32, ScreenSpace>;
pub type Vector2D = euclid::Vector2D<f64, ScreenSpace>;

pub fn partial_compare(p: &Point2D, q: &Point2D) -> Option<Ordering> {
    match p.x.partial_cmp(&q.x) {
        Some(o) => match o {
            Ordering::Equal => p.y.partial_cmp(&q.y),
            _ => Some(o),
        },
        None => Some(Ordering::Equal),
    }
}

pub fn norm2(p: &Vector2D) -> f64 {
    p.x * p.x + p.y * p.y
}

pub fn distance2(a: &Point2D, b: &Point2D) -> f64 {
    norm2(&(*b - *a))
}

pub fn segments_intersect(p1: &Point2D, p2: &Point2D, q1: &Point2D, q2: &Point2D) -> bool {
    // Helper to compute orientation
    fn orientation(a: &Point2D, b: &Point2D, c: &Point2D) -> i32 {
        let val = (b.y - a.y) * (c.x - b.x) - (b.x - a.x) * (c.y - b.y);
        if val.abs() < std::f64::EPSILON {
            0 // colinear
        } else if val > 0.0 {
            1 // clockwise
        } else {
            2 // counterclockwise
        }
    }

    // Helper to check if point c is on segment ab
    fn on_segment(a: &Point2D, b: &Point2D, c: &Point2D) -> bool {
        c.x >= a.x.min(b.x) && c.x <= a.x.max(b.x) && c.y >= a.y.min(b.y) && c.y <= a.y.max(b.y)
    }

    let o1 = orientation(p1, p2, q1);
    let o2 = orientation(p1, p2, q2);
    let o3 = orientation(q1, q2, p1);
    let o4 = orientation(q1, q2, p2);

    // General case
    if o1 != o2 && o3 != o4 {
        return true;
    }

    // Special cases
    if o1 == 0 && on_segment(p1, p2, q1) {
        return true;
    }
    if o2 == 0 && on_segment(p1, p2, q2) {
        return true;
    }
    if o3 == 0 && on_segment(q1, q2, p1) {
        return true;
    }
    if o4 == 0 && on_segment(q1, q2, p2) {
        return true;
    }

    false
}

pub fn nearly_equal(a: f64, b: f64) -> bool {
    let diff = (a - b).abs();
    diff <= 1e-5
}
