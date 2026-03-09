#![allow(non_snake_case)]

use std::collections::HashSet;

use clap::Parser;
use tracks::backend::Backend;
use tracks::math::IntegerSize2D;
use tracks::parameters::{ControlSource, RenderFunction};
use tracks::point_collection::Kind;
use tracks::{error, parameters};
use tracks::{point_collection, speed};

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
    #[arg(long, value_name = "speed")]
    speed: Option<f64>,
    #[arg(long, value_name = "step_distance")]
    step_distance: Option<usize>,
    #[arg(long, value_name = "step_elevation_gain")]
    step_elevation_gain: Option<usize>,
    #[arg(long, value_name = "profile_max_area_ratio")]
    profile_max_area_ratio: Option<f64>,
    #[arg(long, value_name = "map_max_area_ratio")]
    map_max_area_ratio: Option<f64>,
    #[arg(long, value_name = "render_wheel")]
    render_wheel: Option<bool>,
    #[arg(long, value_name = "main-test")]
    main_test: Option<bool>,
    #[arg(long, value_name = "render-graph")]
    render_graph: Option<bool>,
    #[arg(value_name = "gpx")]
    filename: std::path::PathBuf,
}

async fn render_graph(backend: &mut Backend) -> anyhow::Result<()> {
    let segment = backend.trackSegment();

    //let map_size = IntegerSize2D::new(839, 349);
    //let profile_size = IntegerSize2D::new(864, 255);

    let map_size = IntegerSize2D::new(1479, 778);
    let profile_size = IntegerSize2D::new(1504, 255);

    let ret = backend.render_segment_map_profile(
        &segment,
        &map_size,
        &profile_size,
        HashSet::from([
            Kind::Cities,
            Kind::Villages,
            Kind::Hamlets,
            Kind::GPXWaypoints,
            Kind::Controls,
        ]),
    );
    let tmpfilename = std::format!("/tmp/rendergraph-map.svg");
    std::fs::write(&tmpfilename, ret[0].svg.clone()).unwrap();
    let tmpfilename = std::format!("/tmp/rendergraph-profile.svg");
    std::fs::write(&tmpfilename, ret[1].svg.clone()).unwrap();
    Ok(())
}

async fn main_test(backend: &mut Backend) -> anyhow::Result<()> {
    let start = std::time::Instant::now();
    let segment = backend.trackSegment();
    let _ = backend.load_osm().await;
    let mut svg = String::new();
    for _ in 1..30 {
        svg = backend.render_segment_simple(
            &segment,
            &IntegerSize2D::new(2000, 1000),
            point_collection::allkinds(),
            RenderFunction::Map,
        );
    }

    let duration = start.elapsed();
    log::info!("main_test map took: {:.3?}", duration);
    let tmpfilename = std::format!("/tmp/maintestmap.svg");
    std::fs::write(&tmpfilename, svg.clone()).unwrap();

    for _ in 1..30 {
        svg = backend.render_segment_simple(
            &segment,
            &IntegerSize2D::new(2000, 400),
            point_collection::allkinds(),
            RenderFunction::Profile,
        );
    }

    let duration = start.elapsed();
    log::info!("main_test profile took: {:.3?}", duration);
    let tmpfilename = std::format!("/tmp/maintestprofile.svg");

    std::fs::write(&tmpfilename, svg.clone()).unwrap();
    Ok(())
}

fn setup_log() {
    println!("init logger");
    use std::io::Write;
    let _ = env_logger::Builder::new()
        .format(|buf, record| {
            writeln!(
                buf,
                "{} [{}] - {}",
                chrono::Local::now().format("%H:%M:%S:%f"),
                record.level(),
                record.args()
            )
        })
        .filter_level(log::LevelFilter::Trace)
        .try_init();
}

pub fn read_file(filename: &str) -> Vec<u8> {
    let mut f = std::fs::File::open(filename).unwrap();
    let mut buffer = Vec::new();
    // read the whole file
    use std::io::prelude::*;
    f.read_to_end(&mut buffer).unwrap();
    buffer
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> anyhow::Result<()> {
    // env_logger::init();
    setup_log();

    let args = Cli::parse();

    let gpxinput;
    if args.filename.exists() {
        gpxinput = args.filename.as_os_str().to_str().unwrap();
    } else {
        let e = error::TrackError::GPXNotFound;
        return Err(e.into());
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
    let gpxdata = read_file(gpxinput);
    let parts = backend.load_track_parts(&vec![gpxdata]).await?;
    let parts = parameters::karl_order(&parts);
    backend.load_ordered(&parts).await?;
    let _ = backend.load_osm().await;
    backend.load_controls(ControlSource::Segments).await?;

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

    match args.speed {
        Some(speed) => {
            parameters.speed = speed::mps(speed);
        }
        _ => {}
    }

    match args.step_distance {
        Some(km) => {
            parameters.user_steps_options.step_elevation_gain = None;
            parameters.user_steps_options.step_distance = Some((1000 * km) as f64);
        }
        _ => {}
    }

    match args.step_elevation_gain {
        Some(m) => {
            parameters.user_steps_options.step_distance = None;
            parameters.user_steps_options.step_elevation_gain = Some(m as f64);
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

    match args.main_test {
        Some(enabled) => {
            if enabled {
                return main_test(&mut backend).await;
            }
        }
        _ => {}
    }

    match args.render_graph {
        Some(enabled) => {
            if enabled {
                return render_graph(&mut backend).await;
            }
        }
        _ => {}
    }

    match args.render_wheel {
        Some(enabled) => {
            if enabled {
                let track_segment = backend.trackSegment();
                let size = IntegerSize2D::new(250, 250);
                let svg = backend.render_segment_simple(
                    &track_segment,
                    &size,
                    point_collection::allkinds(),
                    RenderFunction::Wheel,
                );
                let filename = std::format!("/tmp/wheel.svg");
                std::fs::write(&filename, svg.clone()).unwrap();
                return Ok(());
            }
        }
        _ => {}
    }

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
