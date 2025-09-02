#![allow(non_snake_case)]

use std::str::FromStr;

use crate::label_placement::Circle;
use crate::label_placement::Label;
use crate::label_placement::Polyline;
use crate::segment;
use crate::utm::UTMPoint;
use crate::{backend, waypoints_table};

use svg::node::element::path::Data;
use svg::Document;

struct UTMBoundingBox {
    min: (f64, f64),
    max: (f64, f64),
}

impl UTMBoundingBox {
    fn new() -> UTMBoundingBox {
        let min = (f64::MAX, f64::MAX);
        let max = (f64::MIN, f64::MIN);
        UTMBoundingBox { min, max }
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
    // TODO: take WxH into account
    fn fix_aspect_ratio(&mut self, _W: i32, _H: i32) {
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

fn readid(id: &str) -> (&str, &str) {
    id.split_once("/").unwrap()
}

use crate::label_placement::set_attr;
use crate::label_placement::Attributes;
use crate::label_placement::PointFeature;

pub struct Map {
    polyline: Polyline,
    points: Vec<PointFeature>,
    document: Attributes,
    debug: svg::node::element::Group,
}

impl Map {
    pub fn make(
        backend: &backend::Backend,
        segment: &segment::Segment,
        W: i32,
        H: i32,
        _debug: bool,
    ) -> Map {
        let geodata = &backend.track;
        let waypoints = backend.get_waypoints();
        let path = &geodata.utm;
        let mut bbox = UTMBoundingBox::new();
        let range = &segment.range;
        for k in range.start..range.end {
            bbox.update(&geodata.utm[k]);
        }
        bbox.fix_aspect_ratio(W, H);
        let mut polyline = Polyline::new();
        // todo: path in the bbox, which more than the path in the range.
        for k in range.start..range.end {
            let p = &path[k];
            let (xg, yg) = bbox.to_graphics_coordinates(p, W, H);
            polyline.points.push((xg, yg));
        }

        let mut document = Attributes::new();
        set_attr(
            &mut document,
            "viewBox",
            format!("(0, 0, {W}, {H})").as_str(),
        );
        set_attr(&mut document, "width", format!("{W}").as_str());
        set_attr(&mut document, "height", format!("{H}").as_str());

        let V = waypoints_table::show_waypoints_in_table(&waypoints, &segment.profile.bbox);
        let mut points = Vec::new();
        for k in 0..waypoints.len() {
            let w = &waypoints[k];
            if !bbox.contains(&w.utm) {
                continue;
            }
            let mut svgPoint = PointFeature::new();
            let (x, y) = bbox.to_graphics_coordinates(&w.utm, W, H);
            svgPoint.circle.id = format!("wp-{}/circle", k);
            svgPoint.circle.cx = x;
            svgPoint.circle.cy = y;
            svgPoint.id = format!("wp-{}", k);
            if V.contains(&k) {
                let label = w.info.as_ref().unwrap().profile_label();
                svgPoint.label.set_text(label.trim());
                svgPoint.label.id = format!("wp-{}/text", k);
            } else {
                svgPoint.circle.fill = Some(String::from_str("blue").unwrap());
            }
            points.push(svgPoint);
        }
        let debug = crate::label_placement::place_labels(&mut points, &polyline);
        Map {
            polyline,
            points,
            document,
            debug,
        }
    }
    fn _import(filename: std::path::PathBuf) -> Map {
        use svg::node::element::tag;
        use svg::parser::Event;
        let mut polyline = crate::svgmap::Polyline::new();
        let mut document = Attributes::new();
        let mut content = String::new();
        let mut points = Vec::new();
        let mut current_circle = PointFeature::new();
        let mut current_text_attributes = Attributes::new();
        for event in svg::open(filename, &mut content).unwrap() {
            match event {
                Event::Tag(tag::Circle, _, attributes) => {
                    if attributes.contains_key("id") {
                        let id = attributes.get("id").unwrap().clone().to_string();
                        let (p_id, _p_attr) = readid(id.as_str());
                        current_circle.id = String::from_str(p_id).unwrap();
                        current_circle.circle = Circle::_from_attributes(&attributes);
                        println!("{}: {:?}", id, attributes);
                    }
                }
                Event::Tag(tag::Text, _, attributes) => {
                    if attributes.contains_key("id") {
                        let id = attributes.get("id").unwrap();
                        current_text_attributes = attributes.clone();
                        println!("{}: {:?}", id, attributes);
                    }
                }
                Event::Text(data) => {
                    println!("Event::Text {:?}", data);
                    current_circle.label = Label::_from_attributes(&current_text_attributes, data);
                    current_text_attributes.clear();
                    debug_assert!(!current_circle.id.is_empty());
                    points.push(current_circle);
                    current_circle = PointFeature::new();
                }
                Event::Tag(tag::Path, _, attributes) => {
                    if attributes.contains_key("id") {
                        let id = attributes.get("id").unwrap();
                        println!("{}: {:?} attributes", id, attributes.len());
                    }
                    polyline = crate::svgmap::Polyline::_from_attributes(&attributes);
                    let data = attributes.get("d").unwrap();
                    let data = Data::parse(data).unwrap();
                    use svg::node::element::path::Command;
                    for command in data.iter() {
                        match command {
                            &Command::Move(..) => { /* … */ }
                            &Command::Line(..) => { /* … */ }
                            _ => {}
                        }
                    }
                }
                Event::Tag(tag::SVG, _, attributes) => {
                    if !attributes.is_empty() {
                        document = attributes.clone();
                    }
                }
                _ => {
                    println!("event {:?}", event);
                }
            }
        }

        Map {
            polyline,
            points,
            document,
            debug: svg::node::element::Group::new(),
        }
    }

    pub fn render(self) -> String {
        let mut document = Document::new();
        for (k, v) in self.document {
            // println!("export {} -> {}", k, v);
            document = document.set(k, v);
        }

        let mut svgpath = svg::node::element::Path::new();
        for (k, v) in self.polyline.to_attributes() {
            svgpath = svgpath.set(k, v);
        }
        document = document.add(svgpath);

        for point in self.points {
            let mut circle = svg::node::element::Circle::new();
            for (k, v) in point.circle.to_attributes() {
                circle = circle.set(k, v);
            }
            document = document.add(circle);
            let text = format!("{}", point.label.text);
            let mut label = svg::node::element::Text::new(text);
            for (k, v) in point.label.to_attributes(point.circle.cx) {
                label = label.set(k, v);
            }

            document = document.add(label);
            /*let mut debug_bb = svg::node::element::Rectangle::new();
            let bb = point.label.bounding_box();
            debug_bb = debug_bb.set("x", bb.x_min());
            debug_bb = debug_bb.set("y", bb.y_min());
            debug_bb = debug_bb.set("width", bb.width());
            debug_bb = debug_bb.set("height", bb.height());
            debug_bb = debug_bb.set("fill", "transparent");
            debug_bb = debug_bb.set("stroke-width", "1");
            debug_bb = debug_bb.set("stroke", "blue");
            document = document.add(debug_bb);
            */
        }
        document = document.add(self.debug);
        document.to_string()
    }
}

pub fn map(
    backend: &backend::Backend,
    segment: &segment::Segment,
    W: i32,
    H: i32,
    debug: bool,
) -> String {
    let svgMap = Map::make(backend, segment, W, H, debug);
    svgMap.render()
}
