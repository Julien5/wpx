pub mod candidate;
pub mod drawings;
pub mod features;
pub mod graph;
pub mod labelboundingbox;
pub mod prioritize;

mod stroke;

use super::label_placement::features::*;
use crate::bbox::BoundingBox;
use crate::inputpoint::InputType;
use crate::label_placement::labelboundingbox::LabelBoundingBox;
use crate::math::distance2;
use crate::math::Point2D;

use candidate::Candidate;
use candidate::Candidates;
use graph::Graph;

use std::collections::BTreeMap;

pub trait CandidatesGenerator {
    fn gen(&self, feature: &PointFeature) -> Vec<LabelBoundingBox>;
}

fn build_graph(
    features: &PointFeatures,
    gen: &dyn CandidatesGenerator,
    obstacles: &Obstacles,
) -> Graph {
    let mut ret = Graph::new(obstacles.drawingbox.bbox.clone());
    let candidates = candidate::utils::generate(gen, features, obstacles);
    // since the graph is undirected, we could probably speed up
    // edge computation. TODO: use petgraph.
    for k in 0..features.points.len() {
        let feature = &features.points[k];
        let candidates = candidates[k].clone();
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
    pub placed_indices: BTreeMap<features::PointFeatureId, LabelBoundingBox>,
}

impl PlacementResult {
    // not clean: either packets should not immutable or we dont need a return value
    pub fn apply(
        results: &Vec<PlacementResult>,
        obstacles: &Obstacles,
        packets: &mut Vec<PointFeatures>,
    ) -> Vec<PointFeature> {
        let mut ret = Vec::new();
        assert_eq!(results.len(), packets.len());
        for kr in 0..results.len() {
            let result = &results[kr];
            let packet = &mut packets[kr];
            for kp in 0..packet.points.len() {
                let feature = &mut packet.points[kp];
                if result.placed_indices.contains_key(&kp) {
                    let bbox = result.placed_indices.get(&kp).unwrap().clone();
                    feature.place_label(&bbox);
                    feature._make_link(obstacles);
                    ret.push(feature.clone());
                } else {
                    if feature.input_point.as_ref().unwrap().kind() != InputType::OSM {
                        ret.push(feature.clone());
                    }
                }
            }
        }
        ret
    }
}

fn place_quick_best_candidates(
    features: &PointFeatures,
    obstacles: &Obstacles,
) -> BTreeMap<PointFeatureId, Candidate> {
    let mut map_candidate = BTreeMap::new();
    let mut available = obstacles.available_area();
    for k in 0..features.points.len() {
        let feature = &features.points[k];
        let cboxes = cardinal_boxes(&feature.center(), &feature.width(), &feature.height());
        let first = cboxes.first().unwrap();
        let candidate = Candidate::new(first, &1f64, &1f64);
        if available < candidate.bbox().area() {
            break;
        }
        available -= candidate.bbox().area();
        map_candidate.insert(k, candidate);
    }
    map_candidate
}

fn place_subset(
    features: &PointFeatures,
    gen: &dyn CandidatesGenerator,
    obstacles: &Obstacles,
) -> PlacementResult {
    let mut ret = PlacementResult {
        placed_indices: BTreeMap::new(),
    };
    if features.points.is_empty() {
        return ret;
    }
    let quick = false;
    let best_candidates = match quick {
        false => {
            let mut graph = build_graph(features, gen, &obstacles);
            // graph.print_graph();
            graph.solve()
        }
        true => place_quick_best_candidates(features, obstacles),
    };
    //log::trace!("solve label graph [{}]", graph.map.len(),);

    log::trace!("results:");
    for k in 0..features.points.len() {
        let feature = &features.points[k];
        let target_text = feature.text();
        if target_text.is_empty() {
            continue;
        }
        let best_candidate = best_candidates.get(&k);
        match best_candidate {
            Some(candidate) => {
                log::trace!("index:{}", k);
                log::trace!("text: {}", target_text);
                log::trace!("candidate: {}", candidate.bbox().relative());
                ret.placed_indices.insert(k, candidate.bbox().clone());
            }
            _ => {
                log::trace!("failed to find any candidate for [{}]", target_text);
            }
        }
    }
    ret
}

pub fn place_labels(
    packets: &Vec<PointFeatures>,
    gen: &dyn CandidatesGenerator,
    bbox: &BoundingBox,
    polyline: &Polyline,
    max_area_ratio: &f64,
) -> (Vec<PlacementResult>, Obstacles) {
    let mut ret = Vec::new();
    let mut obstacles = Obstacles {
        drawingbox: DrawingArea {
            bbox: bbox.clone(),
            max_area_ratio: *max_area_ratio,
        },
        polylines: vec![polyline.clone()],
        bboxes: Vec::new(),
    };
    for packet in packets {
        log::trace!(
            "[a] features:{} obstacles:{}",
            packet.points.len(),
            obstacles.bboxes.len()
        );
        let results = place_subset(&packet, gen, &obstacles);
        for (_k, bbox) in &results.placed_indices {
            obstacles.bboxes.push(bbox.absolute().clone());
        }
        ret.push(results);
    }
    assert_eq!(ret.len(), packets.len());
    (ret, obstacles)
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
