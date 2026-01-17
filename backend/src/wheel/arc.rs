#![allow(dead_code)]

use crate::{math::Point2D, wheel::model};

pub struct Arc {
    pub center: Point2D,
    pub radius: f64,
    pub angle1: f64,
    pub angle2: f64,
}

impl Arc {
    pub fn from_model(center: &Point2D, radius: f64, m: &model::Arc) -> Self {
        Arc {
            center: center.clone(),
            radius,
            angle1: m.start_angle,
            angle2: m.end_angle,
        }
    }
    fn position(&self, radius: f64, angle: f64) -> Point2D {
        let x = self.center.x as f64 + radius * (angle.to_radians()).sin();
        let y = self.center.y as f64 - radius * (angle.to_radians()).cos();
        Point2D::new(x, y)
    }
    fn start(&self) -> Point2D {
        self.position(self.radius, self.angle1)
    }
    fn raw_arc(&self, a1: f64, a2: f64) -> String {
        let p2 = self.position(self.radius, a2);
        let flag = if a2 - a1 > 180f64 { 1 } else { 0 };
        format!(
            "A {} {} 0 {} 1 {} {}",
            self.radius, self.radius, flag, p2.x, p2.y
        )
    }
    fn arc(&self, a1: f64, a2: f64) -> String {
        let p1 = self.position(self.radius, a1);
        format!("M {} {} {}", p1.x, p1.y, self.raw_arc(a1, a2))
    }
    pub fn closed_path(&self) -> String {
        let p1 = self.start();
        format!(
            "M {} {} L {} {} {} Z",
            self.center.x,
            self.center.y,
            p1.x,
            p1.y,
            self.raw_arc(self.angle1, self.angle2)
        )
    }

    pub fn mid_angle(&self) -> f64 {
        (self.angle1 + self.angle2) * 0.5
    }

    pub fn dash(&self, radius1: f64, radius2: f64, angle: f64) -> String {
        let p1 = self.position(radius1, angle);
        let p2 = self.position(radius2, angle);
        format!("M {} {} L {} {}", p1.x, p1.y, p2.x, p2.y,)
    }
    pub fn open_path(&self) -> String {
        let p1 = self.start();
        format!("M {} {} {}", p1.x, p1.y, self.arc(self.angle1, self.angle2))
    }
}
