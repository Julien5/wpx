use crate::gpsdata;

use svg::node::element::path::{Command, Data, Position};
use svg::node::element::Path;
use svg::Document;

fn to_view(x: f64, y: f64) -> (f64, f64) {
    ((x / 100f64), 250f64 - (y / 5f64))
}

pub fn profile(geodata: &gpsdata::GeoData, range: &std::ops::Range<usize>, filename: &str) {
    let mut data = Data::new();
    let dist = geodata.distance(range.end - 1) - geodata.distance(range.start);
    println!(
        "range:{:?} L={} distance={}",
        range,
        range.len(),
        dist / 1000f64
    );
    for k in range.clone() {
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
    let mut document = Document::new()
        .set(
            "viewBox",
            (
                (start / 100f64).floor(),
                0,
                ((end - start) / 100f64).floor(),
                250,
            ),
        )
        .add(svgpath);

    let indices = geodata.get_automatic_points();
    for k in indices {
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
    svg::save(filename, &document).unwrap();
}

fn to_graphics_coordinates(x: f64, y: f64, ymax: f64) -> (f64, f64) {
    (x, ymax - y)
}

struct BoundingBox {
    pub min: (f64, f64),
    pub max: (f64, f64),
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
    fn update(&mut self, (x, y): (f64, f64)) {
        if x > self.max.0 {
            self.max.0 = x;
        }
        if y > self.max.1 {
            self.max.1 = y;
        }
        if x < self.min.0 {
            self.min.0 = x;
        }
        if y < self.min.1 {
            self.min.1 = y;
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
            let delta = 0.5f64 * (self.width());
            self.max.1 = Y + delta;
            self.min.1 = Y - delta;
        }
    }
}

pub fn map(geodata: &gpsdata::GeoData, range: &std::ops::Range<usize>, filename: &str) {
    let mut data = Data::new();
    let path = &geodata.utm;
    let mut bbox = BoundingBox::new();
    for k in range.clone() {
        bbox.update(geodata.utm[k]);
    }
    bbox.fix_aspect_ratio();
    for k in range.clone() {
        let (x, y) = path[k];
        let (xg, yg) = to_graphics_coordinates(x, y, bbox.max.1);
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

    let document = Document::new()
        .set("viewBox", (bbox.min.0, 0, bbox.width(), bbox.height()))
        .add(svgpath);

    svg::save(filename, &document).unwrap();
}

use std::io::prelude::*;
use std::str::FromStr;

pub fn read_file(filename: &str) -> String {
    let mut f = std::fs::File::open(filename).unwrap();
    let mut c = String::new();
    f.read_to_string(&mut c).unwrap();
    c
}

pub fn write_file(filename: &str, content: String) -> std::io::Result<()> {
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

fn link(templates: &Templates, profilesvg: &str, mapsvg: &str, document: &mut String) {
    let mut table = templates.table_large.clone();
    table = table.replace("{table-points}", templates.table_points.as_str());
    table = table.replace("{profile.svg}", profilesvg);
    table = table.replace("{map.svg}", mapsvg);
    document.push_str(table.as_str());
}

pub fn compile(geodata: &gpsdata::GeoData) -> String {
    let templates = Templates::new();
    let mut document = templates.header.clone();
    let km = 1000f64;
    let mut start = 0f64;
    let mut k = 0usize;
    loop {
        let end = start + 100f64 * km;
        let range = geodata.segment(start, end);
        if range.is_empty() {
            break;
        }
        let p = format!("/tmp/profile-{}.svg", k);
        profile(&geodata, &range, p.as_str());
        let m = format!("/tmp/map-{}.svg", k);
        map(&geodata, &range, m.as_str());
        link(&templates, &p, &m, &mut document);
        if range.end == geodata.len() {
            break;
        }
        start = start + 50f64 * km;
        k = k + 1;
    }
    let _ = write_file("/tmp/document.typ", document);
    String::from_str("/tmp/document.typ").unwrap()
}
