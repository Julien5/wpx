use super::bbox::LabelBoundingBox;

#[derive(Clone)]
pub struct Candidate {
    pub bbox: LabelBoundingBox,
    pub dtarget: f64,
    pub dothers: f64,
}

impl Candidate {
    pub fn new(bbox: LabelBoundingBox, dtarget: f64, dothers: f64) -> Candidate {
        Candidate {
            bbox,
            dtarget,
            dothers,
        }
    }
    fn _intersect(&self, other: &Self) -> bool {
        self.bbox.overlap(&other.bbox)
    }
}

impl PartialEq for Candidate {
    fn eq(&self, other: &Self) -> bool {
        self.bbox == other.bbox
    }
}

impl Eq for Candidate {}

fn cat(x: f64) -> f64 {
    (x / 2f64).ceil()
    //x
}

use std::cmp::Ordering;
impl PartialOrd for Candidate {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        let t1 = cat(self.dtarget);
        let t2 = cat(other.dtarget);
        if t1 != t2 {
            return t1.partial_cmp(&t2);
        }
        let t1 = -self.dothers;
        let t2 = -other.dothers;
        assert!(t1.partial_cmp(&t2).is_some());
        t1.partial_cmp(&t2)
    }
}

impl Ord for Candidate {
    fn cmp(&self, other: &Self) -> Ordering {
        self.partial_cmp(&other).unwrap_or(Ordering::Equal)
    }
}

pub type Candidates = Vec<Candidate>;

pub fn select_candidates(candidates: &Candidates) -> Vec<usize> {
    if candidates.is_empty() {
        return Vec::<usize>::new();
    }
    // sort indices by candidate order.
    let mut sorted: Vec<_> = (0..candidates.len()).collect();
    sorted.sort_by(|i, j| {
        let ci = &candidates[*i];
        let cj = &candidates[*j];
        ci.partial_cmp(cj).unwrap_or(Ordering::Equal)
    });
    if sorted.len() <= 4 {
        return sorted;
    }
    // note: the candidates must be sorted
    for i in 1..candidates.len() {
        debug_assert!(candidates[i - 1] <= candidates[i]);
    }
    // we always take the first one, which is has the minimal cost.
    let mut ret = vec![0];
    let mut previous = &candidates[0];
    let overlap = 0.75f64;
    let nmax = 16;
    // we want to ensure enought diversity as with the
    // traditional four non-overlaping candidates.
    assert!(nmax as f64 * (1f64 - overlap) >= 4f64);
    for k in sorted {
        if candidates[k].bbox.overlap_ratio(&previous.bbox) < overlap {
            ret.push(k);
            previous = &candidates[k];
        }
        if ret.len() > nmax {
            break;
        }
    }
    ret
}
