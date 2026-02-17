use std::{collections::HashMap, str::FromStr};

use rstar::{PointDistance, RTree, RTreeObject, AABB};

use crate::{
    bbox::BoundingBox,
    inputpoint::InputPoint,
    label_placement::labelboundingbox::LabelBoundingBox,
    math::{self, distance2, Point2D},
};

pub type Attributes = HashMap<String, svg::node::Value>;

pub fn set_attr(attr: &mut Attributes, k: &str, v: &str) {
    attr.insert(String::from_str(k).unwrap(), svg::node::Value::from(v));
}

pub const FONTSIZE: f64 = 14f64;

#[cfg(test)]
pub const DEBUG: bool = true;

#[cfg(not(test))]
pub const DEBUG: bool = false;

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

fn text_width(s: &str, fontsize: f64, fontweight: &str, fontstyle: &str) -> f64 {
    let mut ret = 0f64;
    for c in s.chars() {
        let mut w = char_width(&c);
        w = w * (fontsize / FONTSIZE);
        if fontweight == "normal" {
        } else if fontweight == "bold" {
            w = w * 1.1;
        } else if fontweight == "lighter" {
            w = w / 1.1;
        } else {
            log::error!("unknown weight {}", fontweight);
            assert!(false);
        }
        if fontstyle == "normal" {
        } else if fontstyle == "italic" {
            w = w / 1.1;
        } else {
            log::error!("unknown style {}", fontstyle);
            assert!(false);
        }
        ret += w;
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
    pub fontsize: f64,
    pub fontweight: String,
    pub fontstyle: String,
    pub bbox: LabelBoundingBox,
    pub text: String,
    pub _placed: bool, //ugly
}

impl Label {
    pub fn empty() -> Self {
        Self {
            id: String::new(),
            fontsize: FONTSIZE,
            fontweight: String::new(),
            fontstyle: String::new(),
            bbox: LabelBoundingBox::zero(),
            text: String::new(),
            _placed: false,
        }
    }
    pub fn is_empty(&self) -> bool {
        self.text.is_empty()
    }
    pub fn new(text: &str, fontsize: f64, fontweight: &str, fontstyle: &str) -> Self {
        let width = text_width(text, fontsize, fontweight, fontstyle);
        let bbox = LabelBoundingBox::new_relative(
            &BoundingBox::minsize(Point2D::new(0.0, -fontsize), width, fontsize),
            &Point2D::zero(),
        );
        Label {
            id: String::new(),
            fontsize,
            fontweight: fontweight.to_string(),
            fontstyle: fontstyle.to_string(),
            bbox,
            text: text.to_string(),
            _placed: false,
        }
    }

    pub fn placed(&self) -> bool {
        self._placed
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
        set_attr(
            &mut ret,
            "font-size",
            format!("{:.1}", self.fontsize).as_str(),
        );
        set_attr(
            &mut ret,
            "font-weight",
            format!("{}", self.fontweight).as_str(),
        );
        set_attr(
            &mut ret,
            "font-style",
            format!("{}", self.fontstyle).as_str(),
        );
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
    pub xmlid: PointFeatureId,
}

impl PointFeature {
    pub fn place_label(&mut self, bbox: &LabelBoundingBox) {
        debug_assert!(math::nearly_equal(self.label.bbox.height(), bbox.height()));
        debug_assert!(math::nearly_equal(self.label.bbox.width(), bbox.width()));
        self.label.bbox = bbox.clone();
        self.label._placed = true;
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
    pub fn render_in_group(&self, sd_group: &mut svg::node::element::Group) {
        use svg::Node;
        sd_group.append(self.circle.group.clone());
        let text = format!("{}", self.text());

        if self.label.placed() {
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

            if DEBUG {
                let mut debugrect = svg::node::element::Rectangle::new();
                debugrect = debugrect.set("x", self.label.bbox.relative().get_xmin());
                debugrect = debugrect.set("y", self.label.bbox.relative().get_ymin());
                debugrect = debugrect.set("width", self.label.bbox.relative().width());
                debugrect = debugrect.set("height", self.label.bbox.relative().height());
                debugrect = debugrect.set("stroke", "blue");
                debugrect = debugrect.set("fill", "none");
                debugrect = debugrect.set("stroke-width", "2");
                debugrect = debugrect.set("id", "label-dbg");
                subgroup.append(debugrect);
            }

            subgroup.append(label);
            sd_group.append(subgroup);
            if self.link.is_some() {
                sd_group.append(self.link.as_ref().unwrap().clone());
            }
        }
    }
}

impl PartialEq for PointFeature {
    fn eq(&self, other: &Self) -> bool {
        self.xmlid == other.xmlid
    }
}

impl PartialOrd for PointFeature {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.xmlid.partial_cmp(&other.xmlid)
    }
}

impl Ord for PointFeature {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.xmlid.cmp(&other.xmlid)
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
        let p = Point2D::new(point[0], point[1]);
        distance2(&c, &p)
    }
}

pub struct PointFeatures {
    pub points: Vec<PointFeature>,
    tree: RTree<PointFeature>,
}

impl PointFeatures {
    pub fn make(points: Vec<PointFeature>) -> PointFeatures {
        let tree = RTree::bulk_load(points.clone());
        PointFeatures { points, tree }
    }

    /*pub fn nearest_neighbor_excluding_self<'a>(
        &'a self,
        target: &PointFeature,
    ) -> Option<&'a PointFeature> {
        self.tree
            .nearest_neighbor_iter(&[target.center().x, target.center().y])
            .filter(|&p| p.id != target.id)
            .next()
    }*/

    pub fn nearest_neighbors<'a>(
        &'a self,
        point: &Point2D,
        n: usize,
    ) -> Vec<(&'a PointFeature, f64)> {
        self.tree
            .nearest_neighbor_iter_with_distance_2(&[point.x, point.y])
            .take(n)
            .collect()
    }
}

#[derive(Clone, Copy)]
pub struct PolylinePoint(pub Point2D);
pub type PolylinePoints = Vec<PolylinePoint>;

#[derive(Clone)]
pub struct PolylineSegment {
    pub start: Point2D,
    pub end: Point2D,
}

// Implement RTreeObject for PolylineSegment
impl RTreeObject for PolylineSegment {
    type Envelope = AABB<[f64; 2]>;

    fn envelope(&self) -> Self::Envelope {
        AABB::from_corners(
            [self.start.x.min(self.end.x), self.start.y.min(self.end.y)],
            [self.start.x.max(self.end.x), self.start.y.max(self.end.y)],
        )
    }
}

#[derive(Clone)]
pub struct Polyline {
    id: String,
    pub points: PolylinePoints,
    tree: RTree<PolylineSegment>, // Store segments instead
}

impl RTreeObject for PolylinePoint {
    type Envelope = AABB<[f64; 2]>;

    fn envelope(&self) -> Self::Envelope {
        AABB::from_point([self.0.x, self.0.y])
    }
}

impl PointDistance for PolylinePoint {
    fn distance_2(&self, point: &[f64; 2]) -> f64 {
        let c = self.0;
        let p = Point2D::new(point[0], point[1]);
        distance2(&c, &p)
    }
}

impl Polyline {
    pub fn new(points: PolylinePoints) -> Polyline {
        let mut segments = Vec::new();
        for i in 0..points.len().saturating_sub(1) {
            segments.push(PolylineSegment {
                start: points[i].0.clone(),
                end: points[i + 1].0.clone(),
            });
        }
        let tree = RTree::bulk_load(segments);
        Polyline {
            id: "track".to_string(),
            points: points,
            tree: tree,
        }
    }

    pub fn hit(&self, bbox: &BoundingBox) -> bool {
        let aabb = AABB::from_corners(
            [bbox.get_xmin(), bbox.get_ymin()],
            [bbox.get_xmax(), bbox.get_ymax()],
        );
        for segment in self.tree.locate_in_envelope_intersecting(&aabb) {
            if bbox.segment_intersects(&segment.start, &segment.end) {
                return true;
            }
        }
        false
    }

    pub fn hit_segment(&self, p1: &Point2D, p2: &Point2D) -> bool {
        let mut bbox = BoundingBox::new();
        bbox.update(p1);
        bbox.update(p2);
        let aabb = AABB::from_corners(
            [bbox.get_xmin(), bbox.get_ymin()],
            [bbox.get_xmax(), bbox.get_ymax()],
        );
        for segment in self.tree.locate_in_envelope_intersecting(&aabb) {
            if math::segments_intersect(&segment.start, &segment.end, p1, p2) {
                return true;
            }
        }
        false
    }
}

impl Polyline {
    pub fn to_attributes(&self) -> Attributes {
        let mut ret = Attributes::new();
        let mut dv = Vec::new();
        for p in &self.points {
            if dv.is_empty() {
                dv.push(format!("M{:.1},{:.1}", p.0.x, p.0.y));
            } else {
                dv.push(format!("L{:.1},{:.1}", p.0.x, p.0.y));
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
