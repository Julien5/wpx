use core::fmt;

#[derive(Clone, PartialEq)]
pub struct BoundingBox {
    pub min: (f64, f64),
    pub max: (f64, f64),
}

impl BoundingBox {
    pub fn init(min: (f64, f64), max: (f64, f64)) -> BoundingBox {
        BoundingBox { min, max }
    }
    pub fn min(&self) -> (f64, f64) {
        self.min
    }
    pub fn max(&self) -> (f64, f64) {
        self.max
    }
    pub fn new() -> BoundingBox {
        let min = (f64::MAX, f64::MAX);
        let max = (f64::MIN, f64::MIN);
        BoundingBox { min, max }
    }
    pub fn empty(&self) -> bool {
        self.min.0 > self.max.0
    }
    pub fn width(&self) -> f64 {
        return self.max.0 - self.min.0;
    }
    pub fn height(&self) -> f64 {
        return self.max.1 - self.min.1;
    }
    pub fn update(&mut self, p: &(f64, f64)) {
        self.min.0 = self.min.0.min(p.0);
        self.min.1 = self.min.1.min(p.1);
        self.max.0 = self.max.0.max(p.0);
        self.max.1 = self.max.1.max(p.1);
    }
    // TODO: take WxH into account
    pub fn fix_aspect_ratio(&mut self, _w: i32, _h: i32) {
        let x = (self.min.0 + self.max.0) / 2f64;
        let y = (self.min.1 + self.max.1) / 2f64;
        if self.height() > self.width() {
            let delta = 0.5f64 * (self.height());
            self.max.0 = x + delta;
            self.min.0 = x - delta;
        } else {
            let delta = 0.5f64 * self.width();
            self.max.1 = y + delta;
            self.min.1 = y - delta;
        }
        let margin = 2000f64;
        self.max.0 = self.max.0 + margin;
        self.max.1 = self.max.1 + margin;
        self.min.0 = self.min.0 - margin;
        self.min.1 = self.min.1 - margin;
    }
    pub fn contains(&self, p: &(f64, f64)) -> bool {
        if p.0 < self.min.0 {
            return false;
        }
        if p.0 > self.max.0 {
            return false;
        }
        if p.1 < self.min.1 {
            return false;
        }
        if p.1 > self.max.1 {
            return false;
        }
        return true;
    }
}

impl fmt::Debug for BoundingBox {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("WGS84BoundingBox")
            .field("minlon", &self.min.0)
            .field("minlat", &self.min.1)
            .field("maxlon", &self.max.0)
            .field("maxlat", &self.max.1)
            .finish()
    }
}

impl fmt::Display for BoundingBox {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "BoundingBox {{ min: ({:.2}, {:.2}), max: ({:.2}, {:.2})",
            self.min.0, self.min.1, self.max.0, self.max.1,
        )
    }
}
