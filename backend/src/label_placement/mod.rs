pub mod bbox;
pub mod candidate;
pub mod drawings;
mod graph;
pub mod prioritize;
mod stroke;
pub use bbox::LabelBoundingBox;
pub mod features;
use candidate::Candidate;
use candidate::Candidates;
use graph::Graph;

use std::collections::BTreeMap;

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

use crate::bbox::BoundingBox;
use crate::label_placement::features::*;
use crate::math::distance2;
use crate::math::Point2D;

fn build_graph(
    features: &Vec<PointFeature>,
    gen: &dyn CandidatesGenerator,
    obstacles: &Obstacles,
) -> Graph {
    let mut ret = Graph::new(obstacles.drawingbox.bbox.clone());
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
    pub placed_indices: BTreeMap<features::PointFeatureId, LabelBoundingBox>,
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

    let rtree = build_pointfeature_rtree(&features);

    let quick = false;
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
