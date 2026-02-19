use crate::{
    bbox::{quadtree::QuadTree, BoundingBox},
    label_placement::features::{PointFeature, Polyline},
    math::Point2D,
};

#[derive(Clone)]
pub struct DrawingArea {
    pub bbox: BoundingBox,
    pub max_area_ratio: f64,
}

#[derive(Clone)]
pub struct Obstacles {
    bboxes: Vec<BoundingBox>,
    bboxes_tree: QuadTree<usize>,
    pub polylines: Vec<Polyline>,
    pub drawingbox: DrawingArea,
}

impl Obstacles {
    pub fn new(area: &BoundingBox, ratio: f64) -> Self {
        Self {
            drawingbox: DrawingArea {
                bbox: area.clone(),
                max_area_ratio: ratio,
            },
            polylines: Vec::new(),
            bboxes: Vec::new(),
            bboxes_tree: QuadTree::new(area.clone()),
        }
    }

    pub fn bboxes(&self) -> Vec<BoundingBox> {
        self.bboxes.clone()
    }

    pub fn push_bbox(&mut self, bbox: BoundingBox) {
        let idx = self.bboxes.len();
        self.bboxes_tree.insert(&bbox, idx);
        self.bboxes.push(bbox);
    }

    pub fn hit_bbox(&self, bbox: &BoundingBox) -> bool {
        /*
        for b in &self.bboxes {
             if b.overlap(bbox) {
                 return true;
             }
         }
         false
         */
        self.bboxes_tree.has_overlap(&bbox)
    }

    pub fn _is_clear(&self, p1: &Point2D, p2: &Point2D) -> bool {
        for bbox in &self.bboxes {
            if bbox.segment_intersects(p1, p2) {
                return false;
            }
        }
        true
    }

    pub fn available_area(&self) -> f64 {
        self.drawingbox.bbox.area() - self.bboxes.iter().map(|bbox| bbox.area()).sum::<f64>()
    }

    pub fn occupied_area(&self, search_area: &BoundingBox) -> f64 {
        let mut nearby_indices = Vec::new();
        self.bboxes_tree.query(&search_area, &mut nearby_indices);

        let mut ret = 0.0;
        for idx in nearby_indices {
            let obstacle = &self.bboxes[*idx];
            let intersection = obstacle.intersection(&search_area);
            if intersection.is_some() {
                ret += intersection.unwrap().area();
            }
        }
        ret
    }

    pub fn hit(&self, feature: &PointFeature, candidate_bbox: &BoundingBox) -> bool {
        if !self.drawingbox.bbox.empty() && !self.drawingbox.bbox.contains_other(candidate_bbox) {
            return true;
        }
        if self.hit_bbox(&candidate_bbox) {
            return true;
        }
        let target = feature.center();
        let start = candidate_bbox.project_on_border(&target);
        let aux = Point2D::point_on_segment_from_end(&start, &target, 5f64);
        let is_far_candidate = start.distance_to(&target) > 10f64;
        for polyline in &self.polylines {
            if polyline.hit(candidate_bbox) {
                return true;
            }
            if is_far_candidate && polyline.hit_segment(&start, &aux) {
                return true;
            }
        }
        false
    }
}
