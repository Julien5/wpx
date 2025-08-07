#![allow(non_snake_case)]

use clap::Parser;
use tracks::backend::Backend;
use tracks::error;
use tracks::render_device;

/// Search for a pattern in a file and display the lines that contain it.
#[derive(Parser)]
struct Cli {
    #[arg(short, long, value_name = "debug")]
    debug: Option<bool>,
    #[arg(short, long, value_name = "outdir")]
    output_directory: Option<std::path::PathBuf>,
    #[arg(short, long, value_name = "interval_length")]
    interval_length: Option<i32>,
    #[arg(short, long, value_name = "max_step_length")]
    max_step_length: Option<i32>,
    #[arg(value_name = "gpx")]
    filename: std::path::PathBuf,
}

fn main() -> Result<(), error::Error> {
    let args = Cli::parse();
    let debug = match args.debug {
        Some(v) => v,
        None => false,
    };

    let gpxinput;
    if args.filename.exists() {
        gpxinput = args.filename.as_os_str().to_str().unwrap();
    } else {
        let e = error::Error::GPXNotFound;
        return Err(e);
    }

    let gpxpath = std::path::Path::new(gpxinput);
    let mut outdir = gpxpath.parent().unwrap().to_str().unwrap();
    match &args.output_directory {
        Some(path) => outdir = path.to_str().unwrap(),
        _ => {}
    }

    println!("read gpx {}", gpxinput);
    println!("outdir   {}", outdir);
    let mut backend = Backend::from_filename(gpxinput)?;

    let mut parameters = backend.get_parameters();
    match args.interval_length {
        Some(length) => {
            parameters.segment_length = 1000f64 * (length as f64);
        }
        _ => {}
    }

    match args.max_step_length {
        Some(length) => {
            parameters.max_step_size = 1000f64 * (length as f64);
        }
        _ => {}
    }

    backend.set_parameters(&parameters);

    let pdfbytes = backend.generatePdf(debug);
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

    let segments = backend.segments();
    for k in 0..segments.len() {
        let segment = &segments[k];
        let bytes = backend.render_yaxis_labels_overlay(
            &segment,
            (1000, 285),
            render_device::RenderDevice::Native,
        );
        let name = format!("/tmp/yaxis-{}.svg", k);
        println!("make: {}", name);
        std::fs::write(name, &bytes).expect("Could not write gpx.");
    }
    Ok(())
}
