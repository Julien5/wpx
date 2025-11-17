#![allow(non_snake_case)]

use std::collections::BTreeMap;

use crate::backend::Segment;
use crate::bbox::BoundingBox;
use crate::label_placement::bbox::LabelBoundingBox;
use crate::label_placement::candidate::Candidates;
use crate::label_placement::drawings::draw_for_map;
use crate::label_placement::{self, *};
use crate::math::{distance2, IntegerSize2D, Point2D};
use crate::mercator::{EuclideanBoundingBox, MercatorPoint};
use crate::track::Track;

use svg::Document;

pub fn to_graphics_coordinates(
    bbox: &BoundingBox,
    p: &MercatorPoint,
    W: i32,
    H: i32,
    margin: i32,
) -> Point2D {
    let min = bbox.get_min();
    let max = bbox.get_max();

    let f = |x: f64| -> f64 {
        let a = (W - 2 * margin) as f64 / (max.x - min.x);
        let b = -a * min.x;
        margin as f64 + a * x + b
    };
    let g = |y: f64| -> f64 {
        let a = (H - 2 * margin) as f64 / (min.y - max.y);
        let b = -a * max.y;
        margin as f64 + a * y + b
    };
    Point2D::new(f(p.x()), g(p.y()))
}

fn _readid(id: &str) -> (&str, &str) {
    id.split_once("/").unwrap()
}

use crate::label_placement::set_attr;
use crate::label_placement::Attributes;
use crate::label_placement::PointFeature;

struct MapGenerator {}

impl MapGenerator {
    fn generate_one(point: &PointFeature) -> Vec<LabelBoundingBox> {
        let mut ret =
            label_placement::cardinal_boxes(&point.center(), &point.width(), &point.height());
        let width = point.width();
        let height = point.height();
        let center = point.center();
        ret.extend_from_slice(&label_placement::far_boxes(&center, &width, &height, 0));
        ret.extend_from_slice(&label_placement::far_boxes(&center, &width, &height, 2));
        ret.extend_from_slice(&label_placement::far_boxes(&center, &width, &height, 4));
        ret.sort_by_key(|candidate| {
            let p = candidate.bbox.project_on_border(&point.center());
            (distance2(&point.center(), &p) * 100f64).floor() as i64
        });
        ret
    }
}

impl CandidatesGenerator for MapGenerator {
    fn generate(
        &self,
        features: &Vec<PointFeature>,
        obstacles: &Obstacles,
    ) -> BTreeMap<usize, Candidates> {
        label_placement::candidate::utils::generate(Self::generate_one, features, obstacles)
    }
}

struct MapData {
    polyline: Polyline,
    points: Vec<PointFeature>,
    document: Attributes,
    debug: svg::node::element::Group,
}

pub fn euclidean_bounding_box(
    track: &Track,
    range: &std::ops::Range<usize>,
    size: &(i32, i32),
) -> EuclideanBoundingBox {
    assert!(!range.is_empty());
    let mut bbox = BoundingBox::new();
    for k in range.start..range.end {
        bbox.update(&track.euclidian[k].point2d());
    }
    bbox.fix_aspect_ratio(size.0, size.1);
    bbox
}

impl MapData {
    pub fn make(segment: &Segment, size: &IntegerSize2D) -> MapData {
        let bbox = segment.map_box().clone();
        let mut path = Vec::new();
        for k in segment.range.start..segment.range.end {
            path.push(segment.track.euclidian[k].clone());
        }

        let margin = 20i32;

        let mut polyline = Polyline::new();
        // todo: path in the bbox, which more than the path in the range.
        for p in &path {
            let p = to_graphics_coordinates(&bbox, p, size.width, size.height, margin);
            polyline.points.push(p);
        }

        let mut document = Attributes::new();
        set_attr(
            &mut document,
            "viewBox",
            format!("(0, 0, {}, {})", size.width, size.height).as_str(),
        );
        set_attr(&mut document, "width", format!("{}", size.width).as_str());
        set_attr(&mut document, "height", format!("{}", size.height).as_str());

        let generator = Box::new(MapGenerator {});
        let packets = label_placement::prioritize::map(segment);
        let mut feature_packets = Vec::new();
        for packet in packets {
            let mut feature_packet = Vec::new();
            for k in packet {
                let w = &segment.points[k];
                let euclidean = w.euclidian.clone();

                let p = to_graphics_coordinates(&bbox, &euclidean, size.width, size.height, margin);
                let id = format!("wp-{}/circle", k);
                let circle = draw_for_map(&p, id.as_str(), &w.kind());
                let mut label = Label::new();
                match w.short_name() {
                    Some(text) => {
                        label.set_text(text.clone().trim());
                    }
                    None => {
                        log::error!("should not render a point without name");
                    }
                }
                label.id = format!("wp-{}/text", k);
                feature_packet.push(PointFeature {
                    circle,
                    label,
                    input_point: Some(w.clone()),
                    link: None,
                    id: k,
                });
            }
            feature_packets.push(feature_packet);
        }

        log::trace!("map: place labels");
        let result = crate::label_placement::place_labels(
            &feature_packets,
            &*generator,
            &BoundingBox::init(
                Point2D::new(0f64, 0f64),
                Point2D::new(size.width as f64, size.height as f64),
            ),
            &polyline,
            &segment.parameters.map_options.max_area_ratio,
        );
        log::trace!("map: apply placement");
        let features = result.apply(&mut feature_packets);
        MapData {
            polyline,
            points: features,
            document,
            debug: result.debug,
        }
    }

    pub fn render(self) -> String {
        let mut document = Document::new();
        for (k, v) in self.document {
            document = document.set(k, v);
        }

        let mut svgpath = svg::node::element::Path::new();
        for (k, v) in self.polyline.to_attributes() {
            svgpath = svgpath.set(k, v);
        }
        document = document.add(svgpath);

        let mut points_group = svg::node::element::Group::new();
        for point in self.points {
            point.render_in_group(&mut points_group);
            /*let mut debug_bb = svg::node::element::Rectangle::new();
            let bb = point.label.bounding_box();
            debug_bb = debug_bb.set("x", bb.x_min());
            debug_bb = debug_bb.set("y", bb.y_min());
            debug_bb = debug_bb.set("width", bb.width());
            debug_bb = debug_bb.set("height", bb.height());
            debug_bb = debug_bb.set("fill", "transparent");
            debug_bb = debug_bb.set("stroke-width", "1");
            debug_bb = debug_bb.set("stroke", "blue");
            points_group = points_group.append(debug_bb);
            */
        }
        document = document.add(points_group);
        document = document.add(self.debug);
        document.to_string()
    }
}

pub fn map(segment: &Segment, size: &IntegerSize2D) -> String {
    let svgMap = MapData::make(segment, size);
    svgMap.render()
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use super::MapGenerator;
    use crate::{
        label_placement::{Label, LabelBoundingBox, PointFeature, PointFeatureDrawing},
        math::Point2D,
    };

    #[test]
    fn test_bbox() {
        let id = String::new();
        let target = PointFeature {
            circle: PointFeatureDrawing {
                group: svg::node::element::Group::new(),
                center: Point2D::new(0f64, 0f64),
            },
            label: Label {
                id: id.clone(),
                bbox: LabelBoundingBox::_new_tlbr(
                    Point2D::new(0f64, 0f64),
                    Point2D::new(10f64, 16f64),
                ),
                text: String::from_str("hi").unwrap(),
            },
            input_point: None,
            link: None,
            id: 0,
        };
        let candidates = MapGenerator::generate_one(&target);
        let mut found = false;
        assert!(!candidates.is_empty());
        for c in candidates {
            let _center = target.center();
            let good = c.x_min() > target.center().x && c.y_min() > target.center().y;
            if good {
                found = true;
            }
        }
        assert!(found);
    }
}
