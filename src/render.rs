use crate::gpsdata;

use svg::node::element::path::{Command, Data, Position};
use svg::node::element::Path;
use svg::Document;

fn to_view(x: f64, y: f64) -> (f64, f64) {
    (10f64 * (x / 1000f64), 250f64 - (y / 5f64))
}

pub fn profile(geodata: &gpsdata::GeoData) -> String {
    let mut data = Data::new();
    for k in 0..geodata.len() {
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

    let mut document = Document::new()
        .set("viewBox", (0, 0, 1000, 250))
        .add(svgpath);

    let indices = geodata.get_automatic_points();
    for k in indices {
        let (x, y) = to_view(geodata.distance(k), geodata.elevation(k));
        let dot = svg::node::element::Circle::new()
            .set("cx", x)
            .set("cy", y)
            .set("r", 10);
        document = document.add(dot);
    }
    let filename = String::from_str("/tmp/profile.svg").unwrap();
    svg::save(filename.as_str(), &document).unwrap();
    filename
}

fn to_graphics_coordinates(x: f64, y: f64, ymax: f64) -> (f64, f64) {
    (x, ymax - y)
}

fn bbox(geodata: &gpsdata::GeoData) -> (f64, f64, f64, f64) {
    let path = &geodata.utm;
    let mut xmin = f64::MAX;
    let mut xmax = f64::MIN;
    let mut ymin = f64::MAX;
    let mut ymax = f64::MIN;
    for (x, y) in path {
        if *x > xmax {
            xmax = *x;
        }
        if *y > ymax {
            ymax = *y;
        }
        if *x < xmin {
            xmin = *x;
        }
        if *y < ymin {
            ymin = *y;
        }
    }
    (xmin, ymin, xmax, ymax)
}

pub fn map(geodata: &gpsdata::GeoData) -> String {
    let mut data = Data::new();
    let path = &geodata.utm;
    let (xmin, ymin, xmax, ymax) = bbox(geodata);
    for (x, y) in path {
        let (xg, yg) = to_graphics_coordinates(*x, *y, ymax);
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

    let ret = String::from_str("/tmp/map.svg").unwrap();
    svg::save(ret.as_str(), &document).unwrap();
    ret
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

fn link(templates: &Templates, document: &mut String, profilesvg: &String, mapsvg: &String) {
    let mut table = templates.table_large.clone();
    table = table.replace("{table-points}", templates.table_points.as_str());
    table = table.replace("{profile.svg}", profilesvg);
    table = table.replace("{map.svg}", mapsvg);
    document.push_str(table.as_str());
}

pub fn compile(geodata: &gpsdata::GeoData) -> String {
    let templates = Templates::new();
    let mut document = templates.header.clone();
    let profilesvg = profile(&geodata);
    let mapsvg = map(&geodata);
    link(&templates, &mut document, &profilesvg, &mapsvg);
    let _ = write_file("/tmp/document.typ", document);
    String::from_str("/tmp/document.typ").unwrap()
}
