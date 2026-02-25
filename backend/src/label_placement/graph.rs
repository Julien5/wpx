use super::candidate::utils;
use super::candidate::Candidate;
use super::candidate::Candidates;
use super::features::PointFeature;
use super::features::PointFeatureId;
use crate::bbox::quadtree::QuadTree;
use crate::bbox::BoundingBox;

use crate::label_placement::draw_graph::Graphic;
use crate::label_placement::obstacle::Obstacles;
use crate::math::Point2D;
use std::collections::BTreeMap;
use std::collections::BTreeSet;

#[allow(unused_imports)]
use crate::label_placement::draw_graph;

// Each node is a PointFeature, represented by its id.
// Edges are modeled with a map.
type Node = PointFeatureId;
type Map = BTreeMap<Node, BTreeSet<Node>>;

pub struct NodeData {
    pub feature: PointFeature,
    pub bbox: BoundingBox,
    pub candidates: Candidates,
}

pub struct Graph {
    map: Map,
    ordered_nodes: Vec<Node>,
    tree: QuadTree<PointFeatureId>,
    nodes: Vec<NodeData>,
    obstacles: Obstacles,
    debug_graphic_dir: Option<String>,
}

pub struct GraphResult {
    pub selected: BTreeMap<Node, Candidate>,
    pub obstacles: Obstacles,
}

impl Graph {
    pub fn new(obstacles: Obstacles, debug_graphic_dir: Option<String>) -> Self {
        let area = obstacles.drawingbox.bbox.clone();
        let dir = match debug_graphic_dir {
            Some(d) => Some(draw_graph::newdir(&d)),
            None => None,
        };
        Self {
            map: Map::new(),
            ordered_nodes: Vec::new(),
            tree: QuadTree::new(area.clone()),
            nodes: Vec::new(),
            obstacles: obstacles,
            debug_graphic_dir: dir,
        }
    }

    fn intersect(&self, a: &Node, b: &Node) -> bool {
        for ca in &self.nodes[*a].candidates {
            for cb in &self.nodes[*b].candidates {
                if ca.hit_other(&cb) {
                    return true;
                }
            }
        }
        false
    }

    pub fn build_map(&mut self) {
        for node1 in 0..self.nodes.len() {
            let cbb = &self.nodes[node1].bbox;
            let mut hits = Vec::new();
            let mut edges = BTreeSet::new();
            self.tree.query(cbb, &mut hits);
            for node2 in hits {
                if node1 == *node2 {
                    continue;
                }
                if self.intersect(&node1, node2) {
                    edges.insert(*node2);
                }
            }
            self.map.insert(node1, edges);
        }
        // note: self.tree is not needed anymore.
        self.draw_graph(&"build");
    }

    pub fn _print_node(&self, node: &Node) {
        let feature = &self.nodes[*node].feature;
        log::info!("node: {}", node);
        log::info!("  - text: {}", feature.text());
        log::info!("  - size: {:.1}x{:.1}", feature.width(), feature.height());

        let candidates = &self.nodes[*node].candidates;
        log::info!("  - candidates: {}", candidates.len());
        for candidate in candidates {
            let bbox = candidate.bbox().relative();
            log::info!("      {:?}", bbox);
        }
    }

    fn make_graphic(&self) -> Graphic {
        let mut graphic = Graphic::new(self.debug_graphic_dir.as_ref().unwrap().clone());
        for bbox in &self.obstacles.bboxes() {
            graphic.add_boundingbox(bbox, "red", 3);
        }
        for (node, edges) in &self.map {
            let nodedata = &self.nodes[*node];
            let text = nodedata.feature.text();
            let p1 = nodedata.feature.center();
            let p = p1 + crate::math::Point2D::new(3f64, -3f64);
            graphic.add_text(&p, &text);
            for candidate in &nodedata.candidates {
                graphic.add_boundingbox(&candidate.bbox().absolute(), "gray", 1i32);
            }

            graphic.add_dot(&p1);
            for n2 in edges {
                let p2 = self.nodes[*n2].feature.center();
                graphic.add_stroke(&p1, &p2);
            }
        }
        graphic
    }

    fn draw_graph(&self, marker: &str) {
        if self.debug_graphic_dir.is_none() {
            return;
        }
        let g = self.make_graphic();
        g.save(marker);
    }

    #[allow(dead_code)]
    pub fn print_graph(&self) {
        for n in &self.ordered_nodes {
            self._print_node(n);
        }
    }

    pub fn add_node(&mut self, feature: &PointFeature, candidates: Candidates) {
        assert_eq!(self.ordered_nodes.len(), self.nodes.len());

        let data = NodeData {
            feature: feature.clone(),
            bbox: utils::candidates_bounding_box(&candidates),
            candidates: candidates.clone(),
        };
        self.nodes.push(data);

        let k = self.nodes.len() - 1;
        debug_assert!(!self.map.contains_key(&k));
        self.ordered_nodes.push(k);
        let cbb = utils::candidates_bounding_box(&candidates);
        self.tree.insert(&cbb, k);
        self.map.insert(k, BTreeSet::new());
        assert_eq!(self.ordered_nodes.len(), self.nodes.len());
        assert_eq!(self.nodes.len(), self.map.len());
    }

    fn remove_node(&mut self, a: &Node) {
        // remove the node on the graph
        let neighbors = self.map.get(&a).unwrap().clone();
        for b in neighbors {
            self.map.get_mut(&b).unwrap().retain(|x| *x != *a);
        }
        self.map.remove(a);

        // cleanup backend data
        self.ordered_nodes.retain(|node| node != a);

        // We could remove candidates from self.tree for completedness,
        // but this is not necessary since solve() does not read it.
        // After build_map(), this tree is not read.
    }

    pub fn update_graph(&mut self, a: &Node, selected: &Candidate) {
        let neighbors = self.map.get(a).unwrap().clone();
        let nodedata = &self.nodes[*a];
        let center = nodedata.feature.center();
        let bboxcenter = selected.bbox().absolute().center();
        let mut selected_large = selected.bbox().absolute().clone();
        if !selected.is_external() {
            let aux = Point2D::point_on_segment_from_end(&bboxcenter, &center, 5.0);
            selected_large.update(&aux);
        }
        for b in neighbors {
            let neighbors_candidates = &mut self.nodes[b].candidates;
            // remove candidates that intersect with the selected candidate
            neighbors_candidates.retain(|cb| !selected_large.overlap(&cb.bbox().absolute()));
            if neighbors_candidates.is_empty() {
                log::info!(
                    "graph removed [{}] because of overlapping with [{}] (and others)",
                    self.nodes[b].feature.id(),
                    self.nodes[*a].feature.id()
                );
            }
        }
        self.draw_graph(&format!("{:03}-4-update", a));
        // Track the placed candidate for density queries
        self.obstacles.push_bbox(selected_large);

        // remove a
        self.draw_graph(&format!("{:03}-5-update", a));
        self.remove_node(a);
        self.draw_graph(&format!("{:03}-6-update", a));
    }

    pub fn max_node(&self) -> Node {
        // more predictable
        *self.ordered_nodes.first().unwrap()
        /* There is a flaw with using the node with max degree.
         * A node with higher priority may not be placed because there is
         * no more place when its turn comes.
         */
        /*assert!(!self.map.is_empty());*/
        /*let node = *self
            .map
            .iter()
            .map(|(node, edges)| (node, edges.len()))
            .max_by_key(|(_node, len)| *len)
            .unwrap()
            .0;
        node*/
    }

    fn candidate_blocks_other(&self, node: &Node, candidate_index: usize, other: &Node) -> bool {
        let this_candidate = &self.nodes[*node].candidates[candidate_index];
        let other_candidates = &self.nodes[*other].candidates;
        let other_has_label = !other_candidates.is_empty();
        if !other_has_label {
            return false;
        }
        // Check if this candidate blocks ALL candidates of the other node
        for other_candidate in other_candidates {
            if !other_candidate.hit_other(&this_candidate) {
                return false;
            }
        }
        true
    }

    fn candidate_blocks_any(&self, node: &Node, candidate_index: usize) -> Option<Node> {
        let nodes: Vec<_> = self.map.keys().collect();
        for other in nodes {
            if other == node {
                continue;
            }
            if self.candidate_blocks_other(node, candidate_index, other) {
                return Some(*other);
            }
        }
        None
    }

    pub fn solve(&mut self) -> GraphResult {
        let mut ret = BTreeMap::new();
        while !self.map.is_empty() {
            let m = self.max_node();
            match self.best_candidate_for_node(&m) {
                Some(best_index) => {
                    let candidates = &self.nodes[m].candidates;
                    let best_candidate = candidates[best_index].clone();
                    ret.insert(m, best_candidate.clone());
                    self.update_graph(&m, &best_candidate);
                }
                None => {
                    self.remove_node(&m);
                }
            }
        }
        GraphResult {
            selected: ret,
            obstacles: self.obstacles.clone(),
        }
    }

    fn best_candidate_for_node(&self, node: &Node) -> Option<usize> {
        let candidates = &self.nodes[*node].candidates;
        if candidates.is_empty() {
            log::info!("no candidate found for {}", self.nodes[*node].feature.id(),);
            return None;
        }
        // Note: the candidates are sorted by priority
        let mut nblock_other = 0;
        for index in 0..candidates.len() {
            let candidate = &candidates[index];

            if self.debug_graphic_dir.is_some() {
                let mut graphic = self.make_graphic();
                graphic.add_boundingbox(&candidate.bbox().absolute(), "black", 4);
                graphic.save(&format!("{:03}-1-check", node));
            }

            match self.candidate_blocks_any(node, index) {
                Some(_other_node) => {
                    nblock_other += 1;
                    continue;
                }
                None => {}
            }

            if self.debug_graphic_dir.is_some() {
                let mut graphic = self.make_graphic();
                graphic.add_boundingbox(&candidate.bbox().absolute(), "black", 4);
                graphic.save(&format!("{:03}-3-found", node));
            }

            return Some(index);
        }

        log::trace!(
            "could not find any good candidate for [{}] ({} block other) => take first candidate",
            self.nodes[*node].feature.id(),
            nblock_other,
        );

        if self.debug_graphic_dir.is_some() {
            let graphic = self.make_graphic();
            graphic.save(&format!("{:03}-3-fail", node));
        }
        // Even though all candidate block some other, we must return an index.
        // Choose the first one, assuming they are ordered by priority.
        return Some(0);
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        bbox::BoundingBox,
        label_placement::{features::FONTSIZE, Label, LabelBoundingBox, PointFeatureDrawing},
        math::Point2D,
    };

    use super::*;

    fn make_candidate(x: i32, y: i32, w: i32, h: i32) -> Candidate {
        Candidate::new(&LabelBoundingBox::new_absolute(
            &BoundingBox::minsize(Point2D::new(x as f64, y as f64), w as f64, h as f64),
            &Point2D::zero(),
        ))
    }

    #[test]
    fn test_graph_operations() {
        let _ = env_logger::try_init();
        // Create a new graph
        let area = BoundingBox::minmax(Point2D::zero(), Point2D::new(10f64, 10f64));
        let obstacles = Obstacles::new(&area, 1.0);
        let mut graph = Graph::new(obstacles, None);
        let mut ca = Candidates::new();
        let mut cb = Candidates::new();
        let mut cc = Candidates::new();
        let mut cd = Candidates::new();
        let mut candidates = Vec::new();
        let ca1 = make_candidate(0, 0, 2, 2);
        let ca2 = make_candidate(2, 2, 3, 2);
        let cb1 = make_candidate(1, 0, 3, 2);
        let cb2 = make_candidate(4, 2, 3, 2);
        assert!(ca2.hit_other(&cb2));
        ca.push(ca1);
        ca.push(ca2);
        cb.push(cb1.clone());
        cb.push(cb2.clone());
        let cc1 = make_candidate(3, 3, 2, 3);
        cc.push(cc1.clone());
        let cc2 = make_candidate(4, 3, 2, 3);
        cc.push(cc2.clone());
        cc.push(make_candidate(3, 8, 2, 3));
        cd.push(make_candidate(3, 9, 2, 3));
        candidates.push(ca);
        candidates.push(cb);
        candidates.push(cc);
        candidates.push(cd);
        let zero = Point2D::new(0f64, 0f64);
        let f = PointFeature {
            circle: PointFeatureDrawing {
                group: svg::node::element::Group::new(),
                center: zero.clone(),
            },
            label: Label {
                id: "id0".to_string(),
                bbox: LabelBoundingBox::new_relative(
                    &BoundingBox::minmax(zero.clone(), zero.clone()),
                    &Point2D::zero(),
                ),
                text: String::new(),
                fontsize: FONTSIZE,
                fontweight: "normal".to_string(),
                fontstyle: "normal".to_string(),
            },
            input_point: None,
            link: None,
            xmlid: 0,
        };
        let mut features = Vec::new();
        for i in [0, 1, 2, 3] {
            let mut g = f.clone();
            g.xmlid = i;
            features.push(g);
        }
        for i in [0, 1, 2, 3] {
            graph.add_node(&features[i], candidates[i].clone());
        }
        graph.build_map();

        assert_eq!(graph.max_node(), 0);
        graph.update_graph(&2, &cc1);
        assert!(!graph.map.contains_key(&2));
        assert_eq!(graph.nodes[0].candidates.len(), 1);
        assert_eq!(graph.nodes[1].candidates.len(), 1);
        assert_eq!(graph.map.get(&0).unwrap().len(), 1);
        assert_eq!(graph.map.get(&1).unwrap().len(), 1);
    }
}
