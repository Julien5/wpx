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
    pub fn _new_tlbr(top_left: Point2D, bottom_right: Point2D) -> Self {
        LabelBoundingBox {
            bbox: BoundingBox::init(top_left, bottom_right),
        }
    }

    pub fn new_tlwh(top_left: Point2D, width: f64, height: f64) -> Self {
        let bottom_right = Point2D::new(top_left.x + width, top_left.y + height);
        LabelBoundingBox {
            bbox: BoundingBox::init(top_left, bottom_right),
        }
    }

    pub fn new_blwh(bottom_left: Point2D, width: f64, height: f64) -> Self {
        let top_left = Point2D::new(bottom_left.x, bottom_left.y - height);
        let bottom_right = Point2D::new(bottom_left.x + width, bottom_left.y);
        LabelBoundingBox {
            bbox: BoundingBox::init(top_left, bottom_right),
        }
    }

    pub fn new_brwh(bottom_right: Point2D, width: f64, height: f64) -> Self {
        let top_left = Point2D::new(bottom_right.x - width, bottom_right.y - height);
        LabelBoundingBox {
            bbox: BoundingBox::init(top_left, bottom_right),
        }
    }

    pub fn new_trwh(top_right: Point2D, width: f64, height: f64) -> Self {
        let top_left = Point2D::new(top_right.x - width, top_right.y);
        let bottom_right = Point2D::new(top_right.x, top_right.y + height);
        LabelBoundingBox {
            bbox: BoundingBox::init(top_left, bottom_right),
        }
    }

    pub fn x_min(&self) -> f64 {
        self.bbox.get_min().x
    }

    pub fn y_min(&self) -> f64 {
        self.bbox.get_min().y
    }

    fn _bottom_left(&self) -> Point2D {
        Point2D::new(self.x_min(), self.y_max())
    }

    fn _top_right(&self) -> Point2D {
        Point2D::new(self.x_max(), self.y_min())
    }

    pub fn _center(&self) -> Point2D {
        Point2D::new(
            0.5 * (self.x_min() + self.x_max()),
            0.5 * (self.y_min() + self.y_max()),
        )
    }

    pub fn x_max(&self) -> f64 {
        self.bbox.get_max().x
    }

    pub fn y_max(&self) -> f64 {
        self.bbox.get_max().y
    }

    pub fn width(&self) -> f64 {
        self.x_max() - self.x_min()
    }

    pub fn height(&self) -> f64 {
        self.y_max() - self.y_min()
    }
    fn _area2(&self) -> f64 {
        let dx = self.x_max() - self.x_min();
        let dy = self.y_max() - self.y_min();
        return dx * dy;
    }
    fn _intersection(&self, other: &Self) -> Option<LabelBoundingBox> {
        let x_min = self.x_min().max(other.x_min());
        let y_min = self.y_min().max(other.y_min());
        let x_max = self.x_max().min(other.x_max());
        let y_max = self.y_max().min(other.y_max());

        // Check if the intersection is valid (non-negative width and height)
        if x_min < x_max && y_min < y_max {
            Some(LabelBoundingBox::_new_tlbr(
                Point2D::new(x_min, y_min),
                Point2D::new(x_max, y_max),
            ))
        } else {
            None // No intersection
        }
    }
}

impl PartialEq for LabelBoundingBox {
    fn eq(&self, other: &Self) -> bool {
        self.bbox.get_min() == other.bbox.get_min() && self.bbox.get_max() == other.bbox.get_max()
    }
}

use std::fmt;

use crate::{bbox::BoundingBox, math::Point2D};
impl fmt::Display for LabelBoundingBox {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "LabelBoundingBox {{ top_left: ({:.2}, {:.2}), (w,h): ({:.2}, {:.2}) }}",
            self.bbox.get_min().x,
            self.bbox.get_min().y,
            self.width(),
            self.height()
        )
    }
}
