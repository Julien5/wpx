pub mod bbox;
pub mod candidate;
pub mod drawings;
mod graph;
pub mod prioritize;
mod stroke;
pub use bbox::LabelBoundingBox;
use candidate::Candidate;
use candidate::Candidates;
use graph::Graph;
use svg::Node;

use std::collections::BTreeMap;
use std::collections::HashMap;
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
        self.bbox = LabelBoundingBox::new_blwh(Point2D::new(0f64, 0f64), width, FONTSIZE);
    }

    pub fn bounding_box(&self) -> LabelBoundingBox {
        self.bbox.clone()
    }
}

#[derive(Clone)]
pub struct PointFeatureDrawing {
    pub group: svg::node::element::Group,
    pub center: Point2D,
}

#[derive(Clone)]
pub struct PointFeature {
    pub id: String,
    pub circle: PointFeatureDrawing,
    pub label: Label,
    pub input_point: Option<InputPoint>,
    pub link: Option<svg::node::element::Path>,
    pub point_index: usize,
}

pub trait CandidatesGenerator {
    fn generate(
        &self,
        points: &Vec<PointFeature>,
        obstacles: &Obstacles,
    ) -> BTreeMap<usize, Candidates>;
    fn prioritize(&self, segment: &Segment) -> Vec<Vec<usize>>;
}

impl PartialEq for PointFeature {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Eq for PointFeature {}
use std::str::FromStr;

use crate::bbox::BoundingBox;
use crate::inputpoint::InputPoint;
use crate::math::distance2;
use crate::math::Point2D;
use crate::segment::Segment;

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
    pub fn make_link(&mut self, obstacles: &Obstacles) {
        let circle = &self.circle.center;
        let label = self.label.bbox.bbox.project_on_border(circle);
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
        sd_group.append(self.circle.group.clone());
        let text = format!("{}", self.text());
        let mut label = svg::node::element::Text::new(text);
        let center = &self.circle.center;
        for (k, v) in self.label.to_attributes(center) {
            label = label.set(k, v);
        }
        let mut whitebg = svg::node::element::Rectangle::new();
        let margin = 2f64;
        whitebg = whitebg.set("x", self.label.bbox.bbox.get_xmin() + margin);
        whitebg = whitebg.set("y", self.label.bbox.bbox.get_ymin() + margin);
        whitebg = whitebg.set("width", self.label.bbox.bbox.width() - 2.0 * margin);
        whitebg = whitebg.set("height", self.label.bbox.bbox.height() - 2.0 * margin);
        whitebg = whitebg.set("fill", "white");
        whitebg = whitebg.set("fill-opacity", "0.75");
        whitebg = whitebg.set("id", "label-bg");
        sd_group.append(whitebg);
        sd_group.append(label);
        if self.link.is_some() {
            sd_group.append(self.link.as_ref().unwrap().clone());
        }
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

impl Label {
    pub fn _from_attributes(a: &Attributes, text: &str) -> Label {
        let anchor = match a.get("text-anchor") {
            Some(data) => data,
            _ => "start",
        };
        let x = a.get("x").unwrap().to_string().parse::<f64>().unwrap();
        let y = a.get("y").unwrap().to_string().parse::<f64>().unwrap();
        let height = FONTSIZE;
        let width = text_width(text);
        let (top_left, bottom_right) = if anchor == "start" {
            (Point2D::new(x, y - height), Point2D::new(x + width, y))
        } else {
            (Point2D::new(x - width, y - height), Point2D::new(x, y))
        };
        let bbox = LabelBoundingBox::_new_tlbr(top_left, bottom_right);
        Label {
            id: a.get("id").unwrap().to_string(),
            bbox,
            text: String::from_str(text).unwrap(),
        }
    }

    pub fn to_attributes(&self, point: &Point2D) -> Attributes {
        let mut ret = Attributes::new();
        let mut x = self.bbox.x_min() + 2f64;
        let anchor = if self.bounding_box().x_max() < point.x {
            "end"
        } else {
            "start"
        };
        if anchor == "end" {
            x = self.bbox.x_max() - 2f64;
        }
        set_attr(&mut ret, "text-anchor", anchor);
        let y = self.bbox.y_max() - 2f64;
        set_attr(&mut ret, "id", self.id.as_str());
        set_attr(&mut ret, "font-size", format!("{:.1}", FONTSIZE).as_str());
        set_attr(&mut ret, "x", format!("{:.3}", x).as_str());
        set_attr(&mut ret, "y", format!("{:.3}", y).as_str());
        ret
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
    fn new() -> Obstacles {
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

fn build_graph(
    points: &Vec<PointFeature>,
    gen: &dyn CandidatesGenerator,
    obstacles: &Obstacles,
) -> Graph {
    let mut ret = Graph::new();
    ret.features = points.clone();
    let candidates_map = gen.generate(&points, obstacles);
    for k in 0..points.len() {
        let candidates = candidates_map[&k].clone();
        ret.add_node(k, candidates);
    }
    ret.build_map();
    ret.max_area = obstacles.available_area();
    ret
}

fn _candidate_debug_rectangle(candidate: &Candidate) -> svg::node::element::Rectangle {
    let mut debug_bb = svg::node::element::Rectangle::new();
    let bb = &candidate.bbox();
    debug_bb = debug_bb.set("x", bb.x_min());
    debug_bb = debug_bb.set("y", bb.y_min());
    debug_bb = debug_bb.set("width", bb.width());
    debug_bb = debug_bb.set("height", bb.height());
    debug_bb = debug_bb.set("fill", "transparent");
    debug_bb = debug_bb.set("stroke-width", "1");
    debug_bb = debug_bb.set("stroke", "green");
    debug_bb
}

pub struct PlacementResult {
    pub debug: svg::node::element::Group,
    pub placed_indices: BTreeMap<usize, LabelBoundingBox>,
    pub obstacles: Obstacles,
}

impl PlacementResult {
    pub fn apply(&self, packets: &mut Vec<Vec<PointFeature>>) -> Vec<PointFeature> {
        let mut ret = Vec::new();
        for packet in packets {
            for feature in packet {
                let k = feature.point_index;
                if self.placed_indices.contains_key(&k) {
                    let bbox = self.placed_indices.get(&k).unwrap();
                    feature.place_label(bbox);
                    feature.make_link(&self.obstacles);
                    ret.push(feature.clone());
                } else {
                    log::trace!("could not place {}, index:{}", feature.text(), k,);
                }
            }
        }
        ret
    }
    pub fn push(&mut self, other: Self) {
        for (_k, bbox) in &other.placed_indices {
            self.obstacles.bboxes.push(bbox.bbox.clone());
        }
        self.debug = self.debug.clone().add(other.debug);
        self.placed_indices.extend(other.placed_indices);
    }
}

fn place_subset(
    points: &Vec<PointFeature>,
    gen: &dyn CandidatesGenerator,
    obstacles: &Obstacles,
) -> PlacementResult {
    //log::trace!("build label graph");
    //let mut lsubset = _subset.clone();
    //lsubset.truncate(10);
    //let subset = &lsubset;
    let mut graph = build_graph(points, gen, &obstacles);
    let mut ret = PlacementResult {
        debug: svg::node::element::Group::new(),
        placed_indices: BTreeMap::new(),
        obstacles: Obstacles::new(),
    };
    //log::trace!("solve label graph [{}]", graph.map.len(),);
    let best_candidates = graph.solve();
    for k in 0..points.len() {
        let point = &points[k];
        let target_text = point.text();
        if target_text.is_empty() {
            continue;
        }
        let best_candidate = best_candidates.get(&k);
        match best_candidate {
            Some(candidate) => {
                ret.placed_indices
                    .insert(point.point_index, candidate.bbox().clone());
            }
            _ => {
                log::info!("failed to find any candidate for [{}]", target_text);
            }
        }
    }
    ret
}

pub fn place_labels(
    packets: &Vec<Vec<PointFeature>>,
    gen: &dyn CandidatesGenerator,
    bbox: &BoundingBox,
    polyline: &Polyline,
    max_area_ratio: &f64,
) -> PlacementResult {
    let mut ret = PlacementResult {
        debug: svg::node::element::Group::new(),
        placed_indices: BTreeMap::new(),
        obstacles: Obstacles {
            drawingbox: DrawingArea {
                bbox: bbox.clone(),
                max_area_ratio: *max_area_ratio,
            },
            polylines: vec![polyline.clone()],
            bboxes: Vec::new(),
        },
    };
    for packet in packets {
        if packet.is_empty() {
            continue;
        }
        let results = place_subset(&packet, gen, &ret.obstacles);
        ret.push(results);
    }
    ret
}

pub fn cardinal_boxes(center: &Point2D, width: &f64, height: &f64) -> Vec<LabelBoundingBox> {
    let mut ret = Vec::new();
    let epsilon = 2f64;
    let b1 = LabelBoundingBox::new_blwh(
        Point2D::new(center.x + epsilon, center.y - epsilon),
        *width,
        *height,
    );
    ret.push(b1);
    let b2 = LabelBoundingBox::new_brwh(
        Point2D::new(center.x - epsilon, center.y - epsilon),
        *width,
        *height,
    );
    ret.push(b2);
    let b3 = LabelBoundingBox::new_trwh(
        Point2D::new(center.x - epsilon, center.y + epsilon),
        *width,
        *height,
    );
    ret.push(b3);
    let b4 = LabelBoundingBox::new_tlwh(
        Point2D::new(center.x + epsilon, center.y + epsilon),
        *width,
        *height,
    );
    ret.push(b4);

    let b5 = LabelBoundingBox::new_blwh(
        Point2D::new(center.x + epsilon, center.y + height / 2.0),
        *width,
        *height,
    );
    ret.push(b5);
    let b6 = LabelBoundingBox::new_blwh(
        Point2D::new(center.x - width / 2.0, center.y - epsilon),
        *width,
        *height,
    );
    ret.push(b6);
    let b7 = LabelBoundingBox::new_brwh(
        Point2D::new(center.x - epsilon, center.y + height / 2.0),
        *width,
        *height,
    );
    ret.push(b7);
    let b8 = LabelBoundingBox::new_tlwh(
        Point2D::new(center.x - width / 2.0, center.y + epsilon),
        *width,
        *height,
    );
    ret.push(b8);

    ret
}

pub fn far_boxes(
    center: &Point2D,
    width: &f64,
    height: &f64,
    level: usize,
) -> Vec<LabelBoundingBox> {
    let mut ret = Vec::new();
    let d = ((level + 2) as f64) * height;
    let xmin = center.x - d;
    let xmax = center.x + d;
    let ymin = center.y - d;
    let ymax = center.y + d;
    let stepsize = *height;
    let mut n = 0;
    loop {
        let b = LabelBoundingBox::new_tlwh(
            Point2D::new(xmin + (n as f64) * stepsize, ymin),
            *width,
            *height,
        );
        if b.x_max() > xmax {
            break;
        }
        ret.push(b);
        n += 1;
    }
    n = 0;
    loop {
        let b = LabelBoundingBox::new_blwh(
            Point2D::new(xmin + (n as f64) * stepsize, ymax),
            *width,
            *height,
        );
        if b.x_max() > xmax {
            break;
        }
        ret.push(b);
        n += 1;
    }
    n = 0;
    loop {
        let b = LabelBoundingBox::new_trwh(
            Point2D::new(xmax, ymin + (n as f64) * stepsize),
            *width,
            *height,
        );
        if b.y_max() > ymax {
            break;
        }
        ret.push(b);
        n += 1;
    }
    n = 0;
    loop {
        let b = LabelBoundingBox::new_tlwh(
            Point2D::new(xmin, ymin + (n as f64) * stepsize),
            *width,
            *height,
        );
        if b.y_max() > ymax {
            break;
        }
        ret.push(b);
        n += 1;
    }
    ret.sort_by_key(|candidate| {
        let p = candidate.bbox.project_on_border(center);
        (distance2(center, &p) * 100f64).floor() as i64
    });
    ret
}
