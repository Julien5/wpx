use crate::gpsdata;
use crate::minimap;

use svg::node::element::path::{Command, Data, Position};
use svg::node::element::Path;
use svg::Document;

fn to_view(x: f64, y: f64) -> (f64, f64) {
    (10f64 * (x / 1000f64), 250f64 - (y / 5f64))
}

pub fn profile(geodata: &gpsdata::GeoData, filename: &str) {
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
    svg::save(filename, &document).unwrap();
}

fn to_graphics_coordinates(x: f64, y: f64, ymax: f64) -> (f64, f64) {
    (x, ymax - y)
}

pub fn map(path: &minimap::Path) {
    let mut data = Data::new();
    for (x, y) in &path.path {
        let (xg, yg) = to_graphics_coordinates(*x, *y, path.max.1 as f64);
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
        .set(
            "viewBox",
            (
                path.min.0,
                0,
                path.max.0 - path.min.0,
                path.max.1 - path.min.1,
            ),
        )
        .add(svgpath);

    svg::save("/tmp/map.svg", &document).unwrap();
}
