use core::fmt;

#[derive(Clone, PartialEq)]
pub struct BoundingBox {
    _min: (f64, f64),
    _max: (f64, f64),
}

impl BoundingBox {
    pub fn init(min: (f64, f64), max: (f64, f64)) -> BoundingBox {
        BoundingBox {
            _min: min,
            _max: max,
        }
    }
    pub fn get_min(&self) -> (f64, f64) {
        self._min
    }
    pub fn get_xmin(&self) -> f64 {
        self._min.0
    }
    pub fn get_ymin(&self) -> f64 {
        self._min.1
    }
    pub fn get_max(&self) -> (f64, f64) {
        self._max
    }
    pub fn get_xmax(&self) -> f64 {
        self._max.0
    }
    pub fn get_ymax(&self) -> f64 {
        self._max.1
    }
    pub fn set_min(&mut self, m: (f64, f64)) {
        self._min = m;
    }
    pub fn set_max(&mut self, m: (f64, f64)) {
        self._max = m;
    }
    pub fn set_ymin(&mut self, m: f64) {
        self._min.1 = m;
    }
    pub fn set_ymax(&mut self, m: f64) {
        self._max.1 = m;
    }
    pub fn middle(&self) -> (f64, f64) {
        (
            0.5f64 * (self._min.0 + self._max.0),
            0.5f64 * (self._min.1 + self._max.1),
        )
    }
    pub fn contains_other(&self, other: &Self) -> bool {
        if other._max.0 > self._max.0 {
            return false;
        }
        if other._max.1 > self._max.1 {
            return false;
        }
        if other._min.0 < self._min.0 {
            return false;
        }
        if other._min.1 < self._min.1 {
            return false;
        }
        true
    }
    pub fn new() -> BoundingBox {
        let min = (f64::MAX, f64::MAX);
        let max = (f64::MIN, f64::MIN);
        BoundingBox {
            _min: min,
            _max: max,
        }
    }
    pub fn empty(&self) -> bool {
        self._min.0 > self._max.0
    }
    pub fn width(&self) -> f64 {
        return self._max.0 - self._min.0;
    }
    pub fn height(&self) -> f64 {
        return self._max.1 - self._min.1;
    }
    pub fn update(&mut self, p: &(f64, f64)) {
        self._min.0 = self._min.0.min(p.0);
        self._min.1 = self._min.1.min(p.1);
        self._max.0 = self._max.0.max(p.0);
        self._max.1 = self._max.1.max(p.1);
    }
    // TODO: take WxH into account
    pub fn fix_aspect_ratio(&mut self, _w: i32, _h: i32) {
        let x = (self._min.0 + self._max.0) / 2f64;
        let y = (self._min.1 + self._max.1) / 2f64;
        if self.height() > self.width() {
            let delta = 0.5f64 * (self.height());
            self._max.0 = x + delta;
            self._min.0 = x - delta;
        } else {
            let delta = 0.5f64 * self.width();
            self._max.1 = y + delta;
            self._min.1 = y - delta;
        }
        let margin = 0f64; //2000f64;
        self._max.0 = self._max.0 + margin;
        self._max.1 = self._max.1 + margin;
        self._min.0 = self._min.0 - margin;
        self._min.1 = self._min.1 - margin;
    }
    pub fn contains(&self, p: &(f64, f64)) -> bool {
        if p.0 < self._min.0 {
            return false;
        }
        if p.0 > self._max.0 {
            return false;
        }
        if p.1 < self._min.1 {
            return false;
        }
        if p.1 > self._max.1 {
            return false;
        }
        return true;
    }
    pub fn points(&self) -> [(f64, f64); 4] {
        [
            self.get_min(),
            self.get_max(),
            (self._min.0, self._max.1),
            (self._min.1, self._max.0),
        ]
    }
    pub fn hits_other(&self, other: &Self) -> bool {
        for p in other.points() {
            if self.contains(&p) {
                return true;
            }
        }
        for p in self.points() {
            if other.contains(&p) {
                return true;
            }
        }
        false
    }
    pub fn enlarge(&mut self, delta: &f64) {
        self._min.0 -= delta;
        self._min.1 -= delta;
        self._max.0 += delta;
        self._max.1 += delta;
    }
}

impl fmt::Debug for BoundingBox {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("BoundingBox")
            .field("min.0", &self._min.0)
            .field("min.1", &self._min.1)
            .field("max.0", &self._max.0)
            .field("max.1", &self._max.1)
            .finish()
    }
}

use std::{
    cmp::Ordering,
    hash::{Hash, Hasher},
};

impl Hash for BoundingBox {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self._min.0.to_bits().hash(state);
        self._min.1.to_bits().hash(state);
        self._max.0.to_bits().hash(state);
        self._max.1.to_bits().hash(state);
    }
}

impl Eq for BoundingBox {}

impl PartialOrd for BoundingBox {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self._min.partial_cmp(&other._min).and_then(|ord| {
            if ord == Ordering::Equal {
                self._max.partial_cmp(&other._max)
            } else {
                Some(ord)
            }
        })
    }
}

impl Ord for BoundingBox {
    fn cmp(&self, other: &Self) -> Ordering {
        self._min
            .partial_cmp(&other._min)
            .unwrap_or(Ordering::Equal)
            .then_with(|| {
                self._max
                    .partial_cmp(&other._max)
                    .unwrap_or(Ordering::Equal)
            })
    }
}

impl fmt::Display for BoundingBox {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "BoundingBox {{ min: ({:.2}, {:.2}), max: ({:.2}, {:.2})",
            self._min.0, self._min.1, self._max.0, self._max.1,
        )
    }
}
