#![allow(non_snake_case)]

use clap::Parser;
use tracks::backend::Backend;
use tracks::error;

/// Search for a pattern in a file and display the lines that contain it.
#[derive(Parser)]
struct Cli {
    #[arg(long, value_name = "debug")]
    debug: Option<bool>,
    #[arg(long, value_name = "outdir")]
    output_directory: Option<std::path::PathBuf>,
    #[arg(long, value_name = "segment_length")]
    segment_length: Option<i32>,
    #[arg(long, value_name = "segment_overlap")]
    segment_overlap: Option<i32>,
    #[arg(long, value_name = "start_time")]
    start_time: Option<String>,
    #[arg(long, value_name = "step_distance")]
    step_distance: Option<usize>,
    #[arg(long, value_name = "step_elevation_gain")]
    step_elevation_gain: Option<usize>,
    #[arg(long, value_name = "profile_max_area_ratio")]
    profile_max_area_ratio: Option<f64>,
    #[arg(long, value_name = "map_max_area_ratio")]
    map_max_area_ratio: Option<f64>,
    #[arg(value_name = "gpx")]
    filename: std::path::PathBuf,
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), error::Error> {
    env_logger::init();
    /*env_logger::Builder::new()
    .format(|buf, record| {
        writeln!(
            buf,
            "{} [{}] - {}",
            chrono::Local::now().format("%M:%S:%3f"),
            record.level(),
            record.args()
        )
    })
    .filter(None, log::LevelFilter::Trace)
    .init();*/

    let args = Cli::parse();

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

    log::info!("read gpx {}", gpxinput);
    log::info!("outdir   {}", outdir);
    let mut backend = Backend::make();
    backend.load_filename(gpxinput).await?;

    let mut parameters = backend.get_parameters();
    match args.segment_length {
        Some(length) => {
            parameters.segment_length = 1000f64 * (length as f64);
        }
        _ => {}
    }

    match args.segment_overlap {
        Some(length) => {
            parameters.segment_overlap = 1000f64 * (length as f64);
        }
        _ => {}
    }

    match args.start_time {
        Some(time) => {
            parameters.start_time = time.clone();
        }
        _ => {}
    }

    match args.step_distance {
        Some(km) => {
            parameters.profile_options.step_distance = Some((1000 * km) as f64);
        }
        _ => {}
    }

    match args.step_elevation_gain {
        Some(m) => {
            parameters.profile_options.step_elevation_gain = Some(m as f64);
        }
        _ => {}
    }

    match args.map_max_area_ratio {
        Some(m) => {
            parameters.map_options.max_area_ratio = m;
        }
        _ => {}
    }

    match args.profile_max_area_ratio {
        Some(m) => {
            parameters.profile_options.max_area_ratio = m;
        }
        _ => {}
    }

    match args.debug {
        Some(d) => {
            parameters.debug = d;
        }
        _ => {}
    }

    backend.set_parameters(&parameters);
    let stats = backend.statistics();
    log::info!("length = {:.1} km", stats.length / 1000f64);
    log::info!("elevation gain = {:.1} km", stats.elevation_gain);

    let pdfbytes = backend.generatePdf().await;
    let pdfname = format!(
        "{}/{}.pdf",
        outdir,
        gpxpath.file_stem().unwrap().to_str().unwrap()
    );
    log::info!("make: {}", pdfname);
    std::fs::write(pdfname, &pdfbytes).expect("Could not write pdf.");

    let gpxbytes = backend.generateGpx();
    let gpxname = format!(
        "{}/{}-waypoints.gpx",
        outdir,
        gpxpath.file_stem().unwrap().to_str().unwrap()
    );
    log::info!("make: {}", gpxname);
    std::fs::write(gpxname, &gpxbytes).expect("Could not write gpx.");

    Ok(())
}
