#![allow(non_snake_case)]

use crate::utm::UTMPoint;
use crate::{gpsdata, waypoints_table};
use crate::{segment, waypoint};

use svg::node::element::path::{Command, Data, Position};
use svg::node::element::Path;
use svg::Document;

struct BoundingBox {
    min: (f64, f64),
    max: (f64, f64),
}

impl BoundingBox {
    fn new() -> BoundingBox {
        let min = (f64::MAX, f64::MAX);
        let max = (f64::MIN, f64::MIN);
        BoundingBox { min, max }
    }
    fn width(&self) -> f64 {
        return self.max.0 - self.min.0;
    }
    fn height(&self) -> f64 {
        return self.max.1 - self.min.1;
    }
    fn update(&mut self, p: &UTMPoint) {
        if p.x() > self.max.0 {
            self.max.0 = p.x();
        }
        if p.y() > self.max.1 {
            self.max.1 = p.y();
        }
        if p.x() < self.min.0 {
            self.min.0 = p.x();
        }
        if p.y() < self.min.1 {
            self.min.1 = p.y();
        }
    }
    fn fix_aspect_ratio(&mut self, W: i32, H: i32) {
        let X = (self.min.0 + self.max.0) / 2f64;
        let Y = (self.min.1 + self.max.1) / 2f64;
        if self.height() > self.width() {
            let delta = 0.5f64 * (self.height());
            self.max.0 = X + delta;
            self.min.0 = X - delta;
        } else {
            let delta = 0.5f64 * self.width();
            self.max.1 = Y + delta;
            self.min.1 = Y - delta;
        }
        let margin = 2000f64;
        self.max.0 = self.max.0 + margin;
        self.max.1 = self.max.1 + margin;
        self.min.0 = self.min.0 - margin;
        self.min.1 = self.min.1 - margin;
    }
    fn to_graphics_coordinates(&self, p: &UTMPoint, W: i32, H: i32) -> (f64, f64) {
        let xmin = self.min.0;
        let xmax = self.max.0;
        let ymin = self.min.1;
        let ymax = self.max.1;

        let f = |x: f64| -> f64 {
            let a = W as f64 / (xmax - xmin);
            let b = -a * xmin;
            a * x + b
        };
        let g = |y: f64| -> f64 {
            let a = H as f64 / (ymin - ymax);
            let b = -a * ymax;
            a * y + b
        };
        (f(p.x()), g(p.y()))
    }
    fn contains(&self, p: &UTMPoint) -> bool {
        if p.x() < self.min.0 {
            return false;
        }
        if p.x() > self.max.0 {
            return false;
        }
        if p.y() < self.min.1 {
            return false;
        }
        if p.y() > self.max.1 {
            return false;
        }
        return true;
    }
}

pub fn map(
    geodata: &gpsdata::Track,
    waypoints: &Vec<waypoint::Waypoint>,
    segment: &segment::Segment,
    W: i32,
    H: i32,
) -> String {
    let mut data = Data::new();
    let path = &geodata.utm;
    let mut bbox = BoundingBox::new();
    let range = &segment.range;
    for k in range.start..range.end {
        bbox.update(&geodata.utm[k]);
    }
    bbox.fix_aspect_ratio(W, H);
    // todo: path in the bbox, which more than the path in the range.
    for k in range.start..range.end {
        let p = &path[k];
        let (xg, yg) = bbox.to_graphics_coordinates(p, W, H);
        if data.is_empty() {
            data.append(Command::Move(Position::Absolute, (xg, yg).into()));
        }
        data.append(Command::Line(Position::Absolute, (xg, yg).into()));
    }

    let svgpath = Path::new()
        .set("fill", "none")
        .set("stroke", "black")
        .set("stroke-width", 2)
        .set("d", data);

    let mut document = Document::new().set("viewBox", (0, 0, W, H)).add(svgpath);

    let V = waypoints_table::show_waypoints_in_table(&waypoints, &segment.profile.bbox);
    println!("V={:?}", V);

    for k in 0..waypoints.len() {
        let w = &waypoints[k];
        let index = w.get_track_index();
        if !bbox.contains(&w.utm) {
            continue;
        }
        let (x, y) = bbox.to_graphics_coordinates(&w.utm, W, H);
        if V.contains(&k) {
            let dot = svg::node::element::Circle::new()
                .set("cx", x)
                .set("cy", y)
                .set("r", 4);
            document = document.add(dot);
            let text = svg::node::element::Text::new(w.info.as_ref().unwrap().profile_label())
                .set("text-anchor", "left")
                .set("font-size", "16")
                .set("x", x + 4f64)
                .set("y", y + 4.5f64);
            document = document.add(text);
        } else {
            let dot = svg::node::element::Circle::new()
                .set("cx", x)
                .set("cy", y)
                .set("fill", "blue")
                .set("stroke", "black")
                .set("stroke-width", "2")
                .set("r", 3);
            document = document.add(dot);
        }
    }
    document.to_string()
}
