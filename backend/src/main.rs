#![allow(non_snake_case)]

pub mod backend;
pub mod elevation;
pub mod gpsdata;
pub mod pdf;
pub mod project;
pub mod render;
pub mod speed;
pub mod svgprofile;
pub mod utm;

use backend::Backend;
use std::env;

fn main() {
    let args: Vec<String> = env::args().collect();
    let mut filename = "data/blackforest.gpx";
    if args.len() > 1usize {
        filename = &args[1];
    }
    println!("read gpx {}", filename);
    let mut backend = Backend::from_filename(filename);
    let gpxpath = std::path::Path::new(filename);
    let dir = gpxpath.parent().unwrap().to_str().unwrap();
    let typbytes = render::compile(&mut backend, (1400, 400));
    std::fs::write("/tmp/d.typ", &typbytes).expect("Could not write typst.");
    let pdfbytes = pdf::compile(&typbytes);
    let pdfname = format!(
        "{}/{}.pdf",
        dir,
        gpxpath.file_stem().unwrap().to_str().unwrap()
    );
    println!("make: {}", pdfname);
    std::fs::write(pdfname, &pdfbytes).expect("Could not write pdf.");

    println!("test backend");
    let backend = Backend::from_filename(filename);
    let W = backend.get_waypoints();
    for w in W {
        println!(
            "waypoing dist={:6.2} ele={:5.1}",
            (w.distance / 1000f64),
            w.elevation
        );
    }
}
