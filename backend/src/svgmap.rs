#![allow(non_snake_case)]

use std::str::FromStr;

use crate::bbox::BoundingBox;
use crate::label_placement::bbox::LabelBoundingBox;
use crate::label_placement::{Circle, Label, Polyline};
use crate::utm::UTMPoint;
use crate::waypoint::{WGS84Point, WaypointOrigin};
use crate::{backend, waypoints_table};
use crate::{segment, utm};

use svg::Document;

fn to_graphics_coordinates(bbox: &BoundingBox, p: &UTMPoint, W: i32, H: i32) -> (f64, f64) {
    let xmin = bbox.min.0;
    let xmax = bbox.max.0;
    let ymin = bbox.min.1;
    let ymax = bbox.max.1;

    let f = |x: f64| -> f64 {
        let a = W as f64 / (xmax - xmin);
        let b = -a * xmin;
        a * x + b
    };
    let g = |y: f64| -> f64 {
        let a = H as f64 / (ymin - ymax);
        let b = -a * ymax;
        a * y + b
    };
    (f(p.x()), g(p.y()))
}

fn _readid(id: &str) -> (&str, &str) {
    id.split_once("/").unwrap()
}

use crate::label_placement::set_attr;
use crate::label_placement::Attributes;
use crate::label_placement::PointFeature;

fn generate_candidates_bboxes(point: &PointFeature) -> Vec<LabelBoundingBox> {
    let mut ret = Vec::new();
    let width = point.width();
    let height = point.height();
    let dtarget_min = 1f64;
    let dtarget_max = 20f64;
    let d0 = dtarget_max;
    let (cx, cy) = point.center();
    let xmin = cx - d0 - width;
    let ymin = cy - d0 - height;
    let xmax = cx + d0;
    let ymax = cy + d0;
    let dp = 5f64;
    let countx = ((xmax - xmin) / dp).ceil() as i32;
    let county = ((ymax - ymin) / dp).ceil() as i32;
    let dx = dp;
    let dy = dp;
    for nx in 0..countx {
        for ny in 0..county {
            let tl = (xmin + nx as f64 * dx, ymin + ny as f64 * dy);
            let bb = LabelBoundingBox::new_tlwh(tl, width, height);
            if bb.contains((cx, cy)) {
                continue;
            }
            if bb.distance((cx, cy)) < dtarget_min {
                continue;
            }
            ret.push(bb);
        }
    }
    ret
}

struct MapData {
    polyline: Polyline,
    points: Vec<PointFeature>,
    document: Attributes,
    debug: svg::node::element::Group,
}

impl MapData {
    pub fn make(
        backend: &backend::BackendData,
        segment: &segment::Segment,
        W: i32,
        H: i32,
        _debug: bool,
    ) -> MapData {
        let geodata = &backend.track;
        let waypoints = backend.get_waypoints();
        let mut bbox84 = BoundingBox::new();
        let range = &segment.range;
        for k in range.start..range.end {
            bbox84.update(&geodata.wgs84[k].xy());
        }
        let projection = utm::UTMProjection::make(WGS84Point::from_xy(&bbox84.middle()));

        let mut bbox = BoundingBox::new();
        let mut path = Vec::new();
        for k in range.start..range.end {
            let w = &geodata.wgs84[k];
            let utm = projection.project(w);
            path.push(utm.clone());
            bbox.update(&utm.xy());
        }
        bbox.fix_aspect_ratio(W, H);

        let mut polyline = Polyline::new();
        // todo: path in the bbox, which more than the path in the range.
        for p in &path {
            let (xg, yg) = to_graphics_coordinates(&bbox, p, W, H);
            polyline.points.push((xg, yg));
        }

        let mut document = Attributes::new();
        set_attr(
            &mut document,
            "viewBox",
            format!("(0, 0, {W}, {H})").as_str(),
        );
        set_attr(&mut document, "width", format!("{W}").as_str());
        set_attr(&mut document, "height", format!("{H}").as_str());

        let V = waypoints_table::show_waypoints_in_table(&waypoints, &segment.bbox);
        let mut points = Vec::new();
        for k in 0..waypoints.len() {
            let w = &waypoints[k];
            let utm = projection.project(&w.wgs84);
            if !bbox.contains(&utm.xy()) {
                continue;
            }
            if w.origin != WaypointOrigin::GPX {
                continue;
            }
            if !range.contains(&w.track_index.unwrap()) {
                continue;
            }
            let (x, y) = to_graphics_coordinates(&bbox, &utm, W, H);
            let mut circle = Circle::new();
            circle.id = format!("wp-{}/circle", k);
            circle.cx = x;
            circle.cy = y;
            let id = format!("wp-{}", k);
            let mut label = Label::new();
            if V.contains(&k) {
                label.set_text(w.info.as_ref().unwrap().profile_label().trim());
                label.id = format!("wp-{}/text", k);
            } else {
                circle.fill = Some(String::from_str("blue").unwrap());
            }
            points.push(PointFeature::new(id, circle, label));
        }

        for (kind, osmpoints) in &backend.osmwaypoints {
            for k in 0..osmpoints.len() {
                let w = &osmpoints[k];
                let utm = projection.project(&w.wgs84);
                if !bbox.contains(&utm.xy()) {
                    continue;
                }
                if w.name.is_none() {
                    continue;
                }
                if !range.contains(&w.track_index.unwrap()) {
                    continue;
                }
                let mut circle = Circle::new();
                let (x, y) = to_graphics_coordinates(&bbox, &utm, W, H);
                let n = points.len();
                circle.id = format!("wp-{}/circle", n);
                circle.cx = x;
                circle.cy = y;
                let id = format!("wp-{}", n);
                use super::osm::osmpoint::OSMType::*;
                match kind {
                    City => {
                        circle.r = 5f64;
                        circle.fill = Some("Gray".to_string());
                    }
                    Village => {
                        circle.r = 3f64;
                        circle.fill = Some("Gray".to_string());
                    }
                    MountainPass => {
                        circle.r = 3f64;
                        circle.fill = Some("Blue".to_string());
                    }
                }
                let mut label = Label::new();
                label.set_text(w.name.clone().unwrap().trim());
                label.id = format!("wp-{}/text", k);
                points.push(PointFeature::new(id, circle, label));
            }
        }
        let result = crate::label_placement::place_labels_gen(
            &mut points,
            generate_candidates_bboxes,
            &polyline,
        );
        let mut placed_points = Vec::new();
        for k in 0..points.len() {
            if !result.failed_indices.contains(&k) {
                placed_points.push(points[k].clone());
            }
        }
        MapData {
            polyline,
            points: placed_points,
            document,
            debug: result.debug,
        }
    }
    /*
        fn _import(filename: std::path::PathBuf) -> MapData {
            use svg::node::element::tag;
            use svg::parser::Event;
            let mut polyline = crate::svgmap::Polyline::new();
            let mut document = Attributes::new();
            let mut content = String::new();
            let mut points = Vec::new();
            let mut current_circle = PointFeature::new();
            let mut current_text_attributes = Attributes::new();
            for event in svg::open(filename, &mut content).unwrap() {
                match event {
                    Event::Tag(tag::Circle, _, attributes) => {
                        if attributes.contains_key("id") {
                            let id = attributes.get("id").unwrap().clone().to_string();
                            let (p_id, _p_attr) = _readid(id.as_str());
                            current_circle.id = String::from_str(p_id).unwrap();
                            current_circle.circle = Circle::_from_attributes(&attributes);
                        }
                    }
                    Event::Tag(tag::Text, _, attributes) => {
                        if attributes.contains_key("id") {
                            // let id = attributes.get("id").unwrap();
                            current_text_attributes = attributes.clone();
                        }
                    }
                    Event::Text(data) => {
                        current_circle.label = Label::_from_attributes(&current_text_attributes, data);
                        current_text_attributes.clear();
                        debug_assert!(!current_circle.id.is_empty());
                        points.push(current_circle);
                        current_circle = PointFeature::new();
                    }
                    Event::Tag(tag::Path, _, attributes) => {
                        if attributes.contains_key("id") {
                            let id = attributes.get("id").unwrap();
                        }
                        polyline = crate::svgmap::Polyline::_from_attributes(&attributes);
                        let data = attributes.get("d").unwrap();
                        let data = Data::parse(data).unwrap();
                        use svg::node::element::path::Command;
                        for command in data.iter() {
                            match command {
                                &Command::Move(..) => { /* … */ }
                                &Command::Line(..) => { /* … */ }
                                _ => {}
                            }
                        }
                    }
                    Event::Tag(tag::SVG, _, attributes) => {
                        if !attributes.is_empty() {
                            document = attributes.clone();
                        }
                    }
                    _ => {}
                }
            }

            MapData {
                polyline,
                points,
                document,
                debug: svg::node::element::Group::new(),
            }
    }
        */

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

pub fn map(
    backend: &backend::BackendData,
    segment: &segment::Segment,
    W: i32,
    H: i32,
    debug: bool,
) -> String {
    let svgMap = MapData::make(backend, segment, W, H, debug);
    svgMap.render()
}

#[cfg(test)]
mod tests {
    use crate::label_placement::*;

    use super::*;

    #[test]
    fn test_bbox() {
        let id = String::new();
        let target = PointFeature::new(
            id.clone(),
            Circle {
                id: id.clone(),
                cx: 0f64,
                cy: 0f64,
                r: 1f64,
                fill: None,
            },
            Label {
                id: id.clone(),
                bbox: LabelBoundingBox {
                    top_left: (0f64, 0f64),
                    bottom_right: (10f64, 16f64),
                },
                text: String::from_str("hi").unwrap(),
            },
        );
        let candidates = generate_candidates_bboxes(&target);
        let mut found = false;
        assert!(!candidates.is_empty());
        for c in candidates {
            let center = target.center();
            let good = c.top_left.0 > target.center().0 && c.top_left.1 > target.center().1;
            if good {
                found = true;
            }
        }
        assert!(found);
    }
}
