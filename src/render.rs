use crate::gpsdata;

use svg::node::element::path::{Command, Data, Position};
use svg::node::element::Path;
use svg::Document;

fn to_view(x: f64, y: f64) -> (f64, f64) {
    ((x / 100f64), 250f64 - (y / 5f64))
}

pub fn profile(geodata: &gpsdata::GeoData, range: &std::ops::Range<usize>, filename: &str) {
    let mut data = Data::new();
    println!("range:{:?} L={}", range, range.len());
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

fn bbox(geodata: &gpsdata::GeoData, range: &std::ops::Range<usize>) -> (f64, f64, f64, f64) {
    let path = &geodata.utm;
    let mut xmin = f64::MAX;
    let mut xmax = f64::MIN;
    let mut ymin = f64::MAX;
    let mut ymax = f64::MIN;
    for k in range.clone() {
        let (x, y) = path[k];
        if x > xmax {
            xmax = x;
        }
        if y > ymax {
            ymax = y;
        }
        if x < xmin {
            xmin = x;
        }
        if y < ymin {
            ymin = y;
        }
    }
    (xmin, ymin, xmax, ymax)
}

pub fn map(geodata: &gpsdata::GeoData, range: &std::ops::Range<usize>, filename: &str) {
    let mut data = Data::new();
    let path = &geodata.utm;
    let (xmin, ymin, xmax, ymax) = bbox(geodata, range);
    for k in range.clone() {
        let (x, y) = path[k];
        let (xg, yg) = to_graphics_coordinates(x, y, ymax);
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
        .set("viewBox", (xmin, 0, xmax - xmin, ymax - ymin))
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
        start = start + 50f64 * km;
        k = k + 1;
    }
    let _ = write_file("/tmp/document.typ", document);
    String::from_str("/tmp/document.typ").unwrap()
}
