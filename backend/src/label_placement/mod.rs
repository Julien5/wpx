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
        self.bbox = LabelBoundingBox::new_relative(
            &BoundingBox::minsize(Point2D::new(0.0, -FONTSIZE), &width, &FONTSIZE),
            &Point2D::zero(),
        );
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

type PointFeatureId = usize;

#[derive(Clone)]
pub struct PointFeature {
    pub circle: PointFeatureDrawing,
    pub label: Label,
    pub input_point: Option<InputPoint>,
    pub link: Option<svg::node::element::Path>,
    pub id: PointFeatureId,
}

pub trait CandidatesGenerator {
    fn generate(
        &self,
        features: &Vec<PointFeature>,
        obstacles: &Obstacles,
    ) -> BTreeMap<usize, Candidates>;
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
use std::str::FromStr;

use crate::bbox::BoundingBox;
use crate::inputpoint::InputPoint;
use crate::math::distance2;
use crate::math::Point2D;

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
    features: &Vec<PointFeature>,
    gen: &dyn CandidatesGenerator,
    obstacles: &Obstacles,
) -> Graph {
    let mut ret = Graph::new();
    for feature in features {
        ret.features.insert(feature.clone());
    }
    let candidates_map = gen.generate(&features, obstacles);
    for feature in features {
        let feature_id = feature.id;
        let candidates = candidates_map[&feature_id].clone();
        ret.add_node(feature, candidates);
    }
    ret.build_map();
    ret.max_area = obstacles.available_area();
    ret
}

fn _candidate_debug_rectangle(candidate: &Candidate) -> svg::node::element::Rectangle {
    let mut debug_bb = svg::node::element::Rectangle::new();
    let bb = &candidate.bbox();
    debug_bb = debug_bb.set("x", bb.relative().get_xmin());
    debug_bb = debug_bb.set("y", bb.relative().get_ymin());
    debug_bb = debug_bb.set("width", bb.width());
    debug_bb = debug_bb.set("height", bb.height());
    debug_bb = debug_bb.set("fill", "transparent");
    debug_bb = debug_bb.set("stroke-width", "1");
    debug_bb = debug_bb.set("stroke", "green");
    debug_bb
}

pub struct PlacementResult {
    pub debug: svg::node::element::Group,
    pub placed_indices: BTreeMap<PointFeatureId, LabelBoundingBox>,
    pub obstacles: Obstacles,
}

impl PlacementResult {
    pub fn apply(&self, packets: &mut Vec<Vec<PointFeature>>) -> Vec<PointFeature> {
        let mut ret = Vec::new();
        for packet in packets {
            for feature in packet {
                let feature_id = feature.id;
                if self.placed_indices.contains_key(&feature_id) {
                    let bbox = self.placed_indices.get(&feature_id).unwrap().clone();
                    feature.place_label(&bbox);
                    //feature.make_link(&self.obstacles);
                    ret.push(feature.clone());
                } else {
                    //log::trace!("could not place {}, index:{}", feature.text(), feature_id,);
                }
            }
        }
        ret
    }
    pub fn push(&mut self, other: Self) {
        for (_k, bbox) in &other.placed_indices {
            self.obstacles.bboxes.push(bbox.absolute().clone());
        }
        self.debug = self.debug.clone().add(other.debug);
        self.placed_indices.extend(other.placed_indices);
    }
}

fn place_quick_best_candidates(
    features: &Vec<PointFeature>,
    obstacles: &Obstacles,
) -> BTreeMap<PointFeatureId, Candidate> {
    let mut map_candidate = BTreeMap::new();
    let mut available = obstacles.available_area();
    for feature in features {
        let cboxes = cardinal_boxes(&feature.center(), &feature.width(), &feature.height());
        let first = cboxes.first().unwrap();
        let candidate = Candidate::new(first, &1f64, &1f64);
        if available < candidate.bbox().area() {
            break;
        }
        available -= candidate.bbox().area();
        map_candidate.insert(feature.id, candidate);
    }
    map_candidate
}

fn place_subset(
    features: &Vec<PointFeature>,
    gen: &dyn CandidatesGenerator,
    obstacles: &Obstacles,
) -> PlacementResult {
    let mut ret = PlacementResult {
        debug: svg::node::element::Group::new(),
        placed_indices: BTreeMap::new(),
        obstacles: Obstacles::new(),
    };
    let quick = true;
    let best_candidates = match quick {
        false => {
            let mut graph = build_graph(features, gen, &obstacles);
            graph.solve()
        }
        true => place_quick_best_candidates(features, obstacles),
    };
    //log::trace!("solve label graph [{}]", graph.map.len(),);

    for feature in features {
        let target_text = feature.text();
        if target_text.is_empty() {
            continue;
        }
        let best_candidate = best_candidates.get(&feature.id);
        match best_candidate {
            Some(candidate) => {
                ret.placed_indices
                    .insert(feature.id, candidate.bbox().clone());
            }
            _ => {
                //log::trace!("failed to find any candidate for [{}]", target_text);
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

fn make(bbox0: &BoundingBox, translation: &Point2D, center: &Point2D) -> LabelBoundingBox {
    LabelBoundingBox::new_relative(&bbox0.make_translate(&translation), center)
}

pub fn cardinal_boxes(center: &Point2D, width: &f64, height: &f64) -> Vec<LabelBoundingBox> {
    let mut ret = Vec::new();
    let epsilon = 2f64;
    let dx = 2f64 * epsilon + width;
    let dy = 2f64 * epsilon + height;
    let bbox0 = BoundingBox::minsize(Point2D::new(epsilon, -epsilon - height), width, height);

    ret.push(make(&bbox0, &Point2D::new(0.0, 0.0), center));
    ret.push(make(&bbox0, &Point2D::new(-dx, 0.0), center));
    ret.push(make(&bbox0, &Point2D::new(-dx, dy), center));
    ret.push(make(&bbox0, &Point2D::new(0.0, dy), center));

    let bbox_right = BoundingBox::minsize(Point2D::new(epsilon, -height / 2.0), width, height);
    let bbox_up = BoundingBox::minsize(Point2D::new(-width / 2.0, epsilon), width, height);

    ret.push(make(&bbox_right, &Point2D::new(0.0, 0.0), center));
    ret.push(make(&bbox_up, &Point2D::new(0.0, 0.0), center));
    ret.push(make(&bbox_right, &Point2D::new(-dx, 0.0), center));
    ret.push(make(&bbox_up, &Point2D::new(0.0, dy), center));

    ret
}

pub fn far_boxes(
    target: &Point2D,
    width: &f64,
    height: &f64,
    level: usize,
) -> Vec<LabelBoundingBox> {
    let mut ret = Vec::new();
    let d = ((level + 2) as f64) * height;
    let stepsize = *height;

    let bbox0 = BoundingBox::minsize(Point2D::new(-d, -d), width, height);

    let mut n = 0;
    loop {
        let b = make(&bbox0, &Point2D::new((n as f64) * stepsize, 0.0), target);
        if b.relative().get_xmax() > d {
            break;
        }
        ret.push(b);
        n += 1;
    }

    let bbox0 = BoundingBox::minsize(Point2D::new(-d, d - height), width, height);
    n = 0;
    loop {
        let b = make(&bbox0, &Point2D::new((n as f64) * stepsize, 0.0), target);
        if b.relative().get_xmax() > d {
            break;
        }
        ret.push(b);
        n += 1;
    }

    let bbox0 = BoundingBox::minsize(Point2D::new(d - width, -d), width, height);
    n = 0;
    loop {
        let b = make(&bbox0, &Point2D::new(0.0, (n as f64) * stepsize), target);
        if b.relative().get_ymax() > d {
            break;
        }
        ret.push(b);
        n += 1;
    }

    let bbox0 = BoundingBox::minsize(Point2D::new(-d, -d), width, height);
    n = 0;
    loop {
        let b = make(&bbox0, &Point2D::new(0.0, (n as f64) * stepsize), target);
        if b.relative().get_ymax() > d {
            break;
        }
        ret.push(b);
        n += 1;
    }

    ret.sort_by_key(|candidate| {
        let p = candidate.absolute().project_on_border(target);
        (distance2(target, &p) * 100f64).floor() as i64
    });
    ret
}
