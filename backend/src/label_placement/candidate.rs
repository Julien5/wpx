use super::labelboundingbox::LabelBoundingBox;

#[derive(Clone)]
pub struct Candidate {
    _bbox: LabelBoundingBox,
    external: bool,
}

impl Candidate {
    pub fn new(bbox: &LabelBoundingBox) -> Candidate {
        Candidate {
            _bbox: bbox.clone(),
            external: false,
        }
    }

    pub fn make_external(bbox: &LabelBoundingBox) -> Candidate {
        Candidate {
            _bbox: bbox.clone(),
            external: true,
        }
    }

    pub fn is_external(&self) -> bool {
        self.external
    }

    pub fn hit_other(&self, other: &Self) -> bool {
        self._bbox.absolute().overlap(&other._bbox.absolute())
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

pub type Candidates = Vec<Candidate>;

pub mod utils {
    use crate::label_placement::{features::PointFeature, obstacle::Obstacles, *};

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

    fn generate_all_candidates(
        gen: &dyn CandidatesGenerator,
        target: &PointFeature,
        obstacles: &Obstacles,
    ) -> Candidates {
        if target.text().is_empty() {
            return Candidates::new();
        }
        let target = &target;
        let available_area = obstacles.available_area();
        if target.area() > available_area {
            return Candidates::new();
        }
        let candidates = gen.gen(target, obstacles);
        if candidates.is_empty() {
            /*log::info!(
                "no candidates passed the upfront obstacles test for: [{}]",
                target.id()
            );*/
        }
        return candidates;
    }

    pub fn generate(
        gen: &dyn CandidatesGenerator,
        features: &PointFeatures,
        obstacles: &Obstacles,
    ) -> Vec<Candidates> {
        let mut ret = Vec::new();
        for k in 0..features.points.len() {
            let feature = &features.points[k];
            let candidates = generate_all_candidates(gen, feature, obstacles);
            ret.push(candidates);
        }
        ret
    }
}
