#![allow(non_snake_case)]

use crate::backend::{Backend, Segment, Step};
use crate::gpsdata;
use crate::utm::UTMPoint;

use svg::node::element::path::{Command, Data, Position};
use svg::node::element::Path;
use svg::Document;

fn to_graphics_coordinates(p: &UTMPoint, ymax: f64) -> (f64, f64) {
    (p.x(), ymax - p.y())
}

struct BoundingBox {
    min: (f64, f64),
    max: (f64, f64),
}

impl BoundingBox {
    fn new() -> BoundingBox {
        let min = (f64::MAX, f64::MAX);
        let max = (f64::MIN, f64::MIN);
        BoundingBox { min, max }
    }
    fn width(&self) -> f64 {
        return self.max.0 - self.min.0;
    }
    fn height(&self) -> f64 {
        return self.max.1 - self.min.1;
    }
    fn update(&mut self, p: &UTMPoint) {
        if p.x() > self.max.0 {
            self.max.0 = p.x();
        }
        if p.y() > self.max.1 {
            self.max.1 = p.y();
        }
        if p.x() < self.min.0 {
            self.min.0 = p.x();
        }
        if p.y() < self.min.1 {
            self.min.1 = p.y();
        }
    }
    fn fix_aspect_ratio(&mut self) {
        let X = (self.min.0 + self.max.0) / 2f64;
        let Y = (self.min.1 + self.max.1) / 2f64;
        if self.height() > self.width() {
            let delta = 0.5f64 * (self.height());
            self.max.0 = X + delta;
            self.min.0 = X - delta;
        } else {
            let delta = 0.5f64 * self.width();
            self.max.1 = Y + delta;
            self.min.1 = Y - delta;
        }
    }
}

fn map(geodata: &gpsdata::Track, waypoints: &Vec<Step>, range: &std::ops::Range<usize>) -> String {
    let mut data = Data::new();
    let path = &geodata.utm;
    let mut bbox = BoundingBox::new();
    for k in range.clone() {
        bbox.update(&geodata.utm[k]);
    }
    bbox.fix_aspect_ratio();
    for k in range.clone() {
        let p = &path[k];
        let (xg, yg) = to_graphics_coordinates(p, bbox.max.1);
        if data.is_empty() {
            data.append(Command::Move(Position::Absolute, (xg, yg).into()));
        }
        data.append(Command::Line(Position::Absolute, (xg, yg).into()));
    }

    let svgpath = Path::new()
        .set("fill", "none")
        .set("stroke", "black")
        .set("stroke-width", 2)
        .set("d", data);

    let mut document = Document::new()
        .set("viewBox", (bbox.min.0, 0, bbox.width(), bbox.height()))
        .add(svgpath);

    for w in waypoints {
        let k = w.track_index;
        if !range.contains(&k) {
            continue;
        }
        let (x, y) = to_graphics_coordinates(&w.utm, bbox.max.1);
        let dot = svg::node::element::Circle::new()
            .set("cx", x)
            .set("cy", y)
            .set("r", 500);
        document = document.add(dot);
    }
    document.to_string()
}

use std::str::FromStr;

struct Templates {
    header: String,
    table_large: String,
    table_points: String,
}

impl Templates {
    fn new() -> Templates {
        Templates {
            header: String::from_str(include_str!("../templates/header.typ")).unwrap(),
            table_large: String::from_str(include_str!("../templates/table-large.typ")).unwrap(),
            table_points: String::from_str(include_str!("../templates/table-points.typ")).unwrap(),
        }
    }
}

fn points_table(
    templates: &Templates,
    track: &gpsdata::Track,
    waypoints: &Vec<Step>,
    segment: &Segment,
) -> String {
    let table = templates.table_points.clone();
    let mut template_line_orig = String::new();
    let mut template_line = String::new();
    for line in table.split("\n") {
        if line.contains("/* #line-template") {
            template_line_orig = String::from_str(line).unwrap();
            template_line = template_line_orig.clone();
            template_line = template_line.replace("/* #line-template", "");
            template_line = template_line.replace("*/", "");
        }
    }
    debug_assert!(!template_line.is_empty());
    // TODO: avoid recomputing the automatic points
    let mut lines = Vec::new();
    for k in 0..waypoints.len() {
        let this = &waypoints[k];
        if !segment.profile.shows_waypoint(this) {
            continue;
        }
        let mut copy = template_line.clone();
        copy = copy.replace("{name}", this.name.as_str());
        let datetime = chrono::DateTime::parse_from_rfc3339(this.time.as_str()).unwrap();
        let time_str = format!("{}", datetime.format("%H:%M"));

        copy = copy.replace("{time}", &time_str);

        copy = copy.replace(
            "{distance}",
            format!("{:4.1}", track.distance(this.track_index) / 1000f64).as_str(),
        );
        let elevation = this.elevation;
        copy = copy.replace("{elevation}", format!("{:5.0} m", elevation).as_str());
        let hm = this.inter_elevation_gain;
        let percent = this.inter_slope;
        copy = copy.replace("{d+}", format!("{:5.0}", hm).as_str());
        copy = copy.replace("{slope}", format!("{:2.1}%", percent).as_str());
        copy = copy.replace("{desc}", "");
        let dist = this.inter_distance / 1000f64;
        copy = copy.replace("{dist}", format!("{:2.1}", dist).as_str());
        lines.push(copy);
    }
    let joined = lines.join("\n");
    table.replace(&template_line_orig, joined.as_str())
}

fn get_typst_bytes(ascii: &str) -> String {
    let mut ret = Vec::new();
    for c in ascii.chars() {
        let code = format!("{:?}", c as u32);
        ret.push(code);
    }
    let rc = ret.join(",");
    format!("bytes(({}))", rc)
}

fn link(
    templates: &Templates,
    profilesvg: &str,
    mapsvg: &str,
    points_table: &String,
    document: &mut String,
) {
    let mut table = templates.table_large.clone();
    table = table.replace("{table-points}", points_table.as_str());
    table = table.replace("{profile.svg}", get_typst_bytes(profilesvg).as_str());
    table = table.replace("{map.svg}", get_typst_bytes(mapsvg).as_str());
    document.push_str(table.as_str());
}

pub fn compile(backend: &mut Backend, (W, H): (i32, i32)) -> String {
    let templates = Templates::new();
    let mut document = templates.header.clone();
    let mut start = 0f64;
    let mut k = 0usize;
    let segments = backend.segments();
    for segment in &segments {
        let range = &segment.range;
        if range.is_empty() {
            break;
        }
        let WP = backend.get_waypoints();
        let p = backend.render_segment(segment, (W, H));
        let m = map(&backend.track, &WP, &range);
        let table = points_table(&templates, &backend.track, &WP, &segment);
        link(&templates, &p, &m, &table, &mut document);
        if range.end == backend.track.len() {
            break;
        }
        start = start + backend.segment_length;
        k = k + 1;
    }
    document
}
