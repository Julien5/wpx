#![allow(non_snake_case)]

use clap::Parser;
use tracks::backend::Backend;

/// Reads a GPX files and generates a PDF feuille de route and cutoff points.
#[derive(Parser)]
struct Cli {
    /// filename for the ouput (zip or pdf)
    #[arg(long, value_name = "name")]
    name: Option<String>,
}

fn setup_log() {
    // println!("init logger");
    // env_logger::init();

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
    let mut backend = Backend::make();
    if let Some(name) = args.name {
        let file = std::path::Path::new(&name);
        if name.ends_with(".gpx") && std::path::Path::exists(&file) {
            let mut gpxdata = Vec::new();
            gpxdata.push(read_file(&name));
            let parts = backend.load_track_parts(&gpxdata)?;
            let ordered = parts.clone(); //karl_order(&parts);
            let _ = backend.load_ordered(&ordered);
            let _ = backend.create_trackfile().await;
        }

        let mut trackfiles = backend.trackfiles().await.unwrap();
        trackfiles.retain(|file| file.name.contains(&name));
        for trackfile in trackfiles {
            backend.unload();
            let _ = backend.read_trackfile(&trackfile).await;
        }
    } else {
        for trackfile in backend.trackfiles().await.unwrap() {
            log::info!("{} - {}", trackfile.number, trackfile.name);
        }
    }

    Ok(())
}
