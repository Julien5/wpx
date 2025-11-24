use super::BoundingBox;
use euclid::Point2D;

// ---- Quadtree Implementation ----

const MAX_OBJECTS: usize = 8;
const MAX_DEPTH: usize = 12;

#[derive(Debug)]
pub struct QuadTree<T> {
    boundary: BoundingBox,
    objects: Vec<(BoundingBox, T)>,
    children: Option<Box<[QuadTree<T>; 4]>>,
    depth: usize,
}

impl<T: Clone> QuadTree<T> {
    pub fn new(boundary: BoundingBox) -> Self {
        Self {
            boundary,
            objects: Vec::new(),
            children: None,
            depth: 0,
        }
    }

    fn subdivide(&mut self) {
        let mid_x = (self.boundary.get_xmin() + self.boundary.get_xmax()) * 0.5;
        let mid_y = (self.boundary.get_ymin() + self.boundary.get_ymax()) * 0.5;

        let (min_x, min_y) = (self.boundary.get_xmin(), self.boundary.get_ymin());
        let (max_x, max_y) = (self.boundary.get_xmax(), self.boundary.get_ymax());

        let quads = [
            BoundingBox::minmax(Point2D::new(min_x, mid_y), Point2D::new(mid_x, max_y)),
            BoundingBox::minmax(Point2D::new(mid_x, mid_y), Point2D::new(max_x, max_y)),
            BoundingBox::minmax(Point2D::new(min_x, min_y), Point2D::new(mid_x, mid_y)),
            BoundingBox::minmax(Point2D::new(mid_x, min_y), Point2D::new(max_x, mid_y)),
        ];

        let children = quads.map(|b| QuadTree {
            boundary: b,
            objects: Vec::new(),
            children: None,
            depth: self.depth + 1,
        });

        self.children = Some(Box::new(children));
    }

    /// Insert a box + payload
    pub fn insert(&mut self, aabb: &BoundingBox, value: T) -> bool {
        // If outside boundary, reject
        if !self.boundary.overlap(aabb) {
            return false;
        }

        // Leaf node
        if self.children.is_none() {
            self.objects.push((aabb.clone(), value));

            if self.objects.len() > MAX_OBJECTS && self.depth < MAX_DEPTH {
                self.subdivide();

                // Re-insert objects into children
                let mut to_reinsert = Vec::new();
                std::mem::swap(&mut to_reinsert, &mut self.objects);

                for (b, v) in to_reinsert {
                    self.insert(&b, v);
                }
            }
            return true;
        }

        // Internal node → push downward
        if let Some(children) = self.children.as_mut() {
            for child in children.iter_mut() {
                if child.insert(aabb, value.clone()) {
                    return true;
                }
            }
        }

        // Fallback (very rare): store here
        self.objects.push((aabb.clone(), value));
        true
    }

    /// Query all objects whose bounding box intersects the given range
    pub fn query<'a>(&'a self, range: &BoundingBox, out: &mut Vec<&'a T>) {
        // No overlap → skip
        if !self.boundary.overlap(range) {
            return;
        }

        // Check objects at this node
        for (b, v) in &self.objects {
            if b.overlap(range) {
                out.push(v);
            }
        }

        // Recurse into children
        if let Some(children) = &self.children {
            for child in children.iter() {
                child.query(range, out);
            }
        }
    }
}

// ---- Example Test ----

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_quadtree() {
        let world = BoundingBox::minmax(Point2D::new(0.0, 0.0), Point2D::new(100.0, 100.0));

        let mut qt = QuadTree::new(world);

        for i in 0..100 {
            let b = BoundingBox::minmax(
                Point2D::new(i as f64, i as f64),
                Point2D::new(i as f64 + 2.0, i as f64 + 2.0),
            );
            qt.insert(&b, i);
        }

        let query_box = BoundingBox::minmax(Point2D::new(10.0, 10.0), Point2D::new(15.0, 15.0));

        let mut hits = Vec::new();
        qt.query(&query_box, &mut hits);

        assert!(hits.contains(&&10));
        assert!(hits.contains(&&11));
    }
}
