#![allow(non_snake_case)]

use clap::Parser;
use std::collections::BTreeMap;
use tracks::backend::{Backend, Segment};
use tracks::parameters::TimeAxis;
use tracks::speed;

/// Reads a GPX file and generates a track-waypoints GPX with power parameters.
#[derive(Parser)]
struct Cli {
    #[arg(value_name = "gpx")]
    filename: std::path::PathBuf,

    #[arg(long, value_name = "output", default_value = "/tmp/output.gpx")]
    output: std::path::PathBuf,

    #[arg(long, value_name = "constant_speed", default_value_t = false)]
    constant_speed: bool,

    #[arg(long, value_name = "weight", default_value_t = 80.0)]
    weight: f64,

    #[arg(long, value_name = "headwind", default_value_t = 0.0)]
    headwind: f64,

    #[arg(long, value_name = "cd", default_value_t = 0.9)]
    cd: f64,

    #[arg(long, value_name = "drivetrain_loss", default_value_t = 2.0)]
    drivetrain_loss: f64,

    #[arg(long, value_name = "start_time")]
    start_time: Option<String>,

    #[arg(long, value_name = "speed", default_value = "15.0")]
    speed: String,
}

fn setup_log() {
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
    match std::fs::File::open(filename) {
        Ok(mut f) => {
            let mut buffer = Vec::new();
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
    setup_log();

    let args = Cli::parse();

    let mut backend = Backend::make();
    let _ = Backend::init_pdf_fonts().await;

    let gpxinput = args.filename.as_os_str().to_str().unwrap();
    log::info!("read gpx {}", gpxinput);
    let gpxdata = read_file(gpxinput);
    let parts = backend.load_track_parts(&vec![gpxdata])?;
    for part in &parts {
        println!("found segment: {}", part.name)
    }

    let mut params = tracks::parameters::Parameters::default();
    match args.constant_speed {
        true => {
            params.profile_options.time_axis = TimeAxis::ConstantSpeed;
        }
        false => {
            params.profile_options.time_axis = TimeAxis::ConstantPower;
            params.power_parameters.W = args.weight;
            params.power_parameters.Vhw = args.headwind;
            params.power_parameters.Cd = args.cd;
            params.power_parameters.DrivetrainLoss = args.drivetrain_loss;
        }
    }

    backend.load_ordered(&parts)?;
    let _ = backend.load_osm_without_download().await;

    let track_segment: Segment = backend.trackSegment();

    match args.start_time {
        Some(time) => {
            params.start_time = time;
        }
        _ => {}
    }

    params.speed = match args.speed.parse::<f64>() {
        Ok(kmh) => speed::format_kmh(kmh),
        Err(_) => {
            let allowed = speed::allowed_speeds(track_segment.end);
            match allowed.iter().find(|spec| spec.contains(&args.speed)) {
                Some(spec) => {
                    log::info!("using speed:{}", spec);
                    spec.clone()
                }
                None => args.speed.clone(),
            }
        }
    };

    backend.set_parameters(&params);

    let gpx_map: BTreeMap<String, Vec<u8>> = backend.generateGpx();

    let output_data = gpx_map
        .get("track-waypoints.gpx")
        .expect("track-waypoints.gpx not found in generated GPX data");

    std::fs::write(&args.output, output_data).expect("Failed to write output GPX");

    let stats = backend.statistics();
    println!("length = {:.1} km", stats.length / 1000f64);
    println!("elevation gain = {:.1} m", stats.elevation_gain);
    println!("wrote: {}", args.output.display());

    Ok(())
}
