#[derive(Clone)]
pub struct LabelBoundingBox {
    relativebbox: BoundingBox,
    target: Point2D,
}

impl LabelBoundingBox {
    pub fn zero() -> Self {
        LabelBoundingBox {
            relativebbox: BoundingBox::new(),
            target: Point2D::zero(),
        }
    }
    pub fn new_relative(bbox: &BoundingBox, target: &Point2D) -> Self {
        LabelBoundingBox {
            relativebbox: bbox.clone(),
            target: target.clone(),
        }
    }
    pub fn new_absolute(absolutebbox: &BoundingBox, target: &Point2D) -> Self {
        let relative = absolutebbox.make_translate(&(*target * (-1f64)));
        LabelBoundingBox {
            relativebbox: relative,
            target: target.clone(),
        }
    }
    pub fn area(&self) -> f64 {
        self.relativebbox.area()
    }
    pub fn relative(&self) -> &BoundingBox {
        &self.relativebbox
    }
    pub fn absolute(&self) -> BoundingBox {
        let mut ret = self.relativebbox.clone();
        ret.translate(&self.target);
        ret
    }
    pub fn width(&self) -> f64 {
        self.relativebbox.width()
    }

    pub fn height(&self) -> f64 {
        self.relativebbox.height()
    }
}

impl PartialEq for LabelBoundingBox {
    fn eq(&self, other: &Self) -> bool {
        self.relativebbox.get_min() == other.relativebbox.get_min()
            && self.relativebbox.get_max() == other.relativebbox.get_max()
    }
}

use std::fmt;

use crate::{bbox::BoundingBox, math::Point2D};
impl fmt::Display for LabelBoundingBox {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "LabelBoundingBox {{ top_left: ({:.2}, {:.2}), (w,h): ({:.2}, {:.2}) }}",
            self.relativebbox.get_min().x,
            self.relativebbox.get_min().y,
            self.width(),
            self.height()
        )
    }
}
