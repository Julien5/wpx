#![allow(non_snake_case)]

pub mod automatic;
pub mod backend;
pub mod elevation;
pub mod gpsdata;
pub mod gpxexport;
pub mod parameters;
pub mod pdf;
pub mod project;
pub mod render;
pub mod speed;
pub mod step;
pub mod svgprofile;
pub mod utm;

use backend::Backend;

use clap::Parser;

/// Search for a pattern in a file and display the lines that contain it.
#[derive(Parser)]
struct Cli {
    #[arg(short, long, value_name = "outdir")]
    output_directory: Option<std::path::PathBuf>,
    #[arg(value_name = "gpx")]
    filename: std::path::PathBuf,
}

fn main() {
    let args = Cli::parse();
    println!("args: {:?}", args.output_directory);

    let mut gpxinput = "data/blackforest.gpx";
    if args.filename.exists() {
        gpxinput = args.filename.as_os_str().to_str().unwrap();
    }

    let gpxpath = std::path::Path::new(gpxinput);
    let mut outdir = gpxpath.parent().unwrap().to_str().unwrap();
    match &args.output_directory {
        Some(path) => outdir = path.to_str().unwrap(),
        _ => {}
    }

    println!("read gpx {}", gpxinput);
    println!("outdir   {}", outdir);
    let mut backend = Backend::from_filename(gpxinput);

    let _gpxname = format!(
        "{}/{}.gpx",
        outdir,
        gpxpath.file_stem().unwrap().to_str().unwrap()
    );

    let pdfbytes = backend.generatePdf();
    let pdfname = format!(
        "{}/{}.pdf",
        outdir,
        gpxpath.file_stem().unwrap().to_str().unwrap()
    );
    println!("make: {}", pdfname);
    std::fs::write(pdfname, &pdfbytes).expect("Could not write pdf.");

    let gpxbytes = backend.generateGpx();
    let gpxname = format!(
        "{}/{}-waypoints.gpx",
        outdir,
        gpxpath.file_stem().unwrap().to_str().unwrap()
    );
    println!("make: {}", gpxname);
    std::fs::write(gpxname, &gpxbytes).expect("Could not write gpx.");

    println!("test backend");
    let backend = Backend::from_filename(gpxinput);
    let W = backend.get_steps();
    for w in W {
        println!(
            "waypoint dist={:6.2} ele={:5.1}",
            (w.distance / 1000f64),
            w.elevation
        );
    }
}
