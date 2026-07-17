#![allow(non_snake_case)]

use clap::Parser;
use tokio_util::sync::CancellationToken;
use tracks::backend::Backend;
use tracks::event;
use tracks::osm::request::{Boxes, Request};
use tracks::osm::request_handler::get_response;
use tracks::osm::DownloadSideData;
use tracks::tile::{Chunks, Tile, Tiles};

/// cache tool
#[derive(Parser)]
struct Cli {
    #[arg(value_name = "gpx")]
    filenames: Vec<std::path::PathBuf>,
}

fn setup_log() {
    env_logger::init();
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

    let gpxinputs: Vec<_> = args
        .filenames
        .iter()
        .map(|p| p.as_os_str().to_str().unwrap())
        .collect();
    let mut gpxdata = Vec::new();
    for gpxinput in &gpxinputs {
        log::info!("read gpx {}", gpxinput);
        gpxdata.push(read_file(gpxinput));
    }

    let mut backend = Backend::make();
    let parts = backend.load_track_parts(&gpxdata)?;
    for part in &parts {
        println!("found segment: {}", part.name)
    }
    backend.load_ordered(&parts)?;
    let track = backend.track();
    let (tiles, chunks) = track.boxes(0f64, track.total_distance());
    let tile1 = Tile { coord: (110, 619) };
    let tile2 = Tile { coord: (121, 619) };
    let (_tiles, _chunks) = (
        Tiles::from([tile1.clone(), tile2.clone()]),
        Chunks::from([tile1.chunk(), tile2.chunk()]),
    );
    let mut boxes = Vec::new();
    boxes.push(Boxes::from_tiles(&tiles));
    boxes.push(Boxes::from_chunks(&chunks));
    let request = Request { boxes };

    let b: event::SenderHandler = Box::new(event::ConsoleEventSender {});
    let logger = std::sync::RwLock::new(Some(b));
    let token = CancellationToken::new();
    let side = DownloadSideData {
        logger: &logger,
        cancel_token: &token,
    };

    let try_download = true;
    if let Ok((chunk_data, _missing_box_count)) = get_response(&request, &side, try_download).await
    {
        for (tile, tile_features) in &chunk_data.data.tiles {
            log::info!("tile: {:?} len:{}", tile, tile_features.len(),);
        }
        for (chunk, chunk_features) in &chunk_data.data.chunks {
            log::info!("chunk: {:?} len:{}", chunk, chunk_features.len(),);
        }
    }
    Ok(())
}
