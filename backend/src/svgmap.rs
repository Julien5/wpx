#![allow(non_snake_case)]

use crate::backend::Segment;
use crate::bbox::BoundingBox;
use crate::label_placement::drawings::draw_for_map;
use crate::label_placement::labelboundingbox::LabelBoundingBox;
use crate::label_placement::{self, *};
use crate::math::{IntegerSize2D, Point2D};
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

use crate::label_placement::features::{set_attr, PointFeatures, PolylinePoint, PolylinePoints};
use crate::label_placement::features::{Attributes, Polyline};
use crate::label_placement::features::{Label, PointFeature};

struct MapGenerator {}

impl CandidatesGenerator for MapGenerator {
    fn gen(&self, feature: &PointFeature) -> Vec<LabelBoundingBox> {
        let mut ret =
            label_placement::cardinal_boxes(&feature.center(), &feature.width(), &feature.height());
        let width = feature.width();
        let height = feature.height();
        let center = feature.center();
        ret.extend_from_slice(&label_placement::far_boxes(&center, &width, &height, 0));
        ret.extend_from_slice(&label_placement::far_boxes(&center, &width, &height, 2));
        ret.extend_from_slice(&label_placement::far_boxes(&center, &width, &height, 4));
        /*ret.sort_by_key(|candidate| {
            let p = candidate.absolute().project_on_border(&point.center());
            (distance2(&point.center(), &p) * 100f64).floor() as i64
        });*/
        ret
    }
}

struct MapData {
    polyline: Polyline,
    points: Vec<PointFeature>,
    document: Attributes,
}

pub fn euclidean_bounding_box(
    track: &Track,
    range: &std::ops::Range<usize>,
    _size: &IntegerSize2D,
) -> EuclideanBoundingBox {
    assert!(!range.is_empty());
    let mut bbox = BoundingBox::new();
    for k in range.start..range.end {
        bbox.update(&track.euclidean[k].point2d());
    }
    bbox
}

impl MapData {
    pub fn make(segment: &Segment, size: &IntegerSize2D) -> MapData {
        let mut bbox = segment.map_box().clone();
        bbox.fix_aspect_ratio(size);
        let mut path = Vec::new();
        let range = segment.range();
        for k in range.start..range.end {
            path.push(segment.track.euclidean[k].clone());
        }

        let margin = 20i32;

        let mut polyline_points = PolylinePoints::new();
        // todo: path in the bbox, which more than the path in the range.
        for p in &path {
            let p = to_graphics_coordinates(&bbox, p, size.width, size.height, margin);
            polyline_points.push(PolylinePoint(p));
        }
        let polyline = Polyline::new(polyline_points);

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
        let mut counter = 0;
        for packet in packets {
            let mut feature_packet = Vec::new();
            for w in packet {
                let euclidean = w.euclidean.clone();
                if !bbox.contains(&euclidean.point2d()) {
                    continue;
                }
                let p = to_graphics_coordinates(&bbox, &euclidean, size.width, size.height, margin);
                let k = counter;
                counter += 1;
                let id = format!("{}/wp/circle", k);
                let circle = draw_for_map(&p, id.as_str(), &w);
                let mut label = Label::new();
                label.set_text(&drawings::make_label_text(&w, segment));
                label.id = format!("{}/wp/text", k);
                feature_packet.push(PointFeature {
                    circle,
                    label,
                    input_point: Some(w.clone()),
                    link: None,
                    xmlid: k,
                });
            }
            feature_packets.push(PointFeatures::make(feature_packet));
        }

        log::trace!("map: place labels");
        let (results, obstacles) = crate::label_placement::place_labels(
            &feature_packets,
            &*generator,
            &BoundingBox::minmax(
                Point2D::new(0f64, 0f64),
                Point2D::new(size.width as f64, size.height as f64),
            ),
            &polyline,
            &segment.parameters.map_options.max_area_ratio,
        );
        log::trace!("map: apply placement");
        let features = PlacementResult::apply(&results, &obstacles, &mut feature_packets);
        MapData {
            polyline,
            points: features,
            document,
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
        bbox::BoundingBox,
        label_placement::{features::*, labelboundingbox::LabelBoundingBox, CandidatesGenerator},
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
                bbox: LabelBoundingBox::new_absolute(
                    &BoundingBox::minmax(Point2D::new(0f64, 0f64), Point2D::new(10f64, 16f64)),
                    &Point2D::zero(),
                ),
                text: String::from_str("hi").unwrap(),
                _placed: false,
            },
            input_point: None,
            link: None,
            xmlid: 0,
        };
        let candidates = MapGenerator {}.gen(&target);
        let mut found = false;
        assert!(!candidates.is_empty());
        for c in candidates {
            let _center = target.center();
            let good = c.absolute().get_xmin() > target.center().x
                && c.absolute().get_ymin() > target.center().y;
            if good {
                found = true;
            }
        }
        assert!(found);
    }
}
