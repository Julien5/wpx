#[derive(Clone, Debug)]
pub struct LabelBoundingBox {
    relativebbox: BoundingBox,
    target: Point2D,
    text_anchor: Option<String>,
}

impl LabelBoundingBox {
    pub fn zero() -> Self {
        LabelBoundingBox {
            relativebbox: BoundingBox::new(),
            target: Point2D::zero(),
            text_anchor: None,
        }
    }
    pub fn new_relative(bbox: &BoundingBox, target: &Point2D) -> Self {
        LabelBoundingBox {
            relativebbox: bbox.clone(),
            target: target.clone(),
            text_anchor: None,
        }
    }
    pub fn new_absolute(absolutebbox: &BoundingBox, target: &Point2D) -> Self {
        let relative = absolutebbox.make_translate(&(*target * (-1f64)));
        LabelBoundingBox {
            relativebbox: relative,
            target: target.clone(),
            text_anchor: None,
        }
    }
    pub fn with_text_anchor(mut self, anchor: &str) -> Self {
        self.text_anchor = Some(anchor.to_string());
        self
    }
    pub fn text_anchor(&self) -> &Option<String> {
        &self.text_anchor
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
        let a1 = self.absolute();
        let a2 = other.absolute();
        a1.get_min() == a2.get_min() && a1.get_max() == a2.get_max()
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
