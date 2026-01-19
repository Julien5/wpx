use std::cmp::Ordering;

use euclid::Point2D as EuclidPoint2D;
use serde::{Deserialize, Serialize};

pub struct ScreenSpace;

// Create a newtype wrapper that can derive Serialize/Deserialize
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Point2D {
    pub x: f64,
    pub y: f64,
}

pub type IntegerSize2D = euclid::Size2D<i32, ScreenSpace>;
pub type Vector2D = Point2D;

impl Point2D {
    pub fn new(x: f64, y: f64) -> Self {
        Self { x, y }
    }

    pub fn zero() -> Point2D {
        Point2D::new(0f64, 0f64)
    }

    pub fn length(&self) -> f64 {
        norm2(self).sqrt()
    }

    pub fn distance_to(&self, other: &Self) -> f64 {
        distance2(self, other).sqrt()
    }

    pub fn to_tuple(&self) -> (f64, f64) {
        (self.x, self.y)
    }

    // Convert to euclid::Point2D when needed
    pub fn to_euclid(&self) -> EuclidPoint2D<f64, ScreenSpace> {
        EuclidPoint2D::new(self.x, self.y)
    }

    // Convert from euclid::Point2D
    pub fn from_euclid(point: EuclidPoint2D<f64, ScreenSpace>) -> Self {
        Self {
            x: point.x,
            y: point.y,
        }
    }
}

// Implement From traits for easy conversion
impl From<EuclidPoint2D<f64, ScreenSpace>> for Point2D {
    fn from(point: EuclidPoint2D<f64, ScreenSpace>) -> Self {
        Self::from_euclid(point)
    }
}

impl From<Point2D> for EuclidPoint2D<f64, ScreenSpace> {
    fn from(point: Point2D) -> Self {
        point.to_euclid()
    }
}

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

impl PartialEq for Point2D {
    fn eq(&self, other: &Self) -> bool {
        self.x == other.x && self.y == other.y
    }
}

impl Eq for Point2D {}

use std::ops::Add;
impl Add for Point2D {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        Self {
            x: self.x + other.x,
            y: self.y + other.y,
        }
    }
}

use std::ops::Mul;

impl Mul<f64> for Point2D {
    type Output = Self;

    fn mul(self, scalar: f64) -> Self {
        Self {
            x: self.x * scalar,
            y: self.y * scalar,
        }
    }
}

impl Mul<Point2D> for f64 {
    type Output = Point2D;

    fn mul(self, point: Point2D) -> Point2D {
        Point2D {
            x: point.x * self,
            y: point.y * self,
        }
    }
}

use std::ops::Sub;

impl Sub for Point2D {
    type Output = Self;

    fn sub(self, other: Self) -> Self {
        Self {
            x: self.x - other.x,
            y: self.y - other.y,
        }
    }
}
