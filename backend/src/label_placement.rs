use crate::label_candidates;
use crate::label_candidates::Candidate;
use crate::label_candidates::Candidates;
use crate::label_candidates::LabelBoundingBox;
use crate::label_graph::Graph;

use std::cmp::Ordering;
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
pub struct Circle {
    pub id: String,
    pub cx: f64,
    pub cy: f64,
    pub r: f64,
    pub fill: Option<String>,
}

impl Circle {
    pub fn new() -> Circle {
        Circle {
            id: String::new(),
            cx: 0f64,
            cy: 0f64,
            r: 4f64,
            fill: None,
        }
    }
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
pub struct PointFeature {
    pub id: String,
    pub circle: Circle,
    pub label: Label,
}

impl PartialEq for PointFeature {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Eq for PointFeature {}
use std::hash::Hash;
use std::hash::Hasher;
use std::str::FromStr;

impl Hash for PointFeature {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.id.hash(state);
    }
}

impl PointFeature {
    pub fn new() -> PointFeature {
        PointFeature {
            id: String::new(),
            circle: Circle::new(),
            label: Label::new(),
        }
    }
}

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

impl Circle {
    pub fn _from_attributes(a: &Attributes) -> Circle {
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

    pub fn to_attributes(&self) -> Attributes {
        let mut ret = Attributes::new();
        set_attr(&mut ret, "id", self.id.as_str());
        set_attr(&mut ret, "cx", format!("{}", self.cx).as_str());
        set_attr(&mut ret, "cy", format!("{}", self.cy).as_str());
        set_attr(&mut ret, "r", format!("{}", self.r).as_str());
        ret
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
        let mut x = self.bbox.top_left.0 + 2f64;
        let anchor = if self.bounding_box().x_max() < cx {
            "end"
        } else {
            "start"
        };
        if anchor == "end" {
            x = self.bbox.bottom_right.0 - 2f64;
        }
        set_attr(&mut ret, "text-anchor", anchor);
        let y = self.bbox.bottom_right.1 - 2f64;
        set_attr(&mut ret, "id", self.id.as_str());
        set_attr(&mut ret, "font-size", format!("{:.1}", FONTSIZE).as_str());
        set_attr(&mut ret, "x", format!("{}", x).as_str());
        set_attr(&mut ret, "y", format!("{}", y).as_str());
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

fn generate_bboxes(point: &PointFeature) -> Vec<LabelBoundingBox> {
    let mut ret = Vec::new();
    let width = point.label.bbox.width();
    let height = point.label.bbox.height();
    let dtarget_min = 1f64;
    let dtarget_max = 20f64;
    let d0 = dtarget_max;
    let cx = point.circle.cx;
    let cy = point.circle.cy;
    let xmin = cx - d0 - width;
    let ymin = cy - d0 - height;
    let xmax = cx + d0;
    let ymax = cy + d0;
    let dp = 5f64;
    let countx = ((xmax - xmin) / dp).ceil() as i32;
    let county = ((ymax - ymin) / dp).ceil() as i32;
    let dx = dp;
    let dy = dp;
    for nx in 0..countx {
        for ny in 0..county {
            let tl = (xmin + nx as f64 * dx, ymin + ny as f64 * dy);
            let bb = LabelBoundingBox::new_tlwh(tl, width, height);
            if bb.contains((cx, cy)) {
                continue;
            }
            if bb.distance((cx, cy)) < dtarget_min {
                continue;
            }
            ret.push(bb);
        }
    }
    ret
}

fn distance_to_others(
    bbox: &LabelBoundingBox,
    points: &Vec<PointFeature>,
    target_id: &String,
) -> (f64, usize) {
    let mut ret = (f64::MAX, 0);
    for l in 0..points.len() {
        let other = &points[l];
        if other.id == *target_id {
            continue;
        }
        let d = bbox.distance((other.circle.cx, other.circle.cy));
        if d < ret.0 {
            ret = (d, l);
        }
    }
    ret
}

fn generate_all_candidates(points: &Vec<PointFeature>, k: usize) -> Candidates {
    if points[k].label.text.is_empty() {
        return Candidates::new();
    }
    let target = &points[k];
    let all = generate_bboxes(target);
    let mut ret = Candidates::new();
    let targetpoint = (target.circle.cx, target.circle.cy);
    for index in 0..all.len() {
        let c = &all[index];
        // let dtarget = distance(c.center(), targetpoint);
        let dtarget = c.distance(targetpoint);
        let (dothers, _) = distance_to_others(c, &points, &target.id);
        ret.push(Candidate::new(c.clone(), dtarget, dothers));
    }
    return ret;
}

fn filter_sort_candidates(candidates: &mut Candidates, polyline: &Polyline) {
    candidates.retain(|c| {
        if polyline_hits_bbox(polyline, &c.bbox) {
            return false;
        }
        if c.dothers < c.dtarget {
            return false;
        }
        true
    });
    candidates.sort_by(|ci, cj| ci.partial_cmp(cj).unwrap_or(Ordering::Equal));
}

fn build_graph(points: &Vec<PointFeature>, polyline: &Polyline) -> Graph {
    let mut ret = Graph::new();
    for k in 0..points.len() {
        if points[k].label.text.is_empty() {
            continue;
        }
        let mut candidates = generate_all_candidates(points, k);
        filter_sort_candidates(&mut candidates, polyline);
        let selected_indices = label_candidates::select_candidates(&candidates);
        let selected_candidates: Vec<_> = selected_indices
            .into_iter()
            .map(|i| candidates[i].clone())
            .collect();
        assert!(!ret.candidates.contains_key(&k));
        ret.add_node(k, selected_candidates);
    }
    ret.build_map();
    ret
}

fn candidate_debug_rectangle(candidate: &Candidate) -> svg::node::element::Rectangle {
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

pub fn place_labels(
    points: &mut Vec<PointFeature>,
    polyline: &Polyline,
) -> svg::node::element::Group {
    let mut graph = build_graph(points, polyline);
    let sorted = graph.debug();
    for item in sorted {
        let (k, others) = item;
        let name = &points[k].label.text;
        let othersnames: Vec<_> = others
            .into_iter()
            .map(|i| points[i].label.text.clone())
            .collect();
        println!("[{k:2} name:{:20}] order:{:?}", name, othersnames)
    }
    let mut debug = svg::node::element::Group::new();
    for k in 0..points.len() {
        let target_text = &points[k].label.text;
        if target_text.is_empty() {
            continue;
        }
        /*
                let mut candidates = generate_all_candidates(points, k);
                if true {
                    filter_sort_candidates(&mut candidates, polyline);
                    let selected_indices = label_candidates::select_candidates(&candidates);
                    let candidatesd: Vec<_> = selected_indices
                        .into_iter()
                        .map(|i| candidates[i].clone())
                        .collect();
        }
             */
        /*
            for c in candidates {
                if target_text.contains("Ehr") {
                    debug.append(candidate_debug_rectangle(&c));
                }
        }
            */
    }
    let best_candidates = graph.solve();
    for k in 0..points.len() {
        let target_text = &points[k].label.text;
        if target_text.is_empty() {
            continue;
        }
        let best_candidate = best_candidates.get(&k);
        match best_candidate {
            Some(candidate) => {
                let bbox = &candidate.bbox;
                let dothers = &candidate.dothers;
                let dtarget = &candidate.dtarget;
                println!(
                    "[{k}={:12}] c({:.1},{:.1}) d_t={:.1} d_o = {:.1}]",
                    target_text,
                    bbox.x_min(),
                    bbox.y_max(),
                    dtarget,
                    dothers
                );
                points[k].label.bbox = bbox.clone();
            }
            _ => {
                println!("failed to find any candidate for [{}]", target_text);
            }
        }
    }
    debug
}

#[cfg(test)]
mod tests {
    use crate::label_candidates::LabelBoundingBox;

    use super::*;

    #[test]
    fn test_bbox() {
        let id = String::new();
        let target = PointFeature {
            id: id.clone(),
            circle: Circle {
                id: id.clone(),
                cx: 0f64,
                cy: 0f64,
                r: 1f64,
                fill: None,
            },
            label: Label {
                id: id.clone(),
                bbox: LabelBoundingBox {
                    top_left: (0f64, 0f64),
                    bottom_right: (10f64, 16f64),
                },
                text: String::from_str("hi").unwrap(),
            },
        };
        let candidates = generate_bboxes(&target);
        let mut found = false;
        for c in candidates {
            let good = c.top_left.0 > target.circle.cx && c.top_left.1 > target.circle.cy;
            println!("{}", c.to_string());
            if good {
                found = true;
            }
        }
        assert!(found);
    }
}
