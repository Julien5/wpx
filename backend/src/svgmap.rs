#![allow(non_snake_case)]

use std::collections::BTreeSet;

use crate::bbox::BoundingBox;
use crate::inputpoint::InputPoint;
use crate::label_placement::candidate::Candidate;
use crate::label_placement::drawings::draw_for_map;
use crate::label_placement::obstacle::Obstacles;
use crate::label_placement::{self, *};
use crate::math::Point2D;
use crate::mercator::{EuclideanBoundingBox, MercatorPoint};
use crate::point_collection::{Packets, RenderInputParameters, RenderResult};
use crate::track::Track;

#[allow(unused_imports)]
use crate::math::distance2;
#[allow(unused_imports)]
use crate::point_collection::Kind;
use crate::track_projection::is_close_to_track;

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
use crate::label_placement::features::{Features, PointFeature};

struct MapGenerator {}

impl CandidatesGenerator for MapGenerator {
    fn gen(
        &self,
        feature: &PointFeature,
        obstacles: &Obstacles,
        _hardness: usize,
    ) -> Vec<Candidate> {
        let mut cardinal_boxes =
            label_placement::cardinal_boxes(&feature.center(), feature.width(), feature.height());
        cardinal_boxes.retain(|bbox| !obstacles.hit(feature, &bbox.absolute()));
        let search_width = 200f64;
        let search_area = BoundingBox::minsize(
            feature.center() - Point2D::new(search_width * 0.5f64, search_width * 0.5f64),
            search_width,
            search_width,
        );
        let cardinal_candidates: Vec<_> = cardinal_boxes
            .iter()
            .map(|lbbox| Candidate::new(lbbox))
            .collect();
        // if the area is not empty, do not try hard placement
        if obstacles.occupied_area(&search_area) / search_area.area() > 0.0f64 {
            return cardinal_candidates;
        }
        match feature.input_point() {
            Some(point) => {
                if !is_close_to_track(&point) {
                    return cardinal_candidates;
                }
            }
            None => {}
        }
        let aux_boxes = label_placement::far_cardinal_boxes(
            &feature.center(),
            feature.width(),
            feature.height(),
            25f64,
        );
        let aux_candidates: Vec<_> = aux_boxes
            .iter()
            .map(|lbbox| Candidate::new(lbbox))
            .collect();
        let mut ret = Vec::new();
        ret.extend_from_slice(&cardinal_candidates);
        ret.extend_from_slice(&aux_candidates);
        ret.retain(|c| !obstacles.hit(feature, &c.bbox().absolute()));
        ret
    }
}

struct MapView {
    polyline: Polyline,
    points: Vec<PointFeature>,
    attributes: Attributes,
}

impl MapView {
    pub fn render(&self) -> String {
        let mut document = Document::new();
        for (k, v) in &self.attributes {
            document = document.set(k, v.clone());
        }

        let mut svgpath = svg::node::element::Path::new();
        for (k, v) in self.polyline.to_attributes() {
            svgpath = svgpath.set(k, v);
        }
        document = document.add(svgpath);

        let mut points_group = svg::node::element::Group::new();
        for point in &self.points {
            point.render_in_group(&mut points_group);
        }
        document = document.add(points_group);
        document.to_string()
    }
}

struct MapMaker {
    parameters: RenderInputParameters,
    map_box: EuclideanBoundingBox,
}

pub fn euclidean_bounding_box(
    track: &Track,
    range: &std::ops::Range<usize>,
) -> EuclideanBoundingBox {
    assert!(!range.is_empty());
    let mut bbox = BoundingBox::new();
    for idx in &track.simplified.xy {
        if range.contains(idx) {
            bbox.update(&track.euclidean[*idx].point2d());
        }
    }
    bbox.enlarge(100f64);
    bbox
}

impl MapMaker {
    pub fn init(track: &Track, parameters: &RenderInputParameters) -> Self {
        let mut bbox = euclidean_bounding_box(track, &parameters.range);
        bbox.fix_aspect_ratio(&parameters.screen_size);
        Self {
            parameters: parameters.clone(),
            map_box: bbox,
        }
    }
}

impl MapMaker {
    fn margin() -> i32 {
        20
    }

    fn make_one_feature(&self, w: &InputPoint, track: &Track, counter: usize) -> PointFeature {
        let euclidean = &w.euclidean;
        let bbox = &self.map_box;
        let size = &self.parameters.screen_size;
        let on_track = track.project_simplified(&euclidean).euclidean;
        let margin = Self::margin();
        let mut p = to_graphics_coordinates(bbox, &euclidean, size.width, size.height, margin);
        let p_track = to_graphics_coordinates(bbox, &on_track, size.width, size.height, margin);
        if p.distance_to(&p_track) < 5f64 {
            p = p_track;
        }
        let k = counter;
        let id = format!("{}/wp/circle", k);
        let circle = draw_for_map(&p, id.as_str(), &w);

        // on the map, all projections are equivalent
        let mut label = drawings::make_label_text(&w);
        label.id = format!("{}/wp/text", k);
        let track_indices: BTreeSet<_> = w
            .track_projections
            .iter()
            .map(|proj| proj.track_index)
            .collect();
        PointFeature {
            circle,
            label,
            input_point: Some((w.clone(), track_indices)),
            link: None,
            xmlid: k,
        }
    }

    fn make_polyline(&self, track: &Track) -> Polyline {
        let mut path = Vec::new();
        let range = &self.parameters.range;
        for idx in &track.simplified.xy {
            if *idx >= range.start && *idx < range.end {
                path.push(track.euclidean[*idx].clone());
            }
        }

        let mut polyline_points = PolylinePoints::new();
        // todo: path in the bbox, which more than the path in the range.
        for p in &path {
            let p = to_graphics_coordinates(
                &self.map_box,
                p,
                self.parameters.screen_size.width,
                self.parameters.screen_size.height,
                Self::margin(),
            );
            polyline_points.push(PolylinePoint(p));
        }
        Polyline::new(polyline_points)
    }

    fn make_features(
        &self,
        track: &Track,
        packets: &Packets,
        debug_graphic_dir: Option<String>,
    ) -> Features {
        let polyline = self.make_polyline(track);
        let generator = Box::new(MapGenerator {});
        let mut feature_packets = Vec::new();
        let mut feature_unlabeled = Vec::new();
        let mut counter = 0;
        let size = &self.parameters.screen_size;
        for packet in packets {
            let mut feature_packet = Vec::new();
            for w in packet {
                let euclidean = w.euclidean.clone();
                if !self.map_box.contains(&euclidean.point2d()) {
                    continue;
                }
                let feature = self.make_one_feature(w, track, counter);
                let empty = feature.label.is_empty();
                counter += 1;
                if empty {
                    feature_unlabeled.push(feature);
                } else {
                    feature_packet.push(feature);
                }
            }
            feature_packets.push(PointFeatures::make(feature_packet));
        }

        let (results, obstacles) = crate::label_placement::place_labels(
            &feature_packets,
            &*generator,
            &BoundingBox::minmax(
                Point2D::new(0f64, 0f64),
                Point2D::new(size.width as f64, size.height as f64),
            ),
            &polyline,
            &self.parameters.parameters.map_options.max_area_ratio,
            debug_graphic_dir.clone(),
        );
        let labeled = PlacementResult::apply(&results, &obstacles, &mut feature_packets);
        Features {
            labeled,
            unlabeled: feature_unlabeled,
            polylines: vec![polyline],
        }
    }

    fn make_view_features(
        &self,
        track: &Track,
        features: &Vec<PointFeature>,
        usersteps: &Vec<InputPoint>,
        _debug_graphic_dir: Option<String>,
    ) -> MapView {
        let mut document = Attributes::new();
        let size = &self.parameters.screen_size;
        set_attr(
            &mut document,
            "viewBox",
            format!("(0, 0, {}, {})", size.width, size.height).as_str(),
        );
        let mut points = features.clone();
        for w in usersteps {
            let euclidean = w.euclidean.clone();
            if !self.map_box.contains(&euclidean.point2d()) {
                continue;
            }
            let feature = self.make_one_feature(w, track, points.len());
            points.push(feature);
        }
        set_attr(&mut document, "width", format!("{}", size.width).as_str());
        set_attr(&mut document, "height", format!("{}", size.height).as_str());
        let polyline = self.make_polyline(track);
        MapView {
            points: points,
            polyline: polyline,
            attributes: document,
        }
    }

    fn make_model_from_packets(
        &self,
        track: &Track,
        packets: &Packets,
        debug_graphic_dir: Option<String>,
    ) -> MapView {
        let mut features = self.make_features(track, packets, debug_graphic_dir);
        let mut document = Attributes::new();
        let size = &self.parameters.screen_size;
        set_attr(
            &mut document,
            "viewBox",
            format!("(0, 0, {}, {})", size.width, size.height).as_str(),
        );
        set_attr(&mut document, "width", format!("{}", size.width).as_str());
        set_attr(&mut document, "height", format!("{}", size.height).as_str());
        MapView {
            points: features.points(),
            polyline: features.polylines.remove(0),
            attributes: document,
        }
    }
}

pub fn map_background(
    track: &Track,
    parameters: &RenderInputParameters,
    packets: &Packets,
    debug_dir: Option<String>,
) -> RenderResult {
    log::info!("compute map background for parameters {:?}", parameters);
    let maker = MapMaker::init(track, parameters);
    let view = maker.make_model_from_packets(track, packets, debug_dir);
    let svg = view.render();
    RenderResult {
        svg,
        rendered: view.points.clone(),
        parameters: parameters.clone(),
    }
}

pub fn map_foreground(
    track: &Track,
    parameters: &RenderInputParameters,
    background: &RenderResult,
    debug_dir: Option<String>,
) -> RenderResult {
    log::info!("compute map foreground for parameters {:?}", parameters);
    let maker = MapMaker::init(track, parameters);
    let view = maker.make_view_features(
        track,
        &background.rendered,
        &parameters.usersteps,
        debug_dir,
    );
    RenderResult {
        svg: view.render(),
        rendered: background.rendered.clone(),
        parameters: parameters.clone(),
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use crate::{
        bbox::BoundingBox,
        label_placement::{
            features::*, labelboundingbox::LabelBoundingBox, obstacle::Obstacles,
            CandidatesGenerator,
        },
        math::Point2D,
        svgmap::MapGenerator,
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
                fontsize: FONTSIZE,
                fontweight: "normal".to_string(),
                fontstyle: "normal".to_string(),
            },
            input_point: None,
            link: None,
            xmlid: 0,
        };
        let area = BoundingBox::new();
        let obstacles = Obstacles::new(&area, 0f64);
        let candidates = MapGenerator {}.gen(&target, &obstacles, 0);
        let mut found = false;
        assert!(!candidates.is_empty());
        for c in candidates {
            let _center = target.center();
            let good = c.bbox().absolute().get_xmin() > target.center().x
                && c.bbox().absolute().get_ymin() > target.center().y;
            if good {
                found = true;
            }
        }
        assert!(found);
    }
}
