use crate::label_placement::PointFeatureId;

use super::candidate::Candidate;
use super::candidate::Candidates;
use super::PointFeature;
use std::collections::BTreeMap;
use std::collections::BTreeSet;
use std::collections::HashMap;

// Each node is a PointFeature, represented by its id.
// Edges are modeled with a map.
type Node = PointFeatureId;
type CandidateMap = BTreeMap<PointFeatureId, Candidates>;
type Map = BTreeMap<Node, Vec<Node>>;

pub struct Graph {
    pub map: Map,
    pub candidates: CandidateMap,
    pub features: BTreeSet<PointFeature>,
    pub ordered_nodes: Vec<Node>,
    pub max_area: f64,
    pub used_area: f64,
}

impl Graph {
    pub fn new() -> Self {
        Self {
            map: Map::new(),
            candidates: CandidateMap::new(),
            features: BTreeSet::new(),
            ordered_nodes: Vec::new(),
            max_area: 0f64,
            used_area: 0f64,
        }
    }
    fn intersect(&self, a: &Node, b: &Node) -> bool {
        for ca in self.candidates.get(a).unwrap() {
            for cb in self.candidates.get(b).unwrap() {
                if ca.hit_other(&cb) {
                    return true;
                }
            }
        }
        false
    }
    fn compute_edges(&mut self, a: &Node) {
        let mut edges = Vec::new();
        for b in self.candidates.keys().clone() {
            if b == a {
                continue;
            }
            if self.intersect(&a, b) {
                edges.push(*b);
            }
        }
        self.map.insert(*a, edges);
    }

    pub fn build_map(&mut self) {
        self.map.clear();
        // O(n^2) if there are fixed number of candidates.
        let nodes: Vec<_> = self.candidates.keys().cloned().collect();
        for node in nodes {
            self.compute_edges(&node);
        }
        // check
        // for node in nodes {}
    }

    pub fn add_node(&mut self, a: &PointFeature, candidates: Candidates) {
        debug_assert!(!self.map.contains_key(&a.id));
        self.candidates.insert(a.id, candidates);
        self.ordered_nodes.push(a.id);
        self.features.insert(a.clone());
    }

    fn remove_node(&mut self, a: &Node) {
        // remove the node on the graph
        let neighbors = self.map.get(&a).unwrap().clone();
        for b in neighbors {
            self.map.get_mut(&b).unwrap().retain(|x| *x != *a);
        }
        self.map.remove(a);

        // cleanup backend data
        self.features.retain(|f| f.id != *a);
        self.ordered_nodes.retain(|node| node != a);
        self.candidates.remove(a);
    }

    fn find_feature(&self, node: &Node) -> Option<&PointFeature> {
        self.features.iter().find(|f| f.id == *node)
    }

    pub fn select(&mut self, a: &Node, selected: &Candidate) {
        // for all b connected to a
        let index = self
            .candidates
            .get(a)
            .unwrap()
            .iter()
            .position(|c| c == selected)
            .unwrap();
        {
            let feature = &self.find_feature(a).unwrap();
            log::trace!(
                "selected {} with area {:.1} [candidate {}] [{}]",
                feature.text(),
                feature.area(),
                index,
                selected.bbox().absolute()
            );
        }
        let neighbors = self.map.get(a).unwrap().clone();
        for b in neighbors {
            // remove candidates of b that overlap with the
            // selected a candidate
            let neighbors_candidates = self.candidates.get_mut(&b).unwrap();
            /*
                for cb in neighbors_candidates.clone() {
                    if selected.overlap(&cb.bbox) {
                        log::info!("remove candidate of {b} because of overlap: {}", cb.bbox);
                    }
            }
                */
            assert!(!neighbors_candidates.is_empty());
            let first = neighbors_candidates.first().unwrap().clone();
            neighbors_candidates.retain(|cb| !selected.hit_other(&cb));
            if neighbors_candidates.is_empty() {
                neighbors_candidates.push(first);
            }
        }
        // assert!((ba - aa).abs() < 1e-11);
        self.used_area += selected.bbox().area();
        // remove a
        self.remove_node(a);
    }
    pub fn max_node(&self) -> Node {
        *self.ordered_nodes.first().unwrap()
        /*
        assert!(!self.map.is_empty());
        let node = *self
            .map
            .iter()
            .map(|(node, edges)| (node, edges.len()))
            .max_by_key(|(_node, len)| *len)
            .unwrap()
            .0;
        node
        */
    }

    fn candidate_blocks_other(&self, node: &Node, candidate_index: usize, other: &Node) -> bool {
        let this_candidate = &self.candidates.get(node).unwrap()[candidate_index];
        let other_candidates = &self.candidates.get(other).unwrap();
        let other_has_label = !other_candidates.is_empty();
        if !other_has_label {
            return false;
        }
        for k in 0..other_candidates.len() {
            let other_candidate = &other_candidates[k];
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

    pub fn solve(&mut self) -> HashMap<Node, Candidate> {
        let mut ret = HashMap::new();
        while !self.map.is_empty() {
            //log::trace!("selecting..");
            let m = self.max_node();
            let target = &self.find_feature(&m).unwrap();
            if self.used_area + target.area() > self.max_area {
                log::trace!("remove {} because it is too large", target.text());
                self.remove_node(&m);
                continue;
            }
            /*log::trace!(
                "placing:{} ({} conflict edges) (priority {})",
                target.text(),
                self.map.get(&m).unwrap().len(),
                target.priority
            );*/
            match self.best_candidate_for_node(&m) {
                Some(best_index) => {
                    let candidates = self.candidates.get(&m).unwrap();
                    let best_candidate = candidates[best_index].clone();
                    ret.insert(m, best_candidate.clone());
                    self.select(&m, &best_candidate);
                }
                None => {
                    self.remove_node(&m);
                }
            }
        }
        ret
    }

    pub fn _debug(&self) {
        let nodes: Vec<_> = self.candidates.keys().cloned().collect();
        for node in nodes {
            let candidates = self.candidates.get(&node).unwrap();
            let rcand = self.map.get(&node);
            let edged_candidates = match rcand {
                None => 0,
                Some(list) => list.len(),
            };
            log::debug!(
                "[{}] => {} candidates (edges:{})",
                self.find_feature(&node).unwrap().text(),
                candidates.len(),
                edged_candidates
            );
        }
    }

    fn best_candidate_for_node(&self, node: &Node) -> Option<usize> {
        match self.candidates.get(node) {
            Some(candidates) => {
                if candidates.is_empty() {
                    return None;
                }
                // note: the candidates are sorted
                //log::trace!("select one candidate");
                for index in 0..candidates.len() {
                    match self.candidate_blocks_any(node, index) {
                        Some(_other_node) => {
                            /*log::trace!(
                                "[node:{node:2}] [candidate:{index:2}] blocks [{_other_node:2}]"
                            );*/
                            continue;
                        }
                        None => {}
                    }
                    /*log::trace!(
                        "[node:{node:2}] [candidate:{index:2}] it bests from #={}",
                        candidates.len()
                    );*/
                    return Some(index);
                }
                return None;
            }
            _ => {
                log::trace!("{node} has no candidate.");
                None
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        bbox::BoundingBox,
        label_placement::{Label, LabelBoundingBox, PointFeatureDrawing},
        math::Point2D,
    };

    use super::*;

    fn make_candidate(x: i32, y: i32, w: i32, h: i32) -> Candidate {
        Candidate::new(
            &LabelBoundingBox::new_absolute(
                &BoundingBox::minsize(Point2D::new(x as f64, y as f64), &(w as f64), &(h as f64)),
                &Point2D::zero(),
            ),
            &0.0,
            &0.0,
        )
    }

    #[test]
    fn test_graph_operations() {
        let _ = env_logger::try_init();
        // Create a new graph
        let mut graph = Graph::new();
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
            },
            input_point: None,
            link: None,
            id: 0,
        };
        for i in [0, 1, 2, 3] {
            let mut g = f.clone();
            g.id = i;
            graph.add_node(&g, candidates[i].clone());
        }
        graph.build_map();

        assert_eq!(graph.max_node(), 0);
        log::info!("select {} {}", 1, "cc1");
        graph.select(&2, &cc1);
        assert!(!graph.map.contains_key(&2));
        assert!(graph.candidates.get(&0).unwrap().len() == 1);
        assert!(graph.candidates.get(&1).unwrap().len() == 1);
        assert!(graph.candidates.get(&3).unwrap().len() == 1);
        assert!(graph.map.get(&0).unwrap().len() == 1);
        assert!(graph.map.get(&1).unwrap().len() == 1);
        log::info!("max node {}", graph.max_node());
    }
}
