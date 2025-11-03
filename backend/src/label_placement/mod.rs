pub mod bbox;
pub mod candidate;
pub mod drawings;
mod graph;
pub mod prioritize;
pub use bbox::LabelBoundingBox;
use candidate::Candidate;
use candidate::Candidates;
use graph::Graph;
use svg::Node;

use std::cmp::Ordering;
use std::collections::BTreeMap;
use std::collections::BTreeSet;
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
        self.bbox = LabelBoundingBox::new_blwh((0f64, 0f64), width, FONTSIZE);
    }

    pub fn bounding_box(&self) -> LabelBoundingBox {
        self.bbox.clone()
    }
}

#[derive(Clone)]
pub struct PointFeatureDrawing {
    pub group: svg::node::element::Group,
    pub cx: f64,
    pub cy: f64,
}

#[derive(Clone)]
pub struct PointFeature {
    pub id: String,
    pub circle: PointFeatureDrawing,
    pub label: Label,
    pub input_point: Option<InputPoint>,
}

pub trait CandidatesGenerator {
    fn generate(&self, point: &PointFeature) -> Vec<LabelBoundingBox>;
    fn prioritize(&self, points: &Vec<PointFeature>) -> Vec<BTreeSet<usize>>;
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
    pub fn text(&self) -> String {
        self.label.text.clone()
    }
    pub fn center(&self) -> (f64, f64) {
        (self.circle.cx, self.circle.cy)
    }
    pub fn input_point(&self) -> Option<InputPoint> {
        self.input_point.clone()
    }
    pub fn render_in_group(&self, sd_group: &mut svg::node::element::Group) {
        sd_group.append(self.circle.group.clone());
        let text = format!("{}", self.text());
        let mut label = svg::node::element::Text::new(text);
        for (k, v) in self.label.to_attributes(self.circle.cx) {
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
    }
}

#[derive(Clone)]
pub struct Polyline {
    id: String,
    pub points: Vec<(f64, f64)>,
}

impl Polyline {
    pub fn new() -> Polyline {
        Polyline {
            id: "track".to_string(),
            points: Vec::<(f64, f64)>::new(),
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
            ((x, y - height), (x + width, y))
        } else {
            ((x - width, y - height), (x, y))
        };
        let bbox = LabelBoundingBox::new_tlbr(top_left, bottom_right);
        Label {
            id: a.get("id").unwrap().to_string(),
            bbox,
            text: String::from_str(text).unwrap(),
        }
    }

    pub fn to_attributes(&self, cx: f64) -> Attributes {
        let mut ret = Attributes::new();
        let mut x = self.bbox.x_min() + 2f64;
        let anchor = if self.bounding_box().x_max() < cx {
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
            points.push((x, y));
        }
        Polyline {
            id: format!("{}", a.get("id").unwrap()),
            points,
        }
    }

    pub fn to_attributes(&self) -> Attributes {
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
        set_attr(&mut ret, "stroke-width", "2");
        set_attr(&mut ret, "stroke", "black");
        set_attr(&mut ret, "stroke-linejoin", "miter");
        set_attr(&mut ret, "stroke-miterlimit", "1");
        set_attr(&mut ret, "d", d.as_str());
        ret
    }
}

fn polyline_hits_bbox(polyline: &Polyline, bbox: &LabelBoundingBox) -> bool {
    for &(x, y) in &polyline.points {
        if bbox.contains((x, y)) {
            return true;
        }
    }

    false
}

fn distance_to_others(
    bbox: &LabelBoundingBox,
    points: &Vec<PointFeature>,
    k: usize,
) -> (f64, usize) {
    let mut ret = (f64::MAX, 0);
    for l in 0..points.len() {
        let other = &points[l];
        if l == k {
            continue;
        }
        let other_center = (other.circle.cx, other.circle.cy);
        let d = bbox.distance(other_center);
        if d < ret.0 {
            ret = (d, l);
        }
    }
    ret
}

fn generate_all_candidates(
    gen: &dyn CandidatesGenerator,
    points: &Vec<PointFeature>,
    k: usize,
) -> Candidates {
    if points[k].text().is_empty() {
        return Candidates::new();
    }
    let target = &points[k];
    let all = gen.generate(target);
    let mut ret = Candidates::new();
    let targetpoint = (target.circle.cx, target.circle.cy);
    for index in 0..all.len() {
        let c = &all[index];
        // let dtarget = distance(c.center(), targetpoint);
        let dtarget = c.distance(targetpoint);
        let (dothers, _) = distance_to_others(c, &points, k);
        ret.push(Candidate::new(c.clone(), dtarget, dothers));
    }
    return ret;
}

struct Obstacles {
    bboxes: Vec<BoundingBox>,
    polylines: Vec<Polyline>,
}

impl Obstacles {
    fn from_polyline(p: &Polyline) -> Obstacles {
        Obstacles {
            bboxes: Vec::new(),
            polylines: vec![p.clone()],
        }
    }
}

fn filter_sort_candidates(
    candidates: &mut Candidates,
    drawbox: &BoundingBox,
    obstacles: &Obstacles,
) {
    candidates.retain(|candidate| {
        if !drawbox.contains_other(&candidate.bbox.bbox) {
            return false;
        }
        for obstacle_box in &obstacles.bboxes {
            if candidate.bbox.bbox.hits_other(obstacle_box) {
                return false;
            }
        }
        for polyline in &obstacles.polylines {
            if polyline_hits_bbox(polyline, &candidate.bbox) {
                return false;
            }
        }
        true
    });
    // dtarget and dothers are considered in the ordering of candidates
    candidates.sort_by(|ci, cj| ci.partial_cmp(cj).unwrap_or(Ordering::Equal));
}

fn build_graph_gen(
    points: &Vec<PointFeature>,
    subset: &BTreeSet<usize>,
    gen: &dyn CandidatesGenerator,
    drawingbox: &BoundingBox,
    obstacles: &Obstacles,
) -> Graph {
    let mut ret = Graph::new();
    ret.features = points.clone();
    for _k in subset {
        let k = *_k;
        let text = points[k].text();
        if text.is_empty() {
            continue;
        }
        let mut candidates = generate_all_candidates(gen, points, k);
        assert!(!candidates.is_empty());
        /*log::trace!(
            "[0] [{}] => {} candidates",
            points[k].text(),
            candidates.len()
        );*/
        let _first = candidates.first().unwrap().clone();
        filter_sort_candidates(&mut candidates, drawingbox, obstacles);
        if candidates.is_empty() {
            log::warn!(
                "{} has {} candidate after filtering with {} obstacles",
                points[k].text(),
                candidates.len(),
                obstacles.bboxes.len()
            );
        }
        /*log::trace!(
            "[1] [{}] => {} candidates",
            points[k].text(),
            candidates.len()
        );*/
        let selected_indices = candidate::select_candidates(&candidates);
        let selected_candidates: Vec<_> = selected_indices
            .into_iter()
            .map(|i| candidates[i].clone())
            .collect();
        assert!(!ret.candidates.contains_key(&k));
        if selected_candidates.is_empty() {
            log::warn!("{} has no candidate after selection.", points[k].text());
        }
        /*log::debug!(
            "[2] [{}] => {} candidates",
            points[k].text(),
            selected_candidates.len()
        );*/
        ret.add_node(k, selected_candidates);
    }
    ret.build_map();
    ret
}

fn _candidate_debug_rectangle(candidate: &Candidate) -> svg::node::element::Rectangle {
    let mut debug_bb = svg::node::element::Rectangle::new();
    let bb = &candidate.bbox;
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
    pub forced_indices: BTreeMap<usize, LabelBoundingBox>,
}

impl PlacementResult {
    pub fn apply(&self, points: &mut Vec<PointFeature>, force: bool) -> Vec<usize> {
        let mut ret = Vec::new();
        for (k, bbox) in &self.placed_indices {
            points[*k].place_label(bbox);
            ret.push(*k);
        }
        if force {
            for (k, bbox) in &self.forced_indices {
                log::trace!("force place {}", points[*k].text());
                points[*k].place_label(bbox);
                ret.push(*k);
            }
        }
        ret
    }
}

fn place_labels_gen_worker(
    points: &Vec<PointFeature>,
    subset: &BTreeSet<usize>,
    gen: &dyn CandidatesGenerator,
    bbox: &BoundingBox,
    obstacles: &Obstacles,
) -> PlacementResult {
    //log::trace!("build label graph");
    let mut graph = build_graph_gen(points, subset, gen, bbox, &obstacles);
    let candidates = graph.candidates.clone();
    let mut ret = PlacementResult {
        debug: svg::node::element::Group::new(),
        placed_indices: BTreeMap::new(),
        forced_indices: BTreeMap::new(),
    };
    for k in 0..points.len() {
        let target_text = &points[k].text();
        if target_text.is_empty() {
            continue;
        }
    }
    //log::trace!("solve label graph [{}]", graph.map.len(),);
    let best_candidates = graph.solve();
    for k in subset {
        let point = &points[*k];
        let target_text = point.text();
        if target_text.is_empty() {
            continue;
        }
        let best_candidate = best_candidates.get(&k);
        match best_candidate {
            Some(candidate) => {
                let bbox = &candidate.bbox;
                ret.placed_indices.insert(*k, bbox.clone());
            }
            _ => {
                log::info!("failed to find any candidate for [{}]", target_text);
                // here we could force with
                let candidate = candidates.get(k).unwrap().first();
                match candidate {
                    Some(c) => {
                        let bbox = &c.bbox;
                        ret.forced_indices.insert(*k, bbox.clone());
                    }
                    None => {
                        log::info!("no candidate available for [{}] => make one", target_text);
                        let bbox = gen.generate(point).first().unwrap().clone();
                        ret.forced_indices.insert(*k, bbox);
                    }
                }
            }
        }
    }
    ret
}

pub fn place_labels_gen(
    points: &Vec<PointFeature>,
    gen: &dyn CandidatesGenerator,
    bbox: &BoundingBox,
    polyline: &Polyline,
    force: bool,
) -> PlacementResult {
    let mut ret = PlacementResult {
        debug: svg::node::element::Group::new(),
        placed_indices: BTreeMap::new(),
        forced_indices: BTreeMap::new(),
    };
    let mut obstacles = Obstacles::from_polyline(polyline);
    let packets = gen.prioritize(points);
    for packet_points in packets {
        if packet_points.is_empty() {
            continue;
        }
        let results = place_labels_gen_worker(&points, &packet_points, gen, bbox, &obstacles);
        for (k, bbox) in &results.placed_indices {
            obstacles.bboxes.push(bbox.bbox.clone());
        }
        if force {
            for (k, bbox) in &results.forced_indices {
                obstacles.bboxes.push(bbox.bbox.clone());
            }
        }

        ret.debug = ret.debug.add(results.debug);
        ret.placed_indices.extend(results.placed_indices);
        ret.forced_indices.extend(results.forced_indices);
    }
    ret
}
