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
    let mut backend = Backend::new(filename);
    let typfile = render::compile(&mut backend, (1400, 400));
    println!("make pdf");
    let pdffile = typfile.replace(".typ", ".pdf");
    pdf::run(typfile.as_str(), pdffile.as_str());

    println!("test backend");
    let backend = Backend::new(filename);
    let W = backend.get_waypoints();
    for w in W {
        println!(
            "waypoing dist={:6.2} ele={:5.1}",
            (w.distance / 1000f64),
            w.elevation
        );
    }
}
