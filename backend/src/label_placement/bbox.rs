pub fn distance((x1, y1): (f64, f64), (x2, y2): (f64, f64)) -> f64 {
    let dx = x2 - x1;
    let dy = y2 - y1;
    (dx * dx + dy * dy).sqrt()
}

#[derive(Clone)]
pub struct LabelBoundingBox {
    pub bbox: BoundingBox,
}

impl LabelBoundingBox {
    pub fn zero() -> Self {
        LabelBoundingBox {
            bbox: BoundingBox::new(),
        }
    }
    pub fn new_tlbr(top_left: (f64, f64), bottom_right: (f64, f64)) -> Self {
        LabelBoundingBox {
            bbox: BoundingBox::init(top_left, bottom_right),
        }
    }

    pub fn new_tlwh(top_left: (f64, f64), width: f64, height: f64) -> Self {
        let bottom_right = (top_left.0 + width, top_left.1 + height);
        LabelBoundingBox {
            bbox: BoundingBox::init(top_left, bottom_right),
        }
    }

    pub fn new_blwh(bottom_left: (f64, f64), width: f64, height: f64) -> Self {
        let top_left = (bottom_left.0, bottom_left.1 - height);
        let bottom_right = (bottom_left.0 + width, bottom_left.1);
        LabelBoundingBox {
            bbox: BoundingBox::init(top_left, bottom_right),
        }
    }

    pub fn _new_brwh(bottom_right: (f64, f64), width: f64, height: f64) -> Self {
        let top_left = (bottom_right.0 - width, bottom_right.1 - height);
        LabelBoundingBox {
            bbox: BoundingBox::init(top_left, bottom_right),
        }
    }

    pub fn _new_trwh(top_right: (f64, f64), width: f64, height: f64) -> Self {
        let top_left = (top_right.0 - width, top_right.1);
        let bottom_right = (top_right.0, top_right.1 + height);
        LabelBoundingBox {
            bbox: BoundingBox::init(top_left, bottom_right),
        }
    }

    pub fn x_min(&self) -> f64 {
        self.bbox.min().0
    }

    pub fn y_min(&self) -> f64 {
        self.bbox.min().1
    }

    fn bottom_left(&self) -> (f64, f64) {
        (self.x_min(), self.y_max())
    }

    fn top_right(&self) -> (f64, f64) {
        (self.x_max(), self.y_min())
    }

    pub fn _center(&self) -> (f64, f64) {
        (
            0.5 * (self.x_min() + self.x_max()),
            0.5 * (self.y_min() + self.y_max()),
        )
    }

    pub fn x_max(&self) -> f64 {
        self.bbox.max().0
    }

    pub fn y_max(&self) -> f64 {
        self.bbox.max().1
    }

    pub fn width(&self) -> f64 {
        self.x_max() - self.x_min()
    }

    pub fn height(&self) -> f64 {
        self.y_max() - self.y_min()
    }
    pub fn project_on_border(&self, q: (f64, f64)) -> (f64, f64) {
        let (qx, qy) = q;

        // Calculate distances to each edge
        let left = self.x_min();
        let right = self.x_max();
        let top = self.y_min();
        let bottom = self.y_max();

        let dist_left = (qx - left).abs();
        let dist_right = (qx - right).abs();
        let dist_top = (qy - top).abs();
        let dist_bottom = (qy - bottom).abs();

        // Find the closest edge
        let min_dist = dist_left.min(dist_right).min(dist_top).min(dist_bottom);

        if min_dist == dist_left {
            (left, qy.clamp(top, bottom)) // Project onto the left edge
        } else if min_dist == dist_right {
            (right, qy.clamp(top, bottom)) // Project onto the right edge
        } else if min_dist == dist_top {
            (qx.clamp(left, right), top) // Project onto the top edge
        } else {
            (qx.clamp(left, right), bottom) // Project onto the bottom edge
        }
    }
    pub fn distance(&self, q: (f64, f64)) -> f64 {
        let p = self.project_on_border(q);
        distance(p, q)
    }
    pub fn contains(&self, (x, y): (f64, f64)) -> bool {
        if x >= self.x_min() && x <= self.x_max() && y >= self.y_min() && y <= self.y_max() {
            return true;
        }
        false
    }
    fn overlap_self(&self, other: &Self) -> bool {
        for p in [
            self.bbox.min(),
            self.bbox.max(),
            self.bottom_left(),
            self.top_right(),
        ] {
            if other.contains(p) {
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
    fn area2(&self) -> f64 {
        let dx = self.x_max() - self.x_min();
        let dy = self.y_max() - self.y_min();
        return dx * dy;
    }
    fn intersection(&self, other: &Self) -> Option<LabelBoundingBox> {
        let x_min = self.x_min().max(other.x_min());
        let y_min = self.y_min().max(other.y_min());
        let x_max = self.x_max().min(other.x_max());
        let y_max = self.y_max().min(other.y_max());

        // Check if the intersection is valid (non-negative width and height)
        if x_min < x_max && y_min < y_max {
            Some(LabelBoundingBox::new_tlbr((x_min, y_min), (x_max, y_max)))
        } else {
            None // No intersection
        }
    }
    pub fn overlap_ratio(&self, other: &Self) -> f64 {
        match self.intersection(other) {
            Some(bb) => bb.area2() / self.area2(),
            None => 0f64,
        }
    }
}

impl PartialEq for LabelBoundingBox {
    fn eq(&self, other: &Self) -> bool {
        self.bbox.min() == other.bbox.min() && self.bbox.max() == other.bbox.max()
    }
}

use std::fmt;

use crate::bbox::BoundingBox;
impl fmt::Display for LabelBoundingBox {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "LabelBoundingBox {{ top_left: ({:.2}, {:.2}), (w,h): ({:.2}, {:.2}) }}",
            self.bbox.min().0,
            self.bbox.min().1,
            self.width(),
            self.height()
        )
    }
}
