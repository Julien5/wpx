pub mod candidate;
mod draw_graph;
pub mod drawings;
pub mod features;
pub mod graph;
pub mod labelboundingbox;
pub mod obstacle;

use super::label_placement::features::*;
use crate::bbox::BoundingBox;
use crate::label_placement::labelboundingbox::LabelBoundingBox;
use crate::label_placement::obstacle::Obstacles;
use crate::math::distance2;
use crate::math::Point2D;

use candidate::Candidate;
use candidate::Candidates;
use graph::Graph;

use std::collections::BTreeMap;

pub const FONTSIZE: f64 = 14f64;

pub trait CandidatesGenerator {
    // The hardness indicates how hard the feature we should try to place this
    // features. Number between 0 and 10.
    fn gen(&self, feature: &PointFeature, obstacles: &Obstacles) -> Vec<Candidate>;
}

fn build_graph(
    features: &PointFeatures,
    gen: &dyn CandidatesGenerator,
    obstacles: &Obstacles,
    debug_graphic_dir: Option<String>,
) -> Graph {
    let mut ret = Graph::new(obstacles.clone(), debug_graphic_dir);
    let candidates = candidate::utils::generate(gen, features, obstacles);
    // since the graph is undirected, we could probably speed up
    // edge computation. TODO: use petgraph.
    for (k, feature) in features.points.iter().enumerate() {
        ret.add_node(feature, candidates[k].clone());
    }
    ret.build_map();
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
        _obstacles: &Obstacles,
        packets: &mut Vec<PointFeatures>,
    ) -> Vec<PointFeature> {
        let mut ret = Vec::new();
        assert_eq!(results.len(), packets.len());
        for (result_index, result) in results.iter().enumerate() {
            let packet = &mut packets[result_index];
            for (feature_index, feature) in packet.points.iter_mut().enumerate() {
                if result.placed_indices.contains_key(&feature_index) {
                    let bbox = result.placed_indices.get(&feature_index).unwrap();
                    feature.place_label(&bbox);
                    feature.make_link(&_obstacles);
                    ret.push(feature.clone());
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
        let cboxes = cardinal_boxes(&feature.center(), feature.width(), feature.height());
        let first = cboxes.first().unwrap();
        let candidate = Candidate::new(first);
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
    obstacles: &mut Obstacles,
    debug_graphic_dir: Option<String>,
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
            let mut graph = build_graph(features, gen, &obstacles, debug_graphic_dir);
            let result = graph.solve();
            *obstacles = result.obstacles;
            result.selected
        }
        true => place_quick_best_candidates(features, obstacles),
    };
    for (k, feature) in features.points.iter().enumerate() {
        let target_text = feature.text();
        if target_text.is_empty() {
            continue;
        }
        let best_candidate = best_candidates.get(&k);
        match best_candidate {
            Some(candidate) => {
                //log::trace!("candidate: {}", candidate.bbox().relative());
                ret.placed_indices.insert(k, candidate.bbox().clone());
            }
            _ => { /* log::info!("failed to place [{}]", feature.label.text); */ }
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
    debug_graphic_dir: Option<String>,
) -> (Vec<PlacementResult>, Obstacles) {
    let mut ret = Vec::new();
    let mut obstacles = Obstacles::new(bbox, *max_area_ratio);
    obstacles.polylines = vec![polyline.clone()];
    for (idx, packet) in packets.iter().enumerate() {
        let kind = match packet.points.first() {
            Some(feature) => format!("{:?}", feature.id()),
            None => format!("unknown"),
        };
        log::trace!(
            "subset packet [{}] ({}) features:{} obstacles:{}",
            idx,
            kind,
            packet.points.len(),
            obstacles.bboxes().len()
        );
        let results = place_subset(&packet, gen, &mut obstacles, debug_graphic_dir.clone());
        ret.push(results);
    }
    assert_eq!(ret.len(), packets.len());
    (ret, obstacles)
}

fn make(bbox0: &BoundingBox, translation: &Point2D, center: &Point2D) -> LabelBoundingBox {
    LabelBoundingBox::new_relative(&bbox0.make_translate(&translation), center)
}

pub fn cardinal_boxes(center: &Point2D, width: f64, height: f64) -> Vec<LabelBoundingBox> {
    let mut ret = Vec::new();
    let epsilon = 2f64;
    let dx = 2f64 * epsilon + width;
    let dy = 2f64 * epsilon + height;
    let topright = BoundingBox::minsize(Point2D::new(epsilon, -epsilon - height), width, height);

    ret.push(make(&topright, &Point2D::new(0.0, 0.0), center));
    ret.push(make(&topright, &Point2D::new(-dx, 0.0), center));
    ret.push(make(&topright, &Point2D::new(-dx, dy), center));
    ret.push(make(&topright, &Point2D::new(0.0, dy), center));

    let bbox_right = BoundingBox::minsize(Point2D::new(epsilon, -height / 2.0), width, height);
    let bbox_up =
        BoundingBox::minsize(Point2D::new(-width / 2.0, -epsilon - height), width, height);

    ret.push(make(&bbox_right, &Point2D::new(0.0, 0.0), center));
    ret.push(make(&bbox_up, &Point2D::new(0.0, 0.0), center));
    ret.push(make(&bbox_right, &Point2D::new(-dx, 0.0), center));
    ret.push(make(&bbox_up, &Point2D::new(0.0, dy), center));

    ret
}

pub fn cardinal_boxes_profile(center: &Point2D, width: f64, height: f64) -> Vec<LabelBoundingBox> {
    let mut ret = Vec::new();
    let epsilon = 5f64;
    let dx = 2f64 * epsilon + width;
    let dy = 2f64 * epsilon + height;
    let topright = BoundingBox::minsize(Point2D::new(epsilon, -epsilon - height), width, height);

    let bbox_right = BoundingBox::minsize(Point2D::new(epsilon, -height / 2.0), width, height);
    let bbox_up =
        BoundingBox::minsize(Point2D::new(-width / 2.0, -epsilon - height), width, height);

    ret.push(make(&bbox_up, &Point2D::new(0.0, 0.0), center));
    ret.push(make(&bbox_up, &Point2D::new(0.0, dy), center));

    ret.push(make(&topright, &Point2D::new(0.0, 0.0), center));
    ret.push(make(&topright, &Point2D::new(-dx, 0.0), center));
    ret.push(make(&topright, &Point2D::new(-dx, dy), center));
    ret.push(make(&topright, &Point2D::new(0.0, dy), center));

    ret.push(make(&bbox_right, &Point2D::new(0.0, 0.0), center));
    ret.push(make(&bbox_right, &Point2D::new(-dx, 0.0), center));

    ret
}

#[allow(dead_code)]
pub fn far_cardinal_boxes(
    center: &Point2D,
    width: f64,
    height: f64,
    distance: f64,
) -> Vec<LabelBoundingBox> {
    let mut ret = Vec::new();
    let bbox_right = BoundingBox::minsize(Point2D::new(distance, -height / 2.0), width, height);
    let bbox_up = BoundingBox::minsize(
        Point2D::new(-width / 2.0, -distance - height),
        width,
        height,
    );
    ret.push(make(&bbox_right, &Point2D::new(0.0, 0.0), center));
    ret.push(make(&bbox_up, &Point2D::new(0.0, 0.0), center));
    let dx = 2f64 * distance + width;
    let dy = 2f64 * distance + height;
    ret.push(make(&bbox_right, &Point2D::new(-dx, 0.0), center));
    ret.push(make(&bbox_up, &Point2D::new(0.0, dy), center));

    ret
}

#[allow(dead_code)]
pub fn far_boxes(target: &Point2D, width: f64, height: f64, level: usize) -> Vec<LabelBoundingBox> {
    let mut ret = Vec::new();
    let d = ((level + 2) as f64) * height;
    let stepsize = height;

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
