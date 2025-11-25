use crate::{bbox::BoundingBox, math::Point2D};

use super::labelboundingbox::LabelBoundingBox;

#[derive(Clone)]
pub struct Candidate {
    _bbox: LabelBoundingBox,
    _dtarget: f64,
    _dothers: f64,
}

impl Candidate {
    pub fn new(bbox: &LabelBoundingBox, dtarget: &f64, dothers: &f64) -> Candidate {
        Candidate {
            _bbox: bbox.clone(),
            _dtarget: *dtarget,
            _dothers: *dothers,
        }
    }

    pub fn hit_other(&self, other: &Self) -> bool {
        self._bbox.absolute().overlap(&other._bbox.absolute())
    }

    pub fn hit_bbox(&self, bbox: &BoundingBox) -> bool {
        self._bbox.absolute().overlap(&bbox)
    }

    pub fn hit_polyline(&self, polyline: &Vec<Point2D>) -> bool {
        for p in polyline {
            if self._bbox.absolute().contains(&p) {
                return true;
            }
        }
        false
    }
    pub fn bbox(&self) -> &LabelBoundingBox {
        &self._bbox
    }
}

impl PartialEq for Candidate {
    fn eq(&self, other: &Self) -> bool {
        self._bbox == other._bbox
    }
}

impl Eq for Candidate {}

fn cat(x: f64) -> f64 {
    (x / 2f64).ceil()
    //x
}

use std::cmp::Ordering;
impl PartialOrd for Candidate {
    // ordering taking the distance to target and the distance to other features.
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        let dtarget1 = cat(self._dtarget);
        let dtarget2 = cat(other._dtarget);
        if dtarget1 != dtarget2 {
            return dtarget1.partial_cmp(&dtarget2);
        }
        let t1 = -self._dothers;
        let t2 = -other._dothers;
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

pub mod utils {
    use crate::label_placement::{
        features::{Obstacles, PointFeature},
        labelboundingbox::LabelBoundingBox,
        *,
    };

    pub fn candidates_bounding_box(candidates: &Candidates) -> BoundingBox {
        let mut ret = BoundingBox::new();
        let _: Vec<_> = candidates
            .iter()
            .map(|candidate| {
                let b = candidate.bbox().absolute();
                ret.update(&b.get_min());
                ret.update(&b.get_max());
            })
            .collect();
        ret
    }

    pub fn make_candidate(
        bbox: &LabelBoundingBox,
        target: &PointFeature,
        features: &PointFeatures,
        obstacles: &Obstacles,
    ) -> Candidate {
        let dtarget = bbox.absolute().distance2_to_point(&target.center());
        let neighbors = features.nearest_neighbors(&bbox.absolute().center(), 2);
        let mut dothers = f64::MAX;
        for (neighbor, dist2) in neighbors {
            if neighbor.id == target.id {
                continue;
            } else {
                dothers = dist2;
                break;
            }
        }
        Candidate::new(bbox, &dtarget, &dothers)
    }

    fn hit(candidate: &Candidate, obstacles: &Obstacles) -> bool {
        if !obstacles.drawingbox.bbox.empty()
            && !obstacles
                .drawingbox
                .bbox
                .contains_other(&candidate.bbox().absolute())
        {
            return true;
        }
        for obstacle_box in &obstacles.bboxes {
            if candidate.hit_bbox(obstacle_box) {
                return true;
            }
        }
        for polyline in &obstacles.polylines {
            if candidate.hit_polyline(&polyline.points) {
                return true;
            }
        }
        false
    }

    fn generate_all_candidates(
        gen: fn(&PointFeature) -> Vec<LabelBoundingBox>,
        target: &PointFeature,
        all: &PointFeatures,
        obstacles: &Obstacles,
    ) -> Candidates {
        if target.text().is_empty() {
            return Candidates::new();
        }
        let target = &target;
        let mut ret = Candidates::new();
        let available_area = obstacles.available_area();
        if target.area() > available_area {
            //log::trace!("no place left for {}", target.text());
            return ret;
        }
        for bbox in gen(target) {
            let candidate = make_candidate(&bbox, &target, &all, obstacles);
            if hit(&candidate, obstacles) {
                continue;
            }
            ret.push(candidate);
        }
        return ret;
    }

    pub fn generate(
        gen_one: fn(&PointFeature) -> Vec<LabelBoundingBox>,
        features: &PointFeatures,
        obstacles: &Obstacles,
    ) -> BTreeMap<features::PointFeatureId, Candidates> {
        let mut ret = BTreeMap::new();
        for feature in &features.points {
            let candidates = generate_all_candidates(gen_one, feature, features, obstacles);
            if candidates.is_empty() {
                /*log::trace!(
                    "[0] [{}] => {} candidates",
                    feature.text(),
                    candidates.len()
                );*/
                // force one ?
            }
            ret.insert(feature.id, candidates);
        }
        ret
    }
}
