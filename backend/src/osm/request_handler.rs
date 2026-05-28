use std::hash::{DefaultHasher, Hash, Hasher};

use crate::{
    error::{GenericResult, TrackError},
    event,
    osm::{download, request::*, request_cache::*, request_parse::parse, DownloadSideData},
};

fn hash(data: &String) -> String {
    let mut hasher = DefaultHasher::new();
    data.hash(&mut hasher);
    let final_hash: u64 = hasher.finish();
    let hex_string = format!("{:x}", final_hash);
    let short_hash = &hex_string[0..4];
    format!("{}", short_hash)
}

#[cfg(not(target_arch = "wasm32"))]
fn save_resquest_response_for_debug(hash: &str, req: &str, resp: &str) {
    std::fs::write(format!("/tmp/request-{}.txt", hash), req).unwrap();
    std::fs::write(format!("/tmp/response-{}.txt", hash), &resp).unwrap();
}

#[cfg(target_arch = "wasm32")]
fn save_resquest_response_for_debug(hash: &str, req: &str, resp: &str) {}

async fn download(req_string: &String, side: &DownloadSideData<'_>) -> GenericResult<Vec<u8>> {
    //log::trace!("download:\n{}\n", req_string);
    let hash = hash(&req_string);

    let respfilename = format!("data/osm/response-{}.txt", hash);
    match std::fs::exists(&respfilename) {
        Ok(true) => {
            log::trace!("found response file {}", respfilename);
            return Ok(std::fs::read(&respfilename).unwrap());
        }
        _ => {}
    };
    log::trace!("not found {}", respfilename);

    let mut nretries = 0;
    loop {
        match download::dl_worker(&req_string, &side).await {
            Err(e) => {
                if let Some(TrackError::OSMDownloadCancelled) = e.downcast_ref::<TrackError>() {
                    log::info!("user cancelled download");
                    return Err(e.into());
                } else {
                    log::error!("download failed, error = {}, retry = {}", e, nretries);
                }
                // sleep between retries, (increase the chance overpass server processes the request)
                crate::sleep::sleep_ms(500).await;
                nretries += 1;
            }
            Ok(content) => {
                // if debug
                if cfg!(debug_assertions) {
                    save_resquest_response_for_debug(&hash, &req_string, &content);
                }
                return Ok(content.into_bytes());
            }
        }
    }
}

pub async fn get_response(
    request: &Request,
    side: &DownloadSideData<'_>,
) -> GenericResult<ChunkData> {
    let (chunk_data, missing_request) = read_cache(request, side.logger).await;
    if missing_request.boxes.is_empty() {
        log::trace!("complete cache hit.");
        return Ok(chunk_data);
    }

    log::trace!("incomplete cache hit.");
    let missing = missing_request.strings();
    for (index, pair) in missing.iter().enumerate() {
        let (missing_zones, missing_req_string) = pair;
        log::trace!(
            "request[{}/{}] with {} missing tile bboxes",
            index,
            missing.len(),
            missing_zones.tiles.len(),
        );
        log::trace!(
            "request[{}/{}] with {} missing chunk bboxes",
            index,
            missing.len(),
            missing_zones.chunks.len(),
        );
        event::send_worker(
            &side.logger,
            &format!("osm:download-progress:{}:{}", index, missing.len()),
        );
        match download(&missing_req_string, &side).await {
            Ok(data) => {
                log::trace!("response length: {} bytes", data.len());
                match parse(&data) {
                    Ok(response) => {
                        let _ = super::request_cache::write_cache(
                            &missing_request,
                            &response,
                            &side.logger,
                        )
                        .await;
                    }
                    Err(e) => {
                        log::error!("could not parse response: {:?}", e);
                    }
                }
            }
            Err(e) => {
                log::error!("error:{:?}", e);
            }
        }
    }

    let (chunk_data, missing) = super::request_cache::read_cache(&request, side.logger).await;
    if missing.boxes.is_empty() {
        log::trace!("complete cache hit.");
        return Ok(chunk_data);
    }
    log::trace!("incomplete cache hit.");
    Err(TrackError::OSMDownloadFailed.into())
}
