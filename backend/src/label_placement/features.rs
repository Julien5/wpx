use std::{collections::HashMap, str::FromStr};

use rstar::{PointDistance, RTree, RTreeObject, AABB};

use crate::{
    bbox::BoundingBox,
    inputpoint::InputPoint,
    label_placement::{labelboundingbox::LabelBoundingBox, stroke},
    math::Point2D,
};

pub type Attributes = HashMap<String, svg::node::Value>;

pub fn set_attr(attr: &mut Attributes, k: &str, v: &str) {
    attr.insert(String::from_str(k).unwrap(), svg::node::Value::from(v));
}

const FONTSIZE: f64 = 16f64;

fn check(ret: &mut f64, s: &str, c: &char, w: &f64) -> bool {
    if s.find(*c).is_some() {
        *ret += w;
        return true;
    }
    false
}

// from https://stackoverflow.com/questions/16007743/roughly-approximate-the-width-of-a-string-of-text-in-python
fn char_width(s: &char) -> f64 {
    let mut size = 0f64;
    if check(&mut size, "lij|\' '", s, &37f64) {
    } else if check(&mut size, "![]fI.,:;/\\t", s, &50f64) {
    } else if check(&mut size, "`-(){}r\"", s, &60f64) {
    } else if check(&mut size, "*^zcsJkvxy", s, &85f64) {
    } else if check(&mut size, "aebdhnopqug#$L+<>=?_~FZT", s, &95f64) {
    } else if check(&mut size, "0123456789", s, &95f64) {
    } else if check(&mut size, "BSPEAKVXY&UwNRCHD", s, &112f64) {
    } else if check(&mut size, "QGOMm%W@", s, &135f64) {
    } else if s.is_uppercase() {
        size += 135f64;
    } else {
        size += 95f64;
    }
    let ret = size * 6f64 / 1000.0;
    (ret / 0.57f64) * (FONTSIZE / 16f64) * 9f64
}

fn text_width(s: &str) -> f64 {
    let mut ret = 0f64;
    for c in s.chars() {
        ret += char_width(&c);
    }
    return ret;
}

#[derive(Clone)]
pub struct PointFeatureDrawing {
    pub group: svg::node::element::Group,
    pub center: Point2D,
}

#[derive(Clone)]
pub struct Label {
    pub id: String,
    pub bbox: LabelBoundingBox,
    pub text: String,
}

impl Label {
    pub fn new() -> Label {
        Label {
            id: String::new(),
            bbox: LabelBoundingBox::zero(),
            text: String::new(),
        }
    }

    pub fn set_text(&mut self, s: &str) {
        self.text = String::from_str(s).unwrap();
        let width = text_width(s);
        self.bbox = LabelBoundingBox::new_relative(
            &BoundingBox::minsize(Point2D::new(0.0, -FONTSIZE), &width, &FONTSIZE),
            &Point2D::zero(),
        );
    }

    pub fn bounding_box(&self) -> LabelBoundingBox {
        self.bbox.clone()
    }

    pub fn to_attributes(&self) -> Attributes {
        let mut ret = Attributes::new();
        let mut x = self.bbox.relative().get_xmin() + 2f64;
        let anchor = if self.bounding_box().relative().get_xmax() < 0f64 {
            "end"
        } else {
            "start"
        };
        if anchor == "end" {
            x = self.bbox.relative().get_xmax() - 2f64;
        }
        set_attr(&mut ret, "text-anchor", anchor);
        let y = self.bbox.relative().get_ymax() - 2f64;
        set_attr(&mut ret, "id", self.id.as_str());
        set_attr(&mut ret, "font-size", format!("{:.1}", FONTSIZE).as_str());
        set_attr(&mut ret, "x", format!("{:.3}", x).as_str());
        set_attr(&mut ret, "y", format!("{:.3}", y).as_str());
        ret
    }
}

pub type PointFeatureId = usize;

#[derive(Clone)]
pub struct PointFeature {
    pub circle: PointFeatureDrawing,
    pub label: Label,
    pub input_point: Option<InputPoint>,
    pub link: Option<svg::node::element::Path>,
    pub id: PointFeatureId,
}

/// Build an RTree spatial index from a vector of PointFeature
pub fn build_pointfeature_rtree(features: &[PointFeature]) -> RTree<PointFeature> {
    RTree::bulk_load(features.to_vec())
}

impl PointFeature {
    pub fn place_label(&mut self, bbox: &LabelBoundingBox) {
        self.label.bbox = bbox.clone();
    }
    pub fn width(&self) -> f64 {
        self.label.bbox.width()
    }
    pub fn height(&self) -> f64 {
        self.label.bbox.height()
    }
    pub fn area(&self) -> f64 {
        return self.width() * self.height();
    }
    pub fn text(&self) -> String {
        self.label.text.clone()
    }
    pub fn center(&self) -> Point2D {
        self.circle.center.clone()
    }
    pub fn input_point(&self) -> Option<InputPoint> {
        self.input_point.clone()
    }
    pub fn _make_link(&mut self, obstacles: &Obstacles) {
        let circle = &self.circle.center;
        let label = self.label.bbox.absolute().project_on_border(circle);
        let to_label = *circle - label;
        let distance = to_label.length();
        if distance < 10f64 {
            return;
        }
        assert!(distance > std::f64::EPSILON);
        let unit = to_label * (1.0 / distance);
        debug_assert!(!unit.x.is_nan());
        debug_assert!(!unit.y.is_nan());
        let epsilon = unit * 2.0f64;
        let from = label + epsilon;
        let to = *circle - epsilon;

        let path = stroke::_compute(&from, &to, obstacles);
        let d = path
            .iter()
            .enumerate()
            .map(|(i, p)| {
                if i == 0 {
                    format!("M{:.2},{:.2}", p.x, p.y)
                } else {
                    format!("L{:.2},{:.2}", p.x, p.y)
                }
            })
            .collect::<Vec<_>>();
        let mut stroke = svg::node::element::Path::new();
        stroke = stroke.set("id", "link");
        stroke = stroke.set("stroke", "black");
        stroke = stroke.set("fill", "transparent");
        stroke = stroke.set("stroke-linejoin", "miter");
        stroke = stroke.set("stroke-miterlimit", "1");
        stroke = stroke.set("d", d);
        self.link = Some(stroke);
    }
    pub fn render_in_group(&self, sd_group: &mut svg::node::element::Group) {
        use svg::Node;
        sd_group.append(self.circle.group.clone());
        let text = format!("{}", self.text());

        let mut subgroup = svg::node::element::Group::new();

        let mut label = svg::node::element::Text::new(text);
        let center = &self.circle.center;
        subgroup = subgroup.set("transform", format!("translate({} {})", center.x, center.y));
        for (k, v) in self.label.to_attributes() {
            label = label.set(k, v);
        }
        let mut whitebg = svg::node::element::Rectangle::new();
        let margin = 2f64;
        whitebg = whitebg.set("x", self.label.bbox.relative().get_xmin() + margin);
        whitebg = whitebg.set("y", self.label.bbox.relative().get_ymin() + margin);
        whitebg = whitebg.set("width", self.label.bbox.relative().width() - 2.0 * margin);
        whitebg = whitebg.set("height", self.label.bbox.relative().height() - 2.0 * margin);
        whitebg = whitebg.set("fill", "white");
        whitebg = whitebg.set("fill-opacity", "0.75");
        whitebg = whitebg.set("id", "label-bg");
        subgroup.append(whitebg);
        subgroup.append(label);
        sd_group.append(subgroup);
        if self.link.is_some() {
            sd_group.append(self.link.as_ref().unwrap().clone());
        }
    }
}

impl PartialEq for PointFeature {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl PartialOrd for PointFeature {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.id.partial_cmp(&other.id)
    }
}

impl Ord for PointFeature {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.id.cmp(&other.id)
    }
}

impl Eq for PointFeature {}

impl RTreeObject for PointFeature {
    type Envelope = AABB<[f64; 2]>;

    fn envelope(&self) -> Self::Envelope {
        let c = self.center();
        AABB::from_point([c.x, c.y])
    }
}

impl PointDistance for PointFeature {
    fn distance_2(&self, point: &[f64; 2]) -> f64 {
        let c = self.center();
        let dx = c.x - point[0];
        let dy = c.y - point[1];
        dx * dx + dy * dy
    }
}

#[derive(Clone)]
pub struct Polyline {
    id: String,
    pub points: Vec<Point2D>,
}

impl Polyline {
    pub fn new() -> Polyline {
        Polyline {
            id: "track".to_string(),
            points: Vec::<Point2D>::new(),
        }
    }
}

impl Polyline {
    pub fn _from_attributes(a: &Attributes) -> Polyline {
        let data = a.get("d").unwrap();
        let mut points = Vec::new();
        for tok in data.split(" ") {
            let t: Vec<&str> = tok.split(",").collect();
            debug_assert!(t.len() == 2);
            let x = format!("{}", t[0].get(1..).unwrap())
                .parse::<f64>()
                .unwrap();
            let y = format!("{}", t[1]).parse::<f64>().unwrap();
            points.push(Point2D::new(x, y));
        }
        Polyline {
            id: format!("{}", a.get("id").unwrap()),
            points,
        }
    }

    pub fn to_attributes(&self) -> Attributes {
        let mut ret = Attributes::new();
        let mut dv = Vec::new();
        for p in &self.points {
            if dv.is_empty() {
                dv.push(format!("M{:.1},{:.1}", p.x, p.y));
            } else {
                dv.push(format!("L{:.1},{:.1}", p.x, p.y));
            }
        }
        let d = dv.join(" ");
        set_attr(&mut ret, "id", self.id.as_str());
        set_attr(&mut ret, "fill", "transparent");
        set_attr(&mut ret, "stroke-width", "2");
        set_attr(&mut ret, "stroke", "black");
        set_attr(&mut ret, "stroke-linejoin", "miter");
        set_attr(&mut ret, "stroke-miterlimit", "1");
        set_attr(&mut ret, "d", d.as_str());
        ret
    }
}

#[derive(Clone)]
pub struct DrawingArea {
    pub bbox: BoundingBox,
    pub max_area_ratio: f64,
}

#[derive(Clone)]
pub struct Obstacles {
    pub bboxes: Vec<BoundingBox>,
    pub polylines: Vec<Polyline>,
    pub drawingbox: DrawingArea,
}

impl Obstacles {
    pub fn new() -> Obstacles {
        Obstacles {
            bboxes: Vec::new(),
            polylines: Vec::new(),
            drawingbox: DrawingArea {
                bbox: BoundingBox::new(),
                max_area_ratio: 0f64,
            },
        }
    }

    pub fn _is_clear(&self, p1: &Point2D, p2: &Point2D) -> bool {
        for bbox in &self.bboxes {
            if bbox.segment_intersects(p1, p2) {
                return false;
            }
        }
        true
    }

    pub fn available_area(&self) -> f64 {
        self.drawingbox.bbox.area() * self.drawingbox.max_area_ratio
            - self.bboxes.iter().map(|bbox| bbox.area()).sum::<f64>()
    }
}
