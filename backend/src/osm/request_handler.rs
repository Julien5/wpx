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

use serde::Deserialize;

#[derive(Deserialize)]
struct OverpassResponse {
    // We use Option because 'remark' only exists when there is an error/timeout
    remark: Option<String>,
}

fn is_timeout(content: &str) -> bool {
    // Parse the JSON safely. If it's totally malformed JSON,
    // it's not a successful timeout message, so we default to false.
    if let Ok(response) = serde_json::from_str::<OverpassResponse>(content) {
        if let Some(remark_text) = response.remark {
            return remark_text.contains("timed out");
        }
    }
    false
}

async fn download(req_string: &String, side: &DownloadSideData<'_>) -> GenericResult<Vec<u8>> {
    let hash = hash(&req_string);

    let respfilename = format!("data/osm/response-{}.txt", hash);
    match std::fs::exists(&respfilename) {
        Ok(true) => {
            return Ok(std::fs::read(&respfilename).unwrap());
        }
        _ => {}
    };

    let mut nretries = 0;
    loop {
        match download::dl_worker(&req_string, &side).await {
            Err(e) => {
                if let Some(TrackError::OSMDownloadCancelled) = e.downcast_ref::<TrackError>() {
                    log::info!("user cancelled download");
                    // stop
                    return Err(e.into());
                } else {
                    log::error!("download failed, error = {}, retry = {}", e, nretries);
                }
            }
            Ok(content) => {
                // if debug
                if cfg!(debug_assertions) {
                    save_resquest_response_for_debug(&hash, &req_string, &content);
                }
                if is_timeout(&content) {
                    log::error!("download failed, error = timeout, retry = {}", nretries);
                } else {
                    return Ok(content.into_bytes());
                }
            } // sleep between retries, (increase the chance overpass server processes the request)
        }
        event::send_worker(&side.logger, &format!("osm:retry:{}", nretries));
        crate::sleep::sleep_ms(500).await;
        nretries += 1;
    }
}

pub async fn get_response(
    request: &Request,
    side: &DownloadSideData<'_>,
    try_download: bool,
) -> GenericResult<(ChunkData, usize)> {
    let (chunk_data, missing_request) = read_cache(request, side.logger).await;
    if missing_request.boxes.is_empty() {
        log::info!("complete cache hit.");
        return Ok((chunk_data, 0));
    }
    if !try_download {
        log::info!("incomplete cache hit, and do not try to download osm data.");
        // problem: we cannot return an error *and* the data.
        return Ok((chunk_data, missing_request.boxes.len()));
    }

    log::info!("incomplete cache hit.");
    let missing = missing_request.strings();
    for (index, pair) in missing.iter().enumerate() {
        let (_missing_zones, missing_req_string) = pair;
        event::send_worker(
            &side.logger,
            &format!("osm:download-progress:{}:{}", index, missing.len()),
        );
        match download(&missing_req_string, &side).await {
            Ok(data) => {
                log::info!("response length: {} bytes", data.len());
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
                if let Some(TrackError::OSMDownloadCancelled) = e.downcast_ref::<TrackError>() {
                    log::info!("user cancelled download");
                    // stop
                    return Err(e.into());
                }
            }
        }
    }

    let (chunk_data, missing) = super::request_cache::read_cache(&request, side.logger).await;
    if missing.boxes.is_empty() {
        log::info!("complete cache hit.");
        return Ok((chunk_data, 0));
    }
    log::info!("incomplete cache hit.");
    Err(TrackError::OSMDownloadFailed.into())
}
