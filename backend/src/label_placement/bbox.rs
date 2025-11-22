#[derive(Clone)]
pub struct LabelBoundingBox {
    bbox: BoundingBox,
    target: Point2D,
}

impl LabelBoundingBox {
    pub fn zero() -> Self {
        LabelBoundingBox {
            bbox: BoundingBox::new(),
            target: Point2D::zero(),
        }
    }
    pub fn new_relative(bbox: &BoundingBox, target: &Point2D) -> Self {
        LabelBoundingBox {
            bbox: bbox.clone(),
            target: target.clone(),
        }
    }
    pub fn new_absolute(absolutebbox: &BoundingBox, target: &Point2D) -> Self {
        let relative = absolutebbox.make_translate(&(*target * (-1f64)));
        LabelBoundingBox {
            bbox: relative,
            target: target.clone(),
        }
    }
    pub fn area(&self) -> f64 {
        self.bbox.area()
    }
    pub fn relative(&self) -> &BoundingBox {
        &self.bbox
    }
    pub fn absolute(&self) -> BoundingBox {
        let mut ret = self.bbox.clone();
        ret.translate(&self.target);
        ret
    }
    pub fn width(&self) -> f64 {
        self.bbox.width()
    }

    pub fn height(&self) -> f64 {
        self.bbox.height()
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
