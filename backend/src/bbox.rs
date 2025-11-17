use core::fmt;

#[derive(Clone, PartialEq)]
pub struct BoundingBox {
    _min: Point2D,
    _max: Point2D,
}

impl BoundingBox {
    pub fn init(min: Point2D, max: Point2D) -> BoundingBox {
        BoundingBox {
            _min: min,
            _max: max,
        }
    }
    pub fn get_min(&self) -> Point2D {
        self._min.clone()
    }
    pub fn get_xmin(&self) -> f64 {
        self._min.x
    }
    pub fn get_ymin(&self) -> f64 {
        self._min.y
    }
    pub fn get_max(&self) -> Point2D {
        self._max.clone()
    }
    pub fn get_xmax(&self) -> f64 {
        self._max.x
    }
    pub fn get_ymax(&self) -> f64 {
        self._max.y
    }
    pub fn set_min(&mut self, m: Point2D) {
        self._min = m;
    }
    pub fn set_max(&mut self, m: Point2D) {
        self._max = m;
    }
    pub fn set_ymin(&mut self, m: f64) {
        self._min.y = m;
    }
    pub fn set_ymax(&mut self, m: f64) {
        self._max.y = m;
    }
    pub fn contains_other(&self, other: &Self) -> bool {
        if other._max.x > self._max.x {
            return false;
        }
        if other._max.y > self._max.y {
            return false;
        }
        if other._min.x < self._min.x {
            return false;
        }
        if other._min.y < self._min.y {
            return false;
        }
        true
    }

    pub fn corners(&self) -> [Point2D; 4] {
        [
            self.get_min(),
            self.get_max(),
            Point2D::new(self.get_xmin(), self.get_ymax()),
            Point2D::new(self.get_xmax(), self.get_ymin()),
        ]
    }

    pub fn edges(&self) -> [(Point2D, Point2D); 4] {
        let c = self.corners();
        [
            (c[0], c[3]), // left edge: min -> (min.x, max.y)
            (c[3], c[1]), // top edge: (min.x, max.y) -> max
            (c[1], c[2]), // right edge: max -> (max.x, min.y)
            (c[2], c[0]), // bottom edge: (max.x, min.y) -> min
        ]
    }

    pub fn segment_intersects(&self, p1: &Point2D, p2: &Point2D) -> bool {
        if self.contains(&p1) || self.contains(&p2) {
            return true;
        }
        for (q1, q2) in self.edges() {
            if crate::math::segments_intersect(p1, p2, &q1, &q2) {
                return true;
            }
        }
        false
    }

    fn overlap_self(&self, other: &Self) -> bool {
        for p in self.corners() {
            if other.contains(&p) {
                return true;
            }
        }
        false
    }
    pub fn overlap(&self, other: &Self) -> bool {
        if other.overlap_self(self) || self.overlap_self(other) {
            return true;
        }
        false
    }

    pub fn new() -> BoundingBox {
        BoundingBox {
            _min: Point2D::new(f64::MAX, f64::MAX),
            _max: Point2D::new(f64::MIN, f64::MIN),
        }
    }
    pub fn empty(&self) -> bool {
        self._min.x > self._max.x
    }
    pub fn width(&self) -> f64 {
        return self._max.x - self._min.x;
    }
    pub fn height(&self) -> f64 {
        return self._max.y - self._min.y;
    }
    pub fn area(&self) -> f64 {
        self.width() * self.height()
    }
    pub fn update(&mut self, p: &Point2D) {
        self._min.x = self._min.x.min(p.x);
        self._min.y = self._min.y.min(p.y);
        self._max.x = self._max.x.max(p.x);
        self._max.y = self._max.y.max(p.y);
    }
    // TODO: take WxH into account
    pub fn fix_aspect_ratio(&mut self, wanted: &IntegerSize2D) {
        let x = (self._min.x + self._max.x) / 2f64;
        let y = (self._min.y + self._max.y) / 2f64;
        let (wanted_width, wanted_height) = (wanted.width as f64, wanted.height as f64);
        let alpha = (wanted_width / self.width()) * (self.height() / wanted_height);
        let alpha_w = if self.height() > self.width() {
            alpha
        } else {
            1f64
        };
        let alpha_h = if self.height() < self.width() {
            1f64 / alpha
        } else {
            1f64
        };
        let new_width = alpha_w * self.width();
        let new_height = alpha_h * self.height();
        let deltaw = 0.5f64 * new_width;
        let deltah = 0.5f64 * new_height;
        self._max.x = x + deltaw;
        self._min.x = x - deltaw;
        self._max.y = y + deltah;
        self._min.y = y - deltah;
    }
    pub fn contains(&self, p: &Point2D) -> bool {
        if p.x < self._min.x {
            return false;
        }
        if p.x > self._max.x {
            return false;
        }
        if p.y < self._min.y {
            return false;
        }
        if p.y > self._max.y {
            return false;
        }
        return true;
    }
    pub fn enlarge(&mut self, delta: &f64) {
        self._min.x -= delta;
        self._min.y -= delta;
        self._max.x += delta;
        self._max.y += delta;
    }
    pub fn distance2_to_point(&self, q: &Point2D) -> f64 {
        let p = self.project_on_border(q);
        distance2(&p, q)
    }
    pub fn project_on_border(&self, q: &Point2D) -> Point2D {
        // Calculate distances to each edge
        let left = self.get_xmin();
        let right = self.get_xmax();
        let top = self.get_ymin();
        let bottom = self.get_ymax();

        let pbottom = Point2D::new(q.x.clamp(left, right), bottom);
        let ptop = Point2D::new(q.x.clamp(left, right), top);
        let pleft = Point2D::new(left, q.y.clamp(top, bottom));
        let pright = Point2D::new(right, q.y.clamp(top, bottom));

        let all = [
            (pbottom.clone(), distance2(&pbottom, &q)),
            (ptop.clone(), distance2(&ptop, &q)),
            (pleft.clone(), distance2(&pleft, &q)),
            (pright.clone(), distance2(&pright, &q)),
        ];
        let min = all
            .iter()
            .min_by(|a, b| a.1.partial_cmp(&b.1).unwrap())
            .unwrap();
        min.0
    }
    pub fn distance2_to_other(&self, other: &BoundingBox) -> f64 {
        if self.overlap(&other) {
            return 0f64;
        }
        let distances: Vec<_> = other
            .corners()
            .iter()
            .map(|point| {
                let p = self.project_on_border(&point);
                let (dx, dy) = ((point.x - p.x), (point.y - p.y));
                dx * dx + dy * dy
            })
            .collect();
        let min = distances.iter().fold(f64::INFINITY, |a, &b| a.min(b));
        min
    }
}

impl fmt::Debug for BoundingBox {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("BoundingBox")
            .field("min.x", &self._min.x)
            .field("min.y", &self._min.y)
            .field("max.x", &self._max.x)
            .field("max.y", &self._max.y)
            .finish()
    }
}

use std::{
    cmp::Ordering,
    hash::{Hash, Hasher},
};

use crate::math::{distance2, partial_compare, IntegerSize2D, Point2D};

impl Hash for BoundingBox {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self._min.x.to_bits().hash(state);
        self._min.y.to_bits().hash(state);
        self._max.x.to_bits().hash(state);
        self._max.y.to_bits().hash(state);
    }
}

impl PartialOrd for BoundingBox {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        match partial_compare(&self._min, &other._min) {
            Some(o) => Some(o),
            None => match partial_compare(&self._max, &other._max) {
                Some(ox) => Some(ox),
                None => None,
            },
        }
    }
}

impl Ord for BoundingBox {
    fn cmp(&self, other: &Self) -> Ordering {
        match partial_compare(&self._min, &other._min) {
            Some(Ordering::Equal) => {
                match partial_compare(&self._max, &other._max) {
                    Some(ord) => ord,
                    None => Ordering::Equal, // fallback if comparison is undefined
                }
            }
            Some(ord) => ord,
            None => Ordering::Equal, // fallback if comparison is undefined
        }
    }
}

impl Eq for BoundingBox {}

impl fmt::Display for BoundingBox {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "BoundingBox {{ min: ({:.2}, {:.2}), max: ({:.2}, {:.2})",
            self._min.x, self._min.y, self._max.x, self._max.y,
        )
    }
}
