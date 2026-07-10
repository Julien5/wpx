#![allow(dead_code)]
use std::sync::Arc;

use tokio_util::sync::CancellationToken;

use crate::{
    backend_data::BackendData,
    controls, event,
    gpsdata::GpxData,
    make_points,
    osm::{self, DownloadSideData},
    parameters::Parameters,
    point_collection::{Kind, PacketProvider, PointCollection},
    speed::powergeometry::ConstantPowerGeometry,
    track::Track,
};

fn read(filename: &str) -> GpxData {
    use crate::gpsdata;
    let mut f = std::fs::File::open(filename).unwrap();
    let mut content = Vec::new();
    // read the whole file
    use std::io::prelude::*;
    f.read_to_end(&mut content).unwrap();
    gpsdata::GpxData::read_content(&content).unwrap()
}

fn load_backend_data_with_parameters_no_osm(filename: &str, parameters: Parameters) -> BackendData {
    let gpxdata = read(filename);
    let track = Arc::new(Track::from_tracks(&gpxdata.tracks).unwrap());
    log::trace!("  track length: {}m", track.total_distance());
    let mut collection = PointCollection::new();
    {
        let mut waypoints = gpxdata.waypoints.clone();
        for w in &mut waypoints {
            track.project_point(w);
        }
        collection.import_other(&Kind::GPXWaypoints, waypoints);
    }
    let waypoints = collection.get_vector(&Kind::GPXWaypoints);
    let mut controls = controls::infer_controls_from_gpx_segments(&track, &waypoints);
    for c in &mut controls {
        track.project_point(c);
    }

    let controls = controls::infer_controls_from_gpx_segments(&track, &waypoints);

    collection.import_other(&Kind::GPXWaypoints, waypoints);
    collection.import_other(&Kind::Controls, controls);

    let usersteps = make_points::user_points(&track, &parameters.user_steps_options);
    collection.import_other(&Kind::CutOff, usersteps);

    let mut packet_provider = PacketProvider::new();
    packet_provider.collection = collection;
    let power_geometry = ConstantPowerGeometry::new(&track.geometry);
    BackendData {
        parameters,
        track,
        packet_provider,
        trackfile: None,
        power_geometry,
    }
}

async fn load_osm(track: &Track, collection: &mut PointCollection) -> usize {
    let b: event::SenderHandler = Box::new(event::ConsoleEventSender {});
    let logger = std::sync::RwLock::new(Some(b));
    let token = CancellationToken::new();
    let side = DownloadSideData {
        logger: &logger,
        cancel_token: &token,
    };
    let try_download = true;
    // use try_download if necessary.
    let (mut osmpoints, missing_box_count) = osm::download_for_track(&track, &side, !try_download)
        .await
        .unwrap();
    track.project_map(&mut osmpoints);
    collection.import_osm(&osmpoints.as_vector());
    missing_box_count
}

pub fn load_file(filename: &str) -> (Track, GpxData) {
    let gpxdata = read(filename);
    (Track::from_tracks(&gpxdata.tracks).unwrap(), gpxdata)
}

pub async fn load_backend_data_with_track_and_parameters(
    track: Track,
    gpxdata: GpxData,
    parameters: Parameters,
    with_osm: bool,
) -> BackendData {
    log::trace!("  track length: {}m", track.total_distance());
    let mut collection = PointCollection::new();
    {
        let mut waypoints = gpxdata.waypoints.clone();
        for w in &mut waypoints {
            track.project_point(w);
        }
        collection.import_other(&Kind::GPXWaypoints, waypoints);
    }
    let waypoints = collection.get_vector(&Kind::GPXWaypoints);

    if with_osm {
        load_osm(&track, &mut collection).await;
    }

    let mut controls = controls::infer_controls_from_gpx_segments(&track, &waypoints);

    for c in &mut controls {
        track.project_point(c);
    }

    collection.import_other(&Kind::GPXWaypoints, waypoints);
    collection.import_other(&Kind::Controls, controls);

    let usersteps = make_points::user_points(&track, &parameters.user_steps_options);
    collection.import_other(&Kind::CutOff, usersteps);

    let mut packet_provider = PacketProvider::new();
    packet_provider.collection = collection;
    let power_geometry = ConstantPowerGeometry::new(&track.geometry);
    BackendData {
        parameters,
        track: Arc::new(track),
        packet_provider,
        trackfile: None,
        power_geometry,
    }
}

pub async fn load_backend_data_with_parameters(
    filename: &str,
    parameters: Parameters,
    with_osm: bool,
) -> BackendData {
    let (track, gpxdata) = load_file(filename);
    log::trace!("  track length: {}m", track.total_distance());
    load_backend_data_with_track_and_parameters(track, gpxdata, parameters, with_osm).await
}

pub fn load_backend_data_without_osm(filename: &str) -> BackendData {
    load_backend_data_with_parameters_no_osm(filename, crate::parameters::Parameters::default())
}

pub async fn load_backend_data(filename: &str) -> BackendData {
    load_backend_data_with_parameters(filename, crate::parameters::Parameters::default(), true)
        .await
}
