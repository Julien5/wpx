#![allow(non_snake_case)]

use std::collections::HashMap;
use std::fmt;
use std::str::FromStr;

use crate::utm::UTMPoint;
use crate::{gpsdata, waypoints_table};
use crate::{segment, waypoint};

use svg::node::element::path::{Command, Data};
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

trait SvgElement {
    fn from_attributes(a: &Attributes) -> Self
    where
        Self: Sized;
    fn to_attributes(&self) -> Attributes;
}

type Attributes = HashMap<String, svg::node::Value>;

struct Circle {
    id: String,
    cx: f64,
    cy: f64,
    r: f64,
    fill: Option<String>,
}

impl Circle {
    fn new() -> Circle {
        Circle {
            id: String::new(),
            cx: 0f64,
            cy: 0f64,
            r: 0f64,
            fill: None,
        }
    }
}

struct Label {
    id: String,
    x: f64,
    y: f64,
    text: String,
    text_anchor: String,
}

impl Label {
    fn new() -> Label {
        Label {
            id: String::new(),
            x: 0f64,
            y: 0f64,
            text: String::new(),
            text_anchor: "start".to_string(),
        }
    }

    fn bounding_box(&self) -> LabelBoundingBox {
        let width = self.text.len() as f64 * 10.0; // 10 pixels per character
        let height = 16.0; // Assuming a fixed height of 16 pixels for the font size

        let (top_left, bottom_right) = match self.text_anchor.as_str() {
            "end" => (
                (self.x - width, self.y - height), // Adjust for right alignment
                (self.x, self.y),
            ),
            _ => ((self.x, self.y - height), (self.x + width, self.y)),
        };

        LabelBoundingBox::new(top_left, bottom_right, &self.text_anchor)
    }
}

struct LabelBoundingBox {
    top_left: (f64, f64),
    bottom_right: (f64, f64),
}

fn offset(p: &(f64, f64), d: (f64, f64)) -> (f64, f64) {
    (p.0 + d.0, p.1 + d.1)
}

impl LabelBoundingBox {
    fn new(top_left: (f64, f64), bottom_right: (f64, f64), anchor: &String) -> Self {
        let eps = match anchor.as_str() {
            "end" => (2f64, 2f64),
            _ => (-2f64, 2f64),
        };
        LabelBoundingBox {
            top_left: offset(&top_left, eps),
            bottom_right: offset(&bottom_right, eps),
        }
    }

    fn x_min(&self) -> f64 {
        self.top_left.0
    }

    fn y_min(&self) -> f64 {
        self.top_left.1
    }

    fn x_max(&self) -> f64 {
        self.bottom_right.0
    }

    fn y_max(&self) -> f64 {
        self.bottom_right.1
    }

    fn width(&self) -> f64 {
        self.x_max() - self.x_min()
    }

    fn height(&self) -> f64 {
        self.y_max() - self.y_min()
    }
}

impl fmt::Display for LabelBoundingBox {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "LabelBoundingBox {{ top_left: ({:.0}, {:.0}), bottom_right: ({:.0}, {:.0}) }}",
            self.top_left.0, self.top_left.1, self.bottom_right.0, self.bottom_right.1
        )
    }
}

struct Point {
    id: String,
    circle: Circle,
    label: Label,
}

impl Point {
    fn new() -> Point {
        Point {
            id: String::new(),
            circle: Circle::new(),
            label: Label::new(),
        }
    }
}

struct Polyline {
    id: String,
    points: Vec<(f64, f64)>,
}

impl Polyline {
    fn new() -> Polyline {
        Polyline {
            id: "track".to_string(),
            points: Vec::<(f64, f64)>::new(),
        }
    }
}

impl SvgElement for Circle {
    fn from_attributes(a: &Attributes) -> Circle {
        let fill = match a.get("fill") {
            Some(value) => Some(value.to_string()),
            _ => None,
        };

        Circle {
            id: a.get("id").unwrap().to_string(),
            cx: a.get("cx").unwrap().to_string().parse::<f64>().unwrap(),
            cy: a.get("cy").unwrap().to_string().parse::<f64>().unwrap(),
            r: a.get("r").unwrap().to_string().parse::<f64>().unwrap(),
            fill,
        }
    }

    fn to_attributes(&self) -> Attributes {
        let mut ret = Attributes::new();
        set_attr(&mut ret, "id", self.id.as_str());
        set_attr(&mut ret, "cx", format!("{}", self.cx).as_str());
        set_attr(&mut ret, "cy", format!("{}", self.cy).as_str());
        set_attr(&mut ret, "r", format!("{}", self.r).as_str());
        ret
    }
}

impl SvgElement for Label {
    fn from_attributes(a: &Attributes) -> Label {
        Label {
            id: a.get("id").unwrap().to_string(),
            x: a.get("x").unwrap().to_string().parse::<f64>().unwrap(),
            y: a.get("y").unwrap().to_string().parse::<f64>().unwrap(),
            text: String::new(),
            text_anchor: "start".to_string(),
        }
    }

    fn to_attributes(&self) -> Attributes {
        let mut ret = Attributes::new();
        set_attr(&mut ret, "id", self.id.as_str());
        set_attr(&mut ret, "text-anchor", self.text_anchor.as_str());
        set_attr(&mut ret, "font-size", "16");
        set_attr(&mut ret, "x", format!("{}", self.x).as_str());
        set_attr(&mut ret, "y", format!("{}", self.y).as_str());
        ret
    }
}

impl SvgElement for Polyline {
    fn from_attributes(a: &Attributes) -> Polyline {
        let data = a.get("d").unwrap();
        let mut points = Vec::new();
        for tok in data.split(" ") {
            let t: Vec<&str> = tok.split(",").collect();
            debug_assert!(t.len() == 2);
            let x = format!("{}", t[0].get(1..).unwrap())
                .parse::<f64>()
                .unwrap();
            let y = format!("{}", t[1]).parse::<f64>().unwrap();
            points.push((x, y));
        }
        Polyline {
            id: format!("{}", a.get("id").unwrap()),
            points,
        }
    }

    fn to_attributes(&self) -> Attributes {
        let mut ret = Attributes::new();
        let mut dv = Vec::new();
        for (x, y) in &self.points {
            if dv.is_empty() {
                dv.push(format!("M{x:.1},{y:.1}"));
            } else {
                dv.push(format!("L{x:.1},{y:.1}"));
            }
        }
        let d = dv.join(" ");
        set_attr(&mut ret, "id", self.id.as_str());
        set_attr(&mut ret, "fill", "transparent");
        set_attr(&mut ret, "stroke-width", "3");
        set_attr(&mut ret, "stroke", "black");
        set_attr(&mut ret, "d", d.as_str());
        ret
    }
}

fn polyline_hits_label(polyline: &Polyline, label: &Label) -> bool {
    let bbox = label.bounding_box();

    for &(x, y) in &polyline.points {
        if x >= bbox.x_min() && x <= bbox.x_max() && y >= bbox.y_min() && y <= bbox.y_max() {
            return true;
        }
    }

    false
}

pub struct Map {
    polyline: Polyline,
    points: Vec<Point>,
    document: Attributes,
}

fn readid(id: &str) -> (&str, &str) {
    id.split_once("/").unwrap()
}

fn set_attr(attr: &mut Attributes, k: &str, v: &str) {
    attr.insert(String::from_str(k).unwrap(), svg::node::Value::from(v));
}

fn candidates() -> Vec<(f64, f64, String)> {
    let mut ret = Vec::new();
    ret.push((0f64, 0f64, "start".to_string()));
    ret.push((0f64, 0f64, "end".to_string()));
    ret.push((0f64, 16f64, "end".to_string()));
    ret.push((0f64, 16f64, "start".to_string()));
    ret
}

fn place_label(point: &mut Point, polyline: &Polyline) {
    let label = &mut point.label;
    for c in candidates() {
        let (dx, dy, anchor) = c;
        label.x = point.circle.cx + dx;
        label.y = point.circle.cy + dy;
        label.text_anchor = anchor;
        println!(
            "[{}][x={:.1},y={:.1}][a={}] bb={}",
            label.text,
            label.x,
            label.y,
            label.text_anchor,
            label.bounding_box()
        );
        if !polyline_hits_label(polyline, label) {
            println!("OK");
            return;
        }
    }
    println!("FAIL");
}

impl Map {
    pub fn make(
        geodata: &gpsdata::Track,
        waypoints: &Vec<waypoint::Waypoint>,
        segment: &segment::Segment,
        W: i32,
        H: i32,
        _debug: bool,
    ) -> Map {
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
            let mut svgPoint = Point::new();
            let (x, y) = bbox.to_graphics_coordinates(&w.utm, W, H);
            svgPoint.circle.id = format!("wp-{}/circle", k);
            svgPoint.circle.cx = x;
            svgPoint.circle.cy = y;
            svgPoint.circle.r = 4f64;
            if V.contains(&k) {
                let label = w.info.as_ref().unwrap().profile_label();
                svgPoint.label.text = String::from_str(label.trim()).unwrap();
                svgPoint.label.id = format!("wp-{}/text", k);
                place_label(&mut svgPoint, &polyline);
            } else {
                svgPoint.circle.fill = Some(String::from_str("blue").unwrap());
            }
            points.push(svgPoint);
        }
        Map {
            polyline,
            points,
            document,
        }
    }
    pub fn import(filename: std::path::PathBuf) -> Map {
        use svg::node::element::*;
        use svg::parser::Event;
        let mut polyline = crate::svgmap::Polyline::new();
        let mut document = Attributes::new();
        let mut content = String::new();
        let mut points = Vec::new();
        let mut current_circle = Point::new();
        for event in svg::open(filename, &mut content).unwrap() {
            match event {
                Event::Tag(tag::Circle, _, attributes) => {
                    if attributes.contains_key("id") {
                        let id = attributes.get("id").unwrap().clone().to_string();
                        let (p_id, _p_attr) = readid(id.as_str());
                        current_circle.id = String::from_str(p_id).unwrap();
                        current_circle.circle = crate::svgmap::Circle::from_attributes(&attributes);
                        println!("{}: {:?}", id, attributes);
                    }
                }
                Event::Tag(tag::Text, _, attributes) => {
                    if attributes.contains_key("id") {
                        let id = attributes.get("id").unwrap();
                        current_circle.label = Label::from_attributes(&attributes);
                        println!("{}: {:?}", id, attributes);
                    }
                }
                Event::Text(data) => {
                    println!("Event::Text {:?}", data);
                    current_circle.label.text = String::from_str(data).unwrap();
                    debug_assert!(!current_circle.id.is_empty());
                    points.push(current_circle);
                    current_circle = Point::new();
                }
                Event::Tag(tag::Path, _, attributes) => {
                    if attributes.contains_key("id") {
                        let id = attributes.get("id").unwrap();
                        println!("{}: {:?} attributes", id, attributes.len());
                    }
                    polyline = crate::svgmap::Polyline::from_attributes(&attributes);
                    let data = attributes.get("d").unwrap();
                    let data = Data::parse(data).unwrap();
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
        }
    }

    pub fn render(self) -> String {
        let mut document = Document::new();
        for (k, v) in self.document {
            println!("export {} -> {}", k, v);
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

            let mut label = svg::node::element::Text::new(point.label.text.clone());
            for (k, v) in point.label.to_attributes() {
                label = label.set(k, v);
            }
            document = document.add(label);
            let mut debug_bb = svg::node::element::Rectangle::new();
            let bb = point.label.bounding_box();
            debug_bb = debug_bb.set("x", bb.x_min());
            debug_bb = debug_bb.set("y", bb.y_min());
            debug_bb = debug_bb.set("width", bb.width());
            debug_bb = debug_bb.set("height", bb.height());
            debug_bb = debug_bb.set("fill", "transparent");
            debug_bb = debug_bb.set("stroke-width", "2");
            debug_bb = debug_bb.set("stroke", "blue");
            document = document.add(debug_bb);
        }
        document.to_string()
    }
}

pub fn map(
    geodata: &gpsdata::Track,
    waypoints: &Vec<waypoint::Waypoint>,
    segment: &segment::Segment,
    W: i32,
    H: i32,
    debug: bool,
) -> String {
    let svgMap = Map::make(geodata, waypoints, segment, W, H, debug);
    svgMap.render()
}
