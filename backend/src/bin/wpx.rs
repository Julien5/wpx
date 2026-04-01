#![allow(non_snake_case)]

use std::collections::HashSet;

use clap::Parser;
use tracks::backend::Backend;
use tracks::error;
use tracks::math::IntegerSize2D;
use tracks::parameters::{parse_time, ControlSource, RenderFunction};
use tracks::point_collection::Kind;
use tracks::{point_collection, speed};

/// Search for a pattern in a file and display the lines that contain it.
#[derive(Parser)]
struct Cli {
    /// the segment length in kilometer
    #[arg(long, value_name = "segment_length", default_value_t = 110.0)]
    segment_length: f64,
    /// the segment overlap in kilometer
    #[arg(long, value_name = "segment_overlap", default_value_t = 10.0)]
    segment_overlap: f64,
    /// start date time in ISO 8601 format, like 2026-01-10T20:00 [default: now]
    #[arg(long, value_name = "start_time")]
    start_time: Option<String>,
    /// speed in kilometer per hour
    #[arg(long, value_name = "speed", default_value_t = 15.0)]
    speed: f64,
    /// generate one pacing point every [distance] kilometer [default: 10]
    #[arg(long, value_name = "step_distance")]
    step_distance: Option<f64>,
    /// generate one pacing point every [evelation gain] meter
    #[arg(long, value_name = "step_elevation_gain")]
    step_elevation_gain: Option<f64>,
    #[arg(long, value_delimiter = ',', default_values_t = [Kind::Controls,Kind::GPXWaypoints,Kind::Cities,Kind::Mountains, Kind::Villages,Kind::Hamlets,Kind::UserStep],value_name = "kinds")]
    kinds: Vec<Kind>,

    #[arg(long, value_name = "render_wheel", hide = true)]
    render_wheel: Option<bool>,
    #[arg(long, value_name = "performance-test", hide = true)]
    performance_test: Option<bool>,
    #[arg(long, value_name = "render-graph", hide = true)]
    render_graph: Option<bool>,
    #[arg(long, value_name = "debug", hide = true)]
    debug: Option<bool>,

    /// filename for the ouput (a zip file)
    #[arg(long, value_name = "zip")]
    output: Option<std::path::PathBuf>,

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
    match std::fs::File::open(filename) {
        Ok(mut f) => {
            let mut buffer = Vec::new();
            // read the whole file
            use std::io::prelude::*;
            f.read_to_end(&mut buffer).unwrap();
            buffer
        }
        Err(e) => {
            log::error!("{:?}", e);
            panic!("failed to read {}. Bye.", filename);
        }
    }
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

    let kinds: HashSet<Kind> = args.kinds.into_iter().collect();
    let track_segment = backend.trackSegment();
    {
        let points = backend.get_waypoints(&track_segment, kinds.clone());
        println!("* found {} points", points.len());
        for point in &points {
            let time = parse_time(&point.get_info().time);
            println!(
                "   {:>3.0} km [{}]: {:30} [{:10}]",
                point.get_info().distance / 1000.0,
                time.format("%H:%M"),
                point.name,
                point.get_info().origin
            )
        }
    }

    let mut parameters = backend.get_parameters();
    parameters.segment_length = 1000f64 * args.segment_length;
    parameters.segment_overlap = 1000f64 * args.segment_overlap;

    match args.start_time {
        Some(time) => {
            parameters.start_time = time;
        }
        _ => {}
    }

    parameters.speed = speed::mps(args.speed);

    match args.step_distance {
        Some(km) => {
            parameters.user_steps_options.step_elevation_gain = None;
            parameters.user_steps_options.step_distance = Some(1000.0 * km);
        }
        _ => {}
    }

    match args.step_elevation_gain {
        Some(m) => {
            parameters.user_steps_options.step_distance = None;
            parameters.user_steps_options.step_elevation_gain = Some(m);
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

    let zipname = match args.output {
        Some(path) => path.into_os_string().into_string().unwrap_or_default(),
        None => format!("{}.zip", gpxpath.file_stem().unwrap().to_str().unwrap()),
    };
    let zip = backend.generateZip(&kinds).await;
    println!("make: {}", zipname);
    std::fs::write(zipname, &zip).expect("Could not write pdf.");
    Ok(())
}
