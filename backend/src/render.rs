#![allow(non_snake_case)]

use crate::gpsdata::{self, UTMPoint};

use svg::node::element::path::{Command, Data, Position};
use svg::node::element::Path;
use svg::Document;

fn to_view(x: f64, y: f64) -> (f64, f64) {
    ((x / 100f64), 250f64 - (y / 5f64))
}

pub struct ViewBox {
    tlx: f64,
    tly: f64,
    width: f64,
    height: f64,
}

impl ViewBox {
    pub fn from_track(track: &gpsdata::Track, range: &std::ops::Range<usize>) -> ViewBox {
        let start = track.distance(range.start);
        let end = track.distance(range.end - 1);
        let (TLx, TLy) = to_view(start, 1250f64);
        let width = end - start;
        let width = width.max(100000f64);
        let (W, H) = to_view(width, 0f64);
        ViewBox {
            tlx: TLx,
            tly: TLy,
            width: W,
            height: H,
        }
    }
}

pub fn track_profile(
    geodata: &gpsdata::Track,
    range: &std::ops::Range<usize>,
    viewbox: &ViewBox,
) -> String {
    let mut data = Data::new();
    for k in range.start..range.end {
        let (x, y) = (geodata.distance(k), geodata.elevation(k));
        let (xg, yg) = to_view(x, y);
        if data.is_empty() {
            data.append(Command::Move(Position::Absolute, (xg, yg).into()));
        }
        data.append(Command::Line(Position::Absolute, (xg, yg).into()));
    }

    let svgpath = Path::new()
        .set("fill", "none")
        .set("stroke", "black")
        .set("stroke-width", 2)
        .set("stroke-linecap", "round")
        .set("stroke-linejoin", "round")
        .set("d", data);

    let document = Document::new()
        .set(
            "viewBox",
            (viewbox.tlx, viewbox.tly, viewbox.width, viewbox.height),
        )
        .add(svgpath);

    document.to_string()
}

pub fn waypoints_profile(
    geodata: &gpsdata::Track,
    waypoints: &Vec<gpsdata::Waypoint>,
    range: &std::ops::Range<usize>,
    viewbox: &ViewBox,
) -> String {
    let mut document = Document::new().set(
        "viewBox",
        (viewbox.tlx, viewbox.tly, viewbox.width, viewbox.height),
    );

    for w in waypoints {
        let k = w.track_index;
        if !range.contains(&k) {
            continue;
        }
        let (x, y) = to_view(geodata.distance(k), geodata.elevation(k));
        let dot = svg::node::element::Circle::new()
            .set("cx", x)
            .set("cy", y)
            .set("r", 10);
        document = document.add(dot);
    }
    document.to_string()
}

fn profile_data(
    geodata: &gpsdata::Track,
    waypoints: &Vec<gpsdata::Waypoint>,
    range: &std::ops::Range<usize>,
) -> String {
    let mut data = Data::new();
    for k in range.start..range.end {
        let (x, y) = (geodata.distance(k), geodata.elevation(k));
        let (xg, yg) = to_view(x, y);
        if data.is_empty() {
            data.append(Command::Move(Position::Absolute, (xg, yg).into()));
        }
        data.append(Command::Line(Position::Absolute, (xg, yg).into()));
    }

    let svgpath = Path::new()
        .set("fill", "none")
        .set("stroke", "black")
        .set("stroke-width", 2)
        .set("stroke-linecap", "round")
        .set("stroke-linejoin", "round")
        .set("d", data);

    let start = geodata.distance(range.start);
    let end = geodata.distance(range.end - 1);
    let (TLx, TLy) = to_view(start, 1250f64);
    let width = end - start;
    let width = width.max(100000f64);
    let (W, H) = to_view(width, 0f64);
    let mut document = Document::new()
        .set("viewBox", (TLx, TLy, W, H))
        .add(svgpath);

    for w in waypoints {
        let k = w.track_index;
        if !range.contains(&k) {
            continue;
        }
        let (x, y) = to_view(geodata.distance(k), geodata.elevation(k));
        let dot = svg::node::element::Circle::new()
            .set("cx", x)
            .set("cy", y)
            .set("r", 10);
        document = document.add(dot);
    }
    document.to_string()
}

fn profile(
    geodata: &gpsdata::Track,
    waypoints: &Vec<gpsdata::Waypoint>,
    range: &std::ops::Range<usize>,
    filename: &str,
) {
    let data = profile_data(geodata, waypoints, range);
    println!("write {}", filename);
    std::fs::write(filename, data).expect("Unable to write file");
}

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
    fn update(&mut self, p: &gpsdata::UTMPoint) {
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

fn map(
    geodata: &gpsdata::Track,
    waypoints: &Vec<gpsdata::Waypoint>,
    range: &std::ops::Range<usize>,
    filename: &str,
) {
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

    svg::save(filename, &document).unwrap();
}

use std::io::prelude::*;
use std::str::FromStr;

fn read_file(filename: &str) -> String {
    let mut f = std::fs::File::open(filename).unwrap();
    let mut c = String::new();
    f.read_to_string(&mut c).unwrap();
    c
}

fn write_file(filename: &str, content: String) -> std::io::Result<()> {
    let mut f = std::fs::File::create(filename)?;
    f.write_all(content.as_bytes())?;
    Ok(())
}

struct Templates {
    header: String,
    table_large: String,
    table_points: String,
}

impl Templates {
    fn new() -> Templates {
        Templates {
            header: read_file("templates/header.typ"),
            table_large: read_file("templates/table-large.typ"),
            table_points: read_file("templates/table-points.typ"),
        }
    }
}

fn points_table(
    templates: &Templates,
    track: &gpsdata::Track,
    waypoints: &Vec<gpsdata::Waypoint>,
    range: &std::ops::Range<usize>,
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
        let tk = this.track_index;
        if !range.contains(&tk) {
            continue;
        }
        let mut copy = template_line.clone();
        copy = copy.replace("{name}", format!("{:02}", 1 + lines.len()).as_str());
        copy = copy.replace(
            "{distance}",
            format!("{:4.1}", track.distance(tk) / 1000f64).as_str(),
        );
        copy = copy.replace("{time}", "00:00");
        if k > 0 {
            let prev = &waypoints[k - 1];
            let pk = prev.track_index;
            debug_assert!(tk >= pk);
            let d0 = track.distance(pk);
            let d1 = track.distance(tk);
            debug_assert!(d1 >= d0);
            let _d = d1 - d0;
            let e0 = track.wgs84[pk].2;
            let e1 = track.wgs84[tk].2;
            let hm = e1 - e0;
            copy = copy.replace("{d+}", format!("{:.1} m", hm).as_str());
        } else {
            copy = copy.replace("{d+}", "0 m");
        }
        lines.push(copy);
    }
    let joined = lines.join("\n");
    table.replace(&template_line_orig, joined.as_str())
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
    table = table.replace("{profile.svg}", profilesvg);
    table = table.replace("{map.svg}", mapsvg);
    document.push_str(table.as_str());
}

pub fn compile(track: &gpsdata::Track, waypoints: &Vec<gpsdata::Waypoint>) -> String {
    let templates = Templates::new();
    let mut document = templates.header.clone();
    let km = 1000f64;
    let mut start = 0f64;
    let mut k = 0usize;
    loop {
        let end = start + 100f64 * km;
        let range = track.segment(start, end);
        if range.is_empty() {
            break;
        }
        let p = format!("/tmp/profile-{}.svg", k);
        profile(&track, &waypoints, &range, p.as_str());
        let m = format!("/tmp/map-{}.svg", k);
        map(&track, &waypoints, &range, m.as_str());
        let table = points_table(&templates, &track, &waypoints, &range);
        link(&templates, &p, &m, &table, &mut document);
        if range.end == track.len() {
            break;
        }
        start = start + 50f64 * km;
        k = k + 1;
    }
    let _ = write_file("/tmp/document.typ", document);
    String::from_str("/tmp/document.typ").unwrap()
}

pub fn svg(track: &gpsdata::Track, waypoints: &Vec<gpsdata::Waypoint>) -> String {
    let km = 1000f64;
    let start = 0f64;
    let end = start + 100f64 * km;
    let range = track.segment(start, end);
    profile_data(&track, &waypoints, &range)
}
