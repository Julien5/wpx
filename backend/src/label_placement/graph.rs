use super::candidate::Candidate;
use super::candidate::Candidates;
use super::PointFeature;
use std::collections::BTreeMap;
use std::collections::HashMap;

type Node = usize;
type Edge = usize;
type Edges = Vec<Edge>;
type CandidateMap = BTreeMap<usize, Candidates>;
type Map = BTreeMap<Node, Edges>;

pub struct Graph {
    pub map: Map,
    pub candidates: CandidateMap,
    pub features: Vec<PointFeature>,
}

impl Graph {
    pub fn new() -> Self {
        Self {
            map: Map::new(),
            candidates: CandidateMap::new(),
            features: Vec::new(),
        }
    }
    fn intersect(&self, a: &Node, b: &Node) -> bool {
        for ca in self.candidates.get(a).unwrap() {
            for cb in self.candidates.get(b).unwrap() {
                if ca.bbox.overlap(&cb.bbox) {
                    return true;
                }
            }
        }
        false
    }
    fn compute_edges(&mut self, a: &Node) {
        let mut e = Edges::new();
        for b in self.candidates.keys().clone() {
            if b == a {
                continue;
            }
            if self.intersect(&a, b) {
                e.push(*b);
            }
        }
        self.map.insert(*a, e);
    }
    pub fn build_map(&mut self) {
        self.map.clear();
        // O(n^2) if there are fixed number of candidates.
        let nodes: Vec<_> = self.candidates.keys().cloned().collect();
        for node in nodes {
            self.compute_edges(&node);
        }
    }
    pub fn add_node(&mut self, a: Node, candidates: Candidates) {
        debug_assert!(!self.map.contains_key(&a));
        self.candidates.insert(a, candidates);
    }

    fn remove_node(&mut self, a: &Node) {
        self.candidates.remove(a);
        self.build_map();
    }

    pub fn select(&mut self, a: &Node, selected: &Candidate) {
        // for all b connected to a
        assert!(self
            .candidates
            .get(a)
            .unwrap()
            .iter()
            .position(|c| c == selected)
            .is_some());
        let neighbors = self.map.get(a).unwrap().clone();
        for b in neighbors {
            // remove candidates of b that overlap with the
            // selected a candidate
            let neighbors_candidates = self.candidates.get_mut(&b).unwrap();
            /*
                for cb in neighbors_candidates.clone() {
                    if selected.bbox.overlap(&cb.bbox) {
                        log::info!("remove candidate of {b} because of overlap: {}", cb.bbox);
                    }
            }
                */
            assert!(!neighbors_candidates.is_empty());
            let first = neighbors_candidates.first().unwrap().clone();
            neighbors_candidates.retain(|cb| !selected.bbox.overlap(&cb.bbox));
            if neighbors_candidates.is_empty() {
                neighbors_candidates.push(first);
            }
        }
        // remove a
        self.remove_node(a);
    }
    pub fn max_node(&self) -> Node {
        assert!(!self.map.is_empty());
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
        let this_candidate = &self.candidates.get(node).unwrap()[candidate_index];
        let other_candidates = &self.candidates.get(other).unwrap();
        let other_has_label = !other_candidates.is_empty();
        if !other_has_label {
            return false;
        }
        for k in 0..other_candidates.len() {
            let other_candidate = &other_candidates[k];
            if !other_candidate.bbox.overlap(&this_candidate.bbox) {
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
            /*log::trace!(
                "placing:{} ({} conflict edges) (priority {})",
                &self.features[m].label.text,
                self.map.get(&m).unwrap().len(),
                &self.features[m].priority
            );*/
            match self.best_candidate_for_node(&m) {
                Some(best_index) => {
                    let candidates = self.candidates.get(&m).unwrap();
                    let best_candidate = candidates[best_index].clone();
                    ret.insert(m, best_candidate.clone());
                    self.select(&m, &best_candidate);
                }
                None => {
                    assert!(self.features[m].placement_order > 5);
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
                self.features[node].text(),
                candidates.len(),
                edged_candidates
            );
        }
    }

    fn best_candidate_for_node(&self, node: &Node) -> Option<usize> {
        match self.candidates.get(node) {
            Some(candidates) => {
                if candidates.is_empty() {
                    assert!(self.features[*node].placement_order > 5);
                    return None;
                }
                let mut sorted: Vec<_> = (0..candidates.len()).collect();
                //log::trace!("sort candidates..");
                sorted.sort_by(|i, j| {
                    let ci = &candidates[*i];
                    let cj = &candidates[*j];
                    assert!(ci.partial_cmp(cj).is_some());
                    ci.partial_cmp(cj).unwrap()
                });
                //log::trace!("select one candidate");
                for index in 0..sorted.len() {
                    match self.candidate_blocks_any(node, index) {
                        Some(_other_node) => {
                            /*log::info!(
                                "[node:{node:2}] [candidate:{index:2}] blocks [{_other_node:2}]"
                            );*/
                            continue;
                        }
                        None => {}
                    }
                    /*log::info!(
                        "[node:{node:2}] [candidate:{index:2}] it bests from #={}",
                        candidates.len()
                    );*/
                    return Some(index);
                }
                if self.features[*node].placement_order <= 5 {
                    //log::info!("all candidates of {node} block some other => take the first one");
                    return Some(0);
                }
                return None;
            }
            _ => {
                //log::info!("{node} has no candidate.");
                None
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::label_placement::LabelBoundingBox;

    use super::*;

    fn make_candidate(x: i32, y: i32, w: i32, h: i32) -> Candidate {
        Candidate::new(
            LabelBoundingBox::new_tlwh((x as f64, y as f64), w as f64, h as f64),
            0.,
            0.,
        )
    }

    #[test]
    fn test_graph_operations() {
        // Create a new graph
        let mut graph = Graph::new();
        let mut CA = Candidates::new();
        let mut CB = Candidates::new();
        let mut CC = Candidates::new();
        let mut CD = Candidates::new();
        let ca1 = make_candidate(0, 0, 2, 2);
        let ca2 = make_candidate(2, 2, 3, 2);
        let cb1 = make_candidate(1, 0, 3, 2);
        let cb2 = make_candidate(4, 2, 3, 2);
        assert!(ca2.bbox.overlap(&cb2.bbox));
        CA.push(ca1);
        CA.push(ca2);
        CB.push(cb1.clone());
        CB.push(cb2.clone());
        let cc1 = make_candidate(3, 3, 2, 3);
        CC.push(cc1.clone());
        let cc2 = make_candidate(4, 3, 2, 3);
        CC.push(cc2.clone());
        CC.push(make_candidate(3, 8, 2, 3));
        CD.push(make_candidate(3, 9, 2, 3));
        graph.add_node(0, CA);
        graph.add_node(1, CB);
        graph.add_node(2, CC);
        graph.add_node(3, CD);
        graph.build_map();

        assert!(graph.max_node() == 2);
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
