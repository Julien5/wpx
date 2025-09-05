pub fn distance((x1, y1): (f64, f64), (x2, y2): (f64, f64)) -> f64 {
    let dx = x2 - x1;
    let dy = y2 - y1;
    (dx * dx + dy * dy).sqrt()
}

#[derive(Clone)]
pub struct LabelBoundingBox {
    pub top_left: (f64, f64),
    pub bottom_right: (f64, f64),
}

impl LabelBoundingBox {
    pub fn zero() -> Self {
        LabelBoundingBox {
            top_left: (0f64, 0f64),
            bottom_right: (0f64, 0f64),
        }
    }

    pub fn new_tlbr(top_left: (f64, f64), bottom_right: (f64, f64)) -> Self {
        LabelBoundingBox {
            top_left,
            bottom_right,
        }
    }

    pub fn new_tlwh(top_left: (f64, f64), width: f64, height: f64) -> Self {
        let bottom_right = (top_left.0 + width, top_left.1 + height);
        LabelBoundingBox {
            top_left,
            bottom_right,
        }
    }

    pub fn new_blwh(bottom_left: (f64, f64), width: f64, height: f64) -> Self {
        let top_left = (bottom_left.0, bottom_left.1 - height);
        let bottom_right = (bottom_left.0 + width, bottom_left.1);
        LabelBoundingBox {
            top_left,
            bottom_right,
        }
    }

    pub fn _new_brwh(bottom_right: (f64, f64), width: f64, height: f64) -> Self {
        let top_left = (bottom_right.0 - width, bottom_right.1 - height);
        LabelBoundingBox {
            top_left,
            bottom_right,
        }
    }

    pub fn _new_trwh(top_right: (f64, f64), width: f64, height: f64) -> Self {
        let top_left = (top_right.0 - width, top_right.1);
        let bottom_right = (top_right.0, top_right.1 + height);
        LabelBoundingBox {
            top_left,
            bottom_right,
        }
    }

    pub fn x_min(&self) -> f64 {
        self.top_left.0
    }

    pub fn y_min(&self) -> f64 {
        self.top_left.1
    }

    fn bottom_left(&self) -> (f64, f64) {
        (self.x_min(), self.y_max())
    }

    fn top_right(&self) -> (f64, f64) {
        (self.x_max(), self.y_min())
    }

    pub fn _center(&self) -> (f64, f64) {
        (
            0.5 * (self.x_min() + self.x_max()),
            0.5 * (self.y_min() + self.y_max()),
        )
    }

    pub fn x_max(&self) -> f64 {
        self.bottom_right.0
    }

    pub fn y_max(&self) -> f64 {
        self.bottom_right.1
    }

    pub fn width(&self) -> f64 {
        self.x_max() - self.x_min()
    }

    pub fn height(&self) -> f64 {
        self.y_max() - self.y_min()
    }
    pub fn project_on_border(&self, q: (f64, f64)) -> (f64, f64) {
        let (qx, qy) = q;

        // Calculate distances to each edge
        let left = self.x_min();
        let right = self.x_max();
        let top = self.y_min();
        let bottom = self.y_max();

        let dist_left = (qx - left).abs();
        let dist_right = (qx - right).abs();
        let dist_top = (qy - top).abs();
        let dist_bottom = (qy - bottom).abs();

        // Find the closest edge
        let min_dist = dist_left.min(dist_right).min(dist_top).min(dist_bottom);

        if min_dist == dist_left {
            (left, qy.clamp(top, bottom)) // Project onto the left edge
        } else if min_dist == dist_right {
            (right, qy.clamp(top, bottom)) // Project onto the right edge
        } else if min_dist == dist_top {
            (qx.clamp(left, right), top) // Project onto the top edge
        } else {
            (qx.clamp(left, right), bottom) // Project onto the bottom edge
        }
    }
    pub fn distance(&self, q: (f64, f64)) -> f64 {
        let p = self.project_on_border(q);
        distance(p, q)
    }
    pub fn contains(&self, (x, y): (f64, f64)) -> bool {
        if x >= self.x_min() && x <= self.x_max() && y >= self.y_min() && y <= self.y_max() {
            return true;
        }
        false
    }
    fn overlap_self(&self, other: &Self) -> bool {
        for p in [
            self.top_left,
            self.bottom_right,
            self.bottom_left(),
            self.top_right(),
        ] {
            if other.contains(p) {
                return true;
            }
        }
        false
    }
    pub fn overlap(&self, other: &Self) -> bool {
        if other.overlap_self(self) || self.overlap_self(other) {
            return true;
        }
        false
    }
    fn area2(&self) -> f64 {
        let dx = self.x_max() - self.x_min();
        let dy = self.y_max() - self.y_min();
        return dx * dy;
    }
    fn intersection(&self, other: &Self) -> Option<LabelBoundingBox> {
        let x_min = self.x_min().max(other.x_min());
        let y_min = self.y_min().max(other.y_min());
        let x_max = self.x_max().min(other.x_max());
        let y_max = self.y_max().min(other.y_max());

        // Check if the intersection is valid (non-negative width and height)
        if x_min < x_max && y_min < y_max {
            Some(LabelBoundingBox::new_tlbr((x_min, y_min), (x_max, y_max)))
        } else {
            None // No intersection
        }
    }
    pub fn overlap_ratio(&self, other: &Self) -> f64 {
        match self.intersection(other) {
            Some(bb) => bb.area2() / self.area2(),
            None => 0f64,
        }
    }
}

impl PartialEq for LabelBoundingBox {
    fn eq(&self, other: &Self) -> bool {
        self.top_left == other.top_left && self.bottom_right == other.bottom_right
    }
}

use std::fmt;
impl fmt::Display for LabelBoundingBox {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "LabelBoundingBox {{ top_left: ({:.2}, {:.2}), (w,h): ({:.2}, {:.2}) }}",
            self.top_left.0,
            self.top_left.1,
            self.width(),
            self.height()
        )
    }
}

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
