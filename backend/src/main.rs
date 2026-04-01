#![allow(non_snake_case)]

use std::collections::HashSet;

use clap::Parser;
use tracks::backend::Backend;
use tracks::error;
use tracks::math::IntegerSize2D;
use tracks::parameters::{ControlSource, RenderFunction};
use tracks::point_collection::Kind;
use tracks::{point_collection, speed};

/// Search for a pattern in a file and display the lines that contain it.
#[derive(Parser)]
struct Cli {
    #[arg(long, value_name = "debug")]
    debug: Option<bool>,
    /// filename for the ouput (a zip file)
    #[arg(long, value_name = "zip")]
    output: Option<std::path::PathBuf>,
    /// the segment length in kilometer
    #[arg(long, value_name = "segment_length")]
    segment_length: Option<i32>,
    /// the segment overlap in kilometer
    #[arg(long, value_name = "segment_overlap")]
    segment_overlap: Option<i32>,
    /// start date time in ISO 8601 format, like 2026-01-10T20:00
    #[arg(long, value_name = "start_time")]
    start_time: Option<String>,
    /// speed in kilometer per hour
    #[arg(long, value_name = "speed")]
    speed: Option<f64>,
    /// generate one pacing point every [distance] kilometer
    #[arg(long, value_name = "step_distance")]
    step_distance: Option<usize>,
    /// generate one pacing point every [evelation gain] meter
    #[arg(long, value_name = "step_elevation_gain")]
    step_elevation_gain: Option<usize>,
    #[arg(long, value_delimiter = ',', default_values_t = [Kind::Controls,Kind::GPXWaypoints,Kind::Mountains, Kind::Cities],value_name = "kinds")]
    kinds: Vec<Kind>,
    #[arg(long, value_name = "render_wheel", hide = true)]
    render_wheel: Option<bool>,
    #[arg(long, value_name = "performance-test", hide = true)]
    performance_test: Option<bool>,
    #[arg(long, value_name = "render-graph", hide = true)]
    render_graph: Option<bool>,

    #[arg(value_name = "gpx")]
    filenames: Vec<std::path::PathBuf>,
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

async fn performance_test(backend: &mut Backend) -> anyhow::Result<()> {
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
    println!("main_test map took: {:.3?}", duration);
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
    println!("main_test profile took: {:.3?}", duration);
    let tmpfilename = std::format!("/tmp/maintestprofile.svg");

    std::fs::write(&tmpfilename, svg.clone()).unwrap();
    Ok(())
}

fn setup_log() {
    // println!("init logger");
    env_logger::init();
    /*
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
    */
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

    let gpxinputs: Vec<_>;
    if !args.filenames.is_empty() {
        gpxinputs = args
            .filenames
            .iter()
            .map(|p| p.as_os_str().to_str().unwrap())
            .collect();
    } else {
        let e = error::TrackError::GPXNotFound;
        return Err(e.into());
    }
    assert!(!gpxinputs.is_empty());

    let gpxpath = std::path::Path::new(gpxinputs.first().unwrap());

    let mut backend = Backend::make();
    let mut gpxdata = Vec::new();
    for gpxinput in &gpxinputs {
        log::info!("read gpx {}", gpxinput);
        gpxdata.push(read_file(gpxinput));
    }
    let parts = backend.load_track_parts(&gpxdata).await?;
    for part in &parts {
        println!("found segment: {}", part.name)
    }
    backend.load_ordered(&parts).await?;
    let _ = backend.load_osm().await;
    backend.load_controls(ControlSource::Segments).await?;

    let track_segment = backend.trackSegment();
    {
        let points = backend.get_waypoints(
            &track_segment,
            point_collection::Kinds::from([Kind::Controls, Kind::GPXWaypoints]),
        );
        println!("* found {} controls/waypoints", points.len());
        for point in &points {
            println!(
                "   {:>3.0} km: {} ",
                point.get_info().distance / 1000.0,
                point.name
            )
        }
    }

    {
        let points = backend.get_waypoints(
            &track_segment,
            point_collection::Kinds::from([Kind::Cities, Kind::Mountains]),
        );
        println!("* found {} cities/mountains", points.len());
        for point in &points {
            println!(
                "   {:>3.0} km: {}",
                point.get_info().distance / 1000.0,
                point.name,
            )
        }
    }

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

    match args.debug {
        Some(d) => {
            parameters.debug = d;
        }
        _ => {}
    }

    backend.set_parameters(&parameters);

    match args.performance_test {
        Some(enabled) => {
            if enabled {
                return performance_test(&mut backend).await;
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
    println!("length = {:.1} km", stats.length / 1000f64);
    println!("elevation gain = {:.1} m", stats.elevation_gain);

    let kinds: HashSet<Kind> = args.kinds.into_iter().collect();

    let zipname = match args.output {
        Some(path) => path.into_os_string().into_string().unwrap_or_default(),
        None => format!("{}.zip", gpxpath.file_stem().unwrap().to_str().unwrap()),
    };
    let zip = backend.generateZip(&kinds).await;
    println!("make: {}", zipname);
    std::fs::write(zipname, &zip).expect("Could not write pdf.");
    Ok(())
}
