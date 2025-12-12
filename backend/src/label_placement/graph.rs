use super::candidate::utils;
use super::candidate::Candidate;
use super::candidate::Candidates;
use super::features::PointFeature;
use super::features::PointFeatureId;
use crate::bbox::quadtree::QuadTree;
use crate::bbox::BoundingBox;
use std::collections::BTreeMap;
use std::collections::BTreeSet;

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
    pub map: Map,
    pub ordered_nodes: Vec<Node>,
    pub max_area: f64,
    pub used_area: f64,
    pub tree: QuadTree<PointFeatureId>,
    pub nodes: Vec<NodeData>,
}

impl Graph {
    pub fn new(area: BoundingBox) -> Self {
        Self {
            map: Map::new(),
            ordered_nodes: Vec::new(),
            max_area: 0f64,
            used_area: 0f64,
            tree: QuadTree::new(area),
            nodes: Vec::new(),
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
        log::trace!("building edges for {} nodes", self.ordered_nodes.len());
        let mut count = 0;
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
                    count += 1;
                }
            }
            self.map.insert(node1, edges);
        }
        log::trace!(
            "built {} edges for {} nodes",
            count,
            self.ordered_nodes.len()
        );

        // note: self.tree is not needed anymore.
    }

    pub fn _print_node(&self, node: &Node) {
        let feature = &self.nodes[*node].feature;
        log::trace!("node: {}", node);
        log::trace!("  - text: {}", feature.text());
        log::trace!("  - size: {:.1}x{:.1}", feature.width(), feature.height());

        let candidates = &self.nodes[*node].candidates;
        log::trace!("  - candidates: {}", candidates.len());
        for candidate in candidates {
            let bbox = candidate.bbox().relative();
            log::trace!("      {:?}", bbox);
        }
    }

    pub fn _print_graph(&self) {
        for n in &self.ordered_nodes {
            self._print_node(n);
        }
    }

    pub fn add_node(&mut self, _a: &PointFeature, candidates: Candidates) {
        assert_eq!(self.ordered_nodes.len(), self.nodes.len());

        let data = NodeData {
            feature: _a.clone(),
            bbox: utils::candidates_bounding_box(&candidates),
            candidates: candidates.clone(),
        };
        self.nodes.push(data);

        let k = self.nodes.len() - 1;
        debug_assert!(!self.map.contains_key(&k));
        self.ordered_nodes.push(k);
        let cbb = utils::candidates_bounding_box(&candidates);
        self.tree.insert(&cbb, k);
        log::trace!("add edge list at {}", k);
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

    pub fn select(&mut self, a: &Node, selected: &Candidate) {
        // for all b connected to a
        let index = self.nodes[*a]
            .candidates
            .iter()
            .position(|c| c == selected)
            .unwrap();
        {
            let feature = &self.nodes[*a].feature;
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
            let neighbors_candidates = &mut self.nodes[b].candidates;
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
        /*self.ordered_nodes.first().unwrap()*/
        /*assert!(!self.map.is_empty());*/
        let node = *self
            .map
            .iter()
            .map(|(node, edges)| (node, edges.len()))
            .max_by_key(|(_node, len)| *len)
            .unwrap()
            .0;
        node
    }

    fn candidate_blocks_other(&self, node: &Node, candidate_index: usize, other: &Node) -> bool {
        let this_candidate = &self.nodes[*node].candidates[candidate_index];
        let other_candidates = &self.nodes[*other].candidates;
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

    pub fn solve(&mut self) -> BTreeMap<Node, Candidate> {
        let mut ret = BTreeMap::new();
        while !self.map.is_empty() {
            let m = self.max_node();
            log::trace!("selecting..{}", m);
            let target = &self.nodes[m].feature;
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
                    let candidates = &self.nodes[m].candidates;
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
        let nodes: Vec<_> = (0..self.nodes.len()).collect();
        for node in nodes {
            let _candidates = &self.nodes[node].candidates;
            let rcand = self.map.get(&node);
            let _edged_candidates = match rcand {
                None => 0,
                Some(list) => list.len(),
            };
            /*
                log::trace!(
                    "[{}] => {} candidates (edges:{})",
                    self.find_feature(&node).unwrap().text(),
                    candidates.len(),
                    edged_candidates
            );
                */
        }
    }

    fn best_candidate_for_node(&self, node: &Node) -> Option<usize> {
        let candidates = &self.nodes[*node].candidates;
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
        let mut graph = Graph::new(BoundingBox::minmax(
            Point2D::zero(),
            Point2D::new(10f64, 10f64),
        ));
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
        log::info!("select {} {}", 1, "cc1");
        log::info!("features:{}", graph.nodes.len());
        graph.select(&2, &cc1);
        assert!(!graph.map.contains_key(&2));
        assert!(graph.nodes[0].candidates.len() == 1);
        assert!(graph.nodes[0].candidates.len() == 1);
        assert!(graph.nodes[0].candidates.len() == 1);
        assert!(graph.map.get(&0).unwrap().len() == 1);
        assert!(graph.map.get(&1).unwrap().len() == 1);
        log::info!("max node {}", graph.max_node());
    }
}
